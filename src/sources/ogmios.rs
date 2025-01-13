
use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, info};

use pallas_primitives::conway::{MintedBlock};
use pallas::network::miniprotocols::Point;

use crate::framework::errors::Error;
use crate::framework::*;

pub mod ogmios;

use crate::sources::ogmios::ogmios::{OgmiosResponse};
use crate::sources::ogmios::ogmios::OgmiosClient;
use crate::sources::hydra::make_conway_block;

#[derive(Stage)]
#[stage(
    name = "source-ogmios",
    unit = "OgmiosResponse"
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,
}

async fn intersect_from_config(
    peer: &mut OgmiosClient,
) -> Result<(), WorkerError> {
    let chainsync = peer.chainsync();

    info!("intersecting origin");
    chainsync.intersect_origin().await.or_restart()?;

    Ok(())
}

pub struct Worker {
    peer_session: OgmiosClient,
}

impl Worker {
    async fn process_next(
        &mut self,
        stage: &mut Stage,
        next: &OgmiosResponse,
    ) -> Result<(), WorkerError> {
        match next {
            OgmiosResponse::RollForward(rf) => {
                debug!(?rf, "chain sync roll forward");
                let block = make_conway_block(
                    rf.transactions.iter().map(|tx| tx.cbor.clone()).collect(),
                    rf.slot,
                ).or_panic()?;
                let block_minted: MintedBlock = minicbor::decode(&block.cbor).or_panic()?;
                let wrapped_block = (7u16, block_minted);
                let block_cbor = minicbor::to_vec(wrapped_block).or_panic()?;
                let evt = ChainEvent::apply(
                    Point::Specific(block.slot, block.hash.to_vec()),
                    Record::RawBlockPayload(block_cbor.to_vec()),
                );
                stage.output.send(evt).await.or_panic()?;
                stage.chain_tip.set(rf.tip.slot as i64);
                Ok(())
            }
            OgmiosResponse::RollBackward(rb) => {
                debug!(?rb, "chain sync roll backward");
                match &rb.point {
                    Point::Origin => debug!("rollback to origin"),
                    Point::Specific(slot, _) => debug!(slot, "rollback"),
                };
                Ok(())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let mut peer_session = OgmiosClient::connect(&stage.config.ogmios_url)
            .await
            .or_retry()?;

        intersect_from_config(&mut peer_session).await?;

        let worker = Self { peer_session };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<OgmiosResponse>, WorkerError> {
        let client = self.peer_session.chainsync();

        info!("requesting next block");
        let next = client.request_next().await.or_restart()?;

        Ok(WorkSchedule::Unit(next))
    }

    async fn execute(
        &mut self,
        unit: &OgmiosResponse,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        self.process_next(stage, unit).await
    }

    async fn teardown(&mut self) -> Result<(), WorkerError> {
        self.peer_session.abort();

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    ogmios_url: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
