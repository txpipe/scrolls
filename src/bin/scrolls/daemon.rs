use std::time::Duration;

use clap::ArgMatches;
use scrolls::{bootstrap, collections, crosscut, sources, storage};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
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

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ReducerConfig {
    UtxoByAddress(collections::utxo_by_address::Config),
    PointByTx(collections::point_by_tx::Config),
}

impl From<ReducerConfig> for collections::Plugin {
    fn from(other: ReducerConfig) -> Self {
        match other {
            ReducerConfig::UtxoByAddress(c) => collections::Plugin::UtxoByAddress(c.into()),
            ReducerConfig::PointByTx(c) => collections::Plugin::PointByTx(c.into()),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(crosscut::PointArg),
    Fallbacks(Vec<crosscut::PointArg>),
}

#[derive(Deserialize)]
struct ConfigRoot {
    source: SourceConfig,
    reducers: Vec<ReducerConfig>,
    storage: StorageConfig,
    intersect: IntersectConfig,
}

impl ConfigRoot {
    pub fn new(explicit_file: Option<String>) -> Result<Self, config::ConfigError> {
        let mut s = config::Config::builder();

        // our base config will always be in /etc/scrolls
        s = s.add_source(config::File::with_name("/etc/scrolls/daemon.toml").required(false));

        // but we can override it by having a file in the working dir
        s = s.add_source(config::File::with_name("scrolls.toml").required(false));

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file {
            s = s.add_source(config::File::with_name(&explicit).required(true));
        }

        // finally, we use env vars to make some last-step overrides
        s = s.add_source(config::Environment::with_prefix("SCROLLS").separator("_"));

        s.build()?.try_deserialize()
    }
}

pub fn run(args: &ArgMatches) -> Result<(), scrolls::Error> {
    env_logger::init();

    let config =
        ConfigRoot::new(None).map_err(|err| scrolls::Error::ConfigError(format!("{:?}", err)))?;

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
