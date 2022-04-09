use std::time::Duration;

use gasket::{
    error::AsWorkError,
    messaging::{connect_ports, InputPort},
    metrics::{self, Registry},
    retries,
    runtime::{spawn_stage, Policy, WorkOutcome, WorkResult, Worker},
};
use pallas::network::miniprotocols::Point;
use scrolls::sources::{self, transport};

struct Terminal {
    input: InputPort<sources::n2n::ChainSyncCommandEx>,
}

impl Worker for Terminal {
    fn metrics(&self) -> Registry {
        metrics::Builder::new().build()
    }

    fn work(&mut self) -> WorkResult {
        let unit = self.input.recv()?;
        println!("{:?}", unit.payload);

        Ok(WorkOutcome::Partial)
    }
}

fn main() {
    env_logger::init();

    let byron_point = Point::Specific(
        43159,
        hex::decode("f5d398d6f71a9578521b05c43a668b06b6103f94fcf8d844d4c0aa906704b7a6").unwrap(),
    );

    let alonzo_point = Point::Specific(
        57867490,
        hex::decode("c491c5006192de2c55a95fb3544f60b96bd1665accaf2dfa2ab12fc7191f016b").unwrap(),
    );

    let mut transport = gasket::retries::retry_operation(
        || {
            transport::Transport::setup(&transport::Config {
                address: "relays-new.cardano-mainnet.iohk.io:3001".to_string(),
                magic: pallas::network::miniprotocols::MAINNET_MAGIC,
                protocols: &[0, 2, 3],
            })
            .or_work_err()
        },
        &retries::Policy {
            max_retries: 5,
            backoff_factor: 2,
            backoff_unit: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        },
        None,
    )
    .expect("conneced transport after several retries");

    let mut chainsync = scrolls::sources::n2n::chainsync::Worker::new(
        transport.muxer.use_channel(2),
        scrolls::sources::n2n::chainsync::Config {
            min_depth: 0,
            known_points: Some(vec![alonzo_point]),
        },
    );

    let mut blockfetch = scrolls::sources::n2n::blockfetch::Worker::new(
        transport.muxer.use_channel(3),
        scrolls::sources::n2n::blockfetch::Config {},
    );

    let mut reducer1 = scrolls::reducers::utxo_by_address::Worker::new(
        scrolls::reducers::utxo_by_address::Config {
            hrp: "addr".to_string(),
        },
    );

    let mut redis = scrolls::storage::redis::Worker::new(scrolls::storage::redis::Config {
        connection_params: "redis://127.0.0.1:6379".to_string(),
    });

    connect_ports(&mut chainsync.output, &mut blockfetch.input, 10);
    connect_ports(&mut blockfetch.output, &mut reducer1.input, 10);
    connect_ports(&mut reducer1.output, &mut redis.input, 10);

    let tether1 = spawn_stage(
        chainsync,
        Policy {
            tick_timeout: Some(Duration::from_secs(3)),
            bootstrap_retry: retries::Policy {
                max_retries: 5,
                backoff_factor: 2,
                backoff_unit: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
            },
            work_retry: retries::Policy::no_retry(),
            teardown_retry: retries::Policy::no_retry(),
        },
    );

    let tether2 = spawn_stage(
        blockfetch,
        Policy {
            tick_timeout: None,
            bootstrap_retry: retries::Policy::no_retry(),
            work_retry: retries::Policy::no_retry(),
            teardown_retry: retries::Policy::no_retry(),
        },
    );

    let tether3 = spawn_stage(
        reducer1,
        Policy {
            tick_timeout: None,
            bootstrap_retry: retries::Policy::no_retry(),
            work_retry: retries::Policy::no_retry(),
            teardown_retry: retries::Policy::no_retry(),
        },
    );

    let tether4 = spawn_stage(
        redis,
        Policy {
            tick_timeout: None,
            bootstrap_retry: retries::Policy::no_retry(),
            work_retry: retries::Policy::no_retry(),
            teardown_retry: retries::Policy::no_retry(),
        },
    );

    let tethers = vec![
        ("chainsync", tether1),
        ("blockfetch", tether2),
        ("reducer1", tether3),
        ("redis", tether4),
    ];

    loop {
        for (name, tether) in tethers.iter() {
            match tether.check_state() {
                gasket::runtime::TetherState::Dropped => log::warn!("{} stage dropped", name),
                gasket::runtime::TetherState::Blocked(x) => {
                    log::warn!("{} stage blocked, state: {:?}", name, x);
                }
                gasket::runtime::TetherState::Alive(x) => {
                    log::info!("{} stage alive, state: {:?}", name, x);
                }
            }

            match tether.read_metrics() {
                Ok(readings) => {
                    for (key, value) in readings {
                        log::info!("stage {}, metric {}: {:?}", name, key, value);
                    }
                }
                Err(err) => {
                    println!("couldn't read metrics");
                    dbg!(err);
                }
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }

    //for (name, tether) in tethers {
    //    log::warn!("{}", name);
    //    tether.dismiss_stage().expect("stage stops");
    //    tether.join_stage();
    //}
}
