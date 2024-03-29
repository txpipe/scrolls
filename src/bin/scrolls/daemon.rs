use clap;

use gasket::runtime::Tether;
use scrolls::{enrich, framework::*, reducers, sources, storage};
use serde::Deserialize;
use std::{collections::VecDeque, time::Duration};
use tracing::{info, warn};

use crate::console;

// use crate::console;

#[derive(Deserialize)]
struct ConfigRoot {
    source: sources::Config,
    enrich: Option<enrich::Config>,
    reducer: reducers::Config,
    storage: storage::Config,
    intersect: IntersectConfig,
    finalize: Option<FinalizeConfig>,
    chain: Option<ChainConfig>,
    retries: Option<gasket::retries::Policy>,
}
impl ConfigRoot {
    pub fn new(explicit_file: &Option<std::path::PathBuf>) -> Result<Self, config::ConfigError> {
        let mut s = config::Config::builder();

        // our base config will always be in /etc/scrolls
        s = s.add_source(config::File::with_name("/etc/scrolls/daemon.toml").required(false));

        // but we can override it by having a file in the working dir
        s = s.add_source(config::File::with_name("scrolls.toml").required(false));

        // if an explicit file was passed, then we load it as mandatory
        if let Some(explicit) = explicit_file.as_ref().and_then(|x| x.to_str()) {
            s = s.add_source(config::File::with_name(explicit).required(true));
        }

        // finally, we use env vars to make some last-step overrides
        s = s.add_source(config::Environment::with_prefix("SCROLLS").separator("_"));

        s.build()?.try_deserialize()
    }
}

struct Runtime {
    source: Tether,
    enrich: Tether,
    reducer: Tether,
    storage: Tether,
}
impl Runtime {
    fn all_tethers(&self) -> impl Iterator<Item = &Tether> {
        vec![&self.source, &self.enrich, &self.reducer, &self.storage].into_iter()
    }

    fn should_stop(&self) -> bool {
        self.all_tethers().any(|tether| match tether.check_state() {
            gasket::runtime::TetherState::Alive(x) => {
                matches!(x, gasket::runtime::StagePhase::Ended)
            }
            _ => true,
        })
    }

    fn shutdown(&self) {
        for tether in self.all_tethers() {
            let state = tether.check_state();
            warn!("dismissing stage: {} with state {:?}", tether.name(), state);
            tether.dismiss_stage().expect("stage stops");

            // Can't join the stage because there's a risk of deadlock, usually
            // because a stage gets stuck sending into a port which depends on a
            // different stage not yet dismissed. The solution is to either
            // create a DAG of dependencies and dismiss in the
            // correct order, or implement a 2-phase teardown where
            // ports are disconnected and flushed before joining the
            // stage.

            //tether.join_stage();
        }
    }
}

fn define_gasket_policy(config: Option<&gasket::retries::Policy>) -> gasket::runtime::Policy {
    let default_policy = gasket::retries::Policy {
        max_retries: 20,
        backoff_unit: Duration::from_secs(1),
        backoff_factor: 2,
        max_backoff: Duration::from_secs(60),
        dismissible: false,
    };

    gasket::runtime::Policy {
        tick_timeout: None,
        bootstrap_retry: config.cloned().unwrap_or(default_policy.clone()),
        work_retry: config.cloned().unwrap_or(default_policy.clone()),
        teardown_retry: config.cloned().unwrap_or(default_policy.clone()),
    }
}

fn chain_stages<'a>(
    source: &'a mut dyn StageBootstrapper,
    enrich: &'a mut dyn StageBootstrapper,
    reducer: &'a mut dyn StageBootstrapper,
    storage: &'a mut dyn StageBootstrapper,
) {
    let (to_next, from_prev) = gasket::messaging::tokio::mpsc_channel(100);
    source.connect_output(to_next);
    enrich.connect_input(from_prev);

    let (to_next, from_prev) = gasket::messaging::tokio::mpsc_channel(100);
    enrich.connect_output(to_next);
    reducer.connect_input(from_prev);

    let (to_next, from_prev) = gasket::messaging::tokio::mpsc_channel(100);
    reducer.connect_output(to_next);
    storage.connect_input(from_prev);
}

fn bootstrap(
    mut source: sources::Bootstrapper,
    mut enrich: enrich::Bootstrapper,
    mut reducer: reducers::Bootstrapper,
    mut storage: storage::Bootstrapper,
    policy: gasket::runtime::Policy,
) -> Result<Runtime, Error> {
    chain_stages(&mut source, &mut enrich, &mut reducer, &mut storage);

    let runtime = Runtime {
        source: source.spawn(policy.clone()),
        enrich: enrich.spawn(policy.clone()),
        reducer: reducer.spawn(policy.clone()),
        storage: storage.spawn(policy.clone()),
    };

    Ok(runtime)
}

pub fn run(args: &Args) -> Result<(), Error> {
    console::initialize(&args.console);

    let config = ConfigRoot::new(&args.config).map_err(Error::config)?;

    let chain = config.chain.unwrap_or_default();
    let intersect = config.intersect;
    let finalize = config.finalize;
    // let current_dir = std::env::current_dir().unwrap();

    // TODO: load from persistence mechanism
    let cursor = Cursor::new(VecDeque::new());

    let ctx = Context {
        chain,
        intersect,
        finalize,
        cursor,
    };

    let source = config.source.bootstrapper(&ctx)?;
    let enrich = config
        .enrich
        .unwrap_or(enrich::Config::default())
        .bootstrapper(&ctx)?;

    let reducer = config.reducer.bootstrapper(&ctx)?;
    let storage = config.storage.bootstrapper(&ctx)?;

    let retries = define_gasket_policy(config.retries.as_ref());
    let runtime = bootstrap(source, enrich, reducer, storage, retries)?;

    info!("Scrolls is running...");

    while !runtime.should_stop() {
        console::refresh(&args.console, runtime.all_tethers());
        std::thread::sleep(Duration::from_millis(1500));
    }

    info!("Scrolls is stopping...");
    runtime.shutdown();

    Ok(())
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, value_parser)]
    //#[clap(description = "config file to load by the daemon")]
    config: Option<std::path::PathBuf>,

    #[clap(long, value_parser)]
    //#[clap(description = "type of progress to display")],
    console: Option<console::Mode>,
}
