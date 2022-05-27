use std::time::Duration;

use clap::ArgMatches;
use scrolls::{bootstrap, crosscut, reducers, sources, storage};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    N2N(sources::n2n::Config),

    #[cfg(target_family = "unix")]
    N2C(sources::n2c::Config),
}

impl SourceConfig {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
        cursor: &crosscut::Cursor,
    ) -> sources::Plugin {
        match self {
            SourceConfig::N2N(c) => c.plugin(chain, intersect, cursor),
            SourceConfig::N2C(c) => c.plugin(chain, intersect, cursor),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ReducerConfig {
    UtxoByAddress(reducers::utxo_by_address::Config),
    PointByTx(reducers::point_by_tx::Config),
    PoolByStake(reducers::pool_by_stake::Config),
}

impl ReducerConfig {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> reducers::Plugin {
        match self {
            ReducerConfig::UtxoByAddress(c) => c.plugin(chain, intersect),
            ReducerConfig::PointByTx(c) => c.plugin(chain, intersect),
            ReducerConfig::PoolByStake(c) => c.plugin(chain, intersect),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    Redis(storage::redis::Config),
}

impl StorageConfig {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> storage::Plugin {
        match self {
            StorageConfig::Redis(c) => c.plugin(chain, intersect),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ChainConfig {
    Mainnet,
    Testnet,
    Custom(crosscut::ChainWellKnownInfo),
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl From<ChainConfig> for crosscut::ChainWellKnownInfo {
    fn from(other: ChainConfig) -> Self {
        match other {
            ChainConfig::Mainnet => crosscut::ChainWellKnownInfo::mainnet(),
            ChainConfig::Testnet => crosscut::ChainWellKnownInfo::testnet(),
            ChainConfig::Custom(x) => x,
        }
    }
}

#[derive(Deserialize)]
struct ConfigRoot {
    source: SourceConfig,
    reducers: Vec<ReducerConfig>,
    storage: StorageConfig,
    intersect: crosscut::IntersectConfig,
    chain: Option<ChainConfig>,
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

    let explicit_config = match args.is_present("config") {
        true => {
            let config_file_path = args
                .value_of_t("config")
                .map_err(|err| scrolls::Error::ConfigError(format!("{:?}", err)))?;

            Some(config_file_path)
        }
        false => None,
    };

    let config = ConfigRoot::new(explicit_config)
        .map_err(|err| scrolls::Error::ConfigError(format!("{:?}", err)))?;

    let chain = config.chain.unwrap_or_default().into();

    // We need to setup the storage first so that we can retrieve the potential
    // cursor
    let storage = config.storage.plugin(&chain, &config.intersect);

    let cursor = storage.read_cursor()?;

    // We can now setup the source plugin specifying a potential cursor
    let source = config.source.plugin(&chain, &config.intersect, &cursor);

    let pipeline = bootstrap::build(
        source,
        config
            .reducers
            .into_iter()
            .map(|x| x.plugin(&chain, &config.intersect))
            .collect(),
        storage,
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
