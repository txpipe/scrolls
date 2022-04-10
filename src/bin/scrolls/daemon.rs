use std::time::Duration;

use clap::ArgMatches;
use scrolls::{bootstrap, collections, sources, storage};

pub enum SourceConfig {
    N2N(sources::n2n::Config),
}

impl From<SourceConfig> for sources::Plugin {
    fn from(other: SourceConfig) -> Self {
        match other {
            SourceConfig::N2N(c) => sources::Plugin::N2N(c.into()),
        }
    }
}

pub enum ReducerConfig {
    UtxoByAddress(collections::utxo_by_address::Config),
}

impl From<ReducerConfig> for collections::Plugin {
    fn from(other: ReducerConfig) -> Self {
        match other {
            ReducerConfig::UtxoByAddress(c) => collections::Plugin::UtxoByAddress(c.into()),
        }
    }
}
pub enum StorageConfig {
    Redis(storage::redis::Config),
}

impl From<StorageConfig> for storage::Plugin {
    fn from(other: StorageConfig) -> Self {
        match other {
            StorageConfig::Redis(c) => storage::Plugin::Redis(c.into()),
        }
    }
}

pub struct Config {
    source: SourceConfig,
    reducers: Vec<ReducerConfig>,
    storage: StorageConfig,
}

pub fn run(args: &ArgMatches) -> Result<(), scrolls::Error> {
    env_logger::init();

    let config = Config {
        source: SourceConfig::N2N(sources::n2n::Config {
            address: "relays-new.cardano-mainnet.iohk.io:3001".to_string(),
            magic: pallas::network::miniprotocols::MAINNET_MAGIC,
        }),
        reducers: vec![ReducerConfig::UtxoByAddress(
            collections::utxo_by_address::Config {
                hrp: "addr".to_string(),
            },
        )],
        storage: StorageConfig::Redis(storage::redis::Config {
            connection_params: "redis://127.0.0.1:6379".to_string(),
        }),
    };

    let pipeline = bootstrap::build(
        config.source.into(),
        config.reducers.into_iter().map(|x| x.into()).collect(),
        config.storage.into(),
    );

    loop {
        for (name, tether) in pipeline.tethers.iter() {
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

/// Creates the clap definition for this sub-command
pub(crate) fn command_definition<'a>() -> clap::Command<'a> {
    clap::Command::new("daemon").arg(
        clap::Arg::new("config")
            .long("config")
            .takes_value(true)
            .help("config file to load by the daemon"),
    )
}
