use pallas::crypto::hash::{Hash, Hasher};
use pallas::ledger::primitives::alonzo;
use pallas::ledger::primitives::alonzo::PlutusScript;
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
}

impl Reducer {
    fn send_set_add(
        &mut self,
        slot: u64,
        hash: Hash<28>,
        cbor: PlutusScript,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = format!("{}", hash);
        let member = hex::encode::<Vec<u8>>(cbor.into());

        let crdt = model::CRDTCommand::LastWriteWins(key, member, slot);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        slot: u64,
        tx: &alonzo::TransactionWitnessSet,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if let Some(plutus_scripts) = &tx.plutus_script {
            for scr in plutus_scripts.iter() {
                let hash = Hasher::<224>::hash_cbor(scr);

                self.send_set_add(slot, hash, scr.clone(), output)?;
            }
        }

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                x.1.transaction_witness_sets.iter().try_for_each(|tx| {
                    self.reduce_alonzo_compatible_tx(x.1.header.header_body.slot, tx, output)
                })
            }
        }
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            address_hrp: chain.address_hrp.clone(),
        };

        super::Reducer::PlutusScriptByHash(reducer)
    }
}
