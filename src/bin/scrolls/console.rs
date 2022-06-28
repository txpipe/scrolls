use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use gasket::metrics::Reading;
use lazy_static::{__Deref, lazy_static};
use log::Log;
use scrolls::bootstrap::Pipeline;

#[derive(clap::ValueEnum, Clone)]
pub enum Mode {
    /// shows progress as a plain sequence of logs
    Plain,
    /// shows aggregated progress and metrics
    TUI,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Plain
    }
}

struct TuiConsole {
    chainsync_progress: indicatif::ProgressBar,
    received_blocks: indicatif::ProgressBar,
    reducer_ops_count: indicatif::ProgressBar,
    storage_ops_count: indicatif::ProgressBar,
    enrich_inserts: indicatif::ProgressBar,
    enrich_matches: indicatif::ProgressBar,
    enrich_mismatches: indicatif::ProgressBar,
    enrich_blocks: indicatif::ProgressBar,
}

impl TuiConsole {
    fn build_counter_spinner(
        name: &str,
        container: &indicatif::MultiProgress,
    ) -> indicatif::ProgressBar {
        container.add(
            indicatif::ProgressBar::new_spinner().with_style(
                indicatif::ProgressStyle::default_spinner()
                    .template(&format!(
                        "{{spinner}} {:<20} {{msg:<20}} {{pos:>8}} | {{per_sec}}",
                        name
                    ))
                    .unwrap(),
            ),
        )
    }

    fn new() -> Self {
        let container = indicatif::MultiProgress::new();

        Self {
            chainsync_progress: container.add(
                indicatif::ProgressBar::new(0).with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("chainsync progress: {bar} {pos}/{len} eta: {eta}\n{msg}")
                        .unwrap(),
                ),
            ),
            received_blocks: Self::build_counter_spinner("received blocks", &container),
            enrich_inserts: Self::build_counter_spinner("enrich inserts", &container),
            enrich_matches: Self::build_counter_spinner("enrich matches", &container),
            enrich_mismatches: Self::build_counter_spinner("enrich mismatches", &container),
            enrich_blocks: Self::build_counter_spinner("enrich blocks", &container),
            reducer_ops_count: Self::build_counter_spinner("reducer ops", &container),
            storage_ops_count: Self::build_counter_spinner("storage ops", &container),
        }
    }

    fn refresh(&self, pipeline: &Pipeline) {
        for (stage, tether) in pipeline.tethers.iter() {
            let state = match tether.check_state() {
                gasket::runtime::TetherState::Dropped => "dropped!",
                gasket::runtime::TetherState::Blocked(_) => "blocked!",
                gasket::runtime::TetherState::Alive(x) => match x {
                    gasket::runtime::StageState::Bootstrap => "bootstrapping...",
                    gasket::runtime::StageState::Working => "working...",
                    gasket::runtime::StageState::Idle => "idle...",
                    gasket::runtime::StageState::StandBy => "stand-by...",
                    gasket::runtime::StageState::Teardown => "tearing down...",
                },
            };

            match tether.read_metrics() {
                Ok(readings) => {
                    for (key, value) in readings {
                        match (*stage, key, value) {
                            (_, "chain_tip", Reading::Gauge(x)) => {
                                self.chainsync_progress.set_length(x as u64);
                            }
                            (_, "last_block", Reading::Gauge(x)) => {
                                self.chainsync_progress.set_position(x as u64);
                            }
                            (_, "received_blocks", Reading::Count(x)) => {
                                self.received_blocks.set_position(x);
                                self.received_blocks.set_message(state);
                            }
                            ("reducers", "ops_count", Reading::Count(x)) => {
                                self.reducer_ops_count.set_position(x);
                                self.reducer_ops_count.set_message(state);
                            }
                            (_, "storage_ops", Reading::Count(x)) => {
                                self.storage_ops_count.set_position(x);
                                self.storage_ops_count.set_message(state);
                            }
                            (_, "enrich_inserts", Reading::Count(x)) => {
                                self.enrich_inserts.set_position(x);
                                self.enrich_inserts.set_message(state);
                            }
                            (_, "enrich_matches", Reading::Count(x)) => {
                                self.enrich_matches.set_position(x);
                                self.enrich_matches.set_message(state);
                            }
                            (_, "enrich_mismatches", Reading::Count(x)) => {
                                self.enrich_mismatches.set_position(x);
                                self.enrich_mismatches.set_message(state);
                            }
                            (_, "enrich_blocks", Reading::Count(x)) => {
                                self.enrich_blocks.set_position(x);
                                self.enrich_blocks.set_message(state);
                            }
                            _ => (),
                        }
                    }
                }
                Err(err) => {
                    println!("couldn't read metrics");
                    dbg!(err);
                }
            }
        }
    }
}

impl Log for TuiConsole {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() >= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        self.chainsync_progress
            .set_message(format!("{}", record.args()))
    }

    fn flush(&self) {}
}

struct PlainConsole {
    last_report: Mutex<Instant>,
}

impl PlainConsole {
    fn new() -> Self {
        Self {
            last_report: Mutex::new(Instant::now()),
        }
    }

    fn refresh(&self, pipeline: &Pipeline) {
        let mut last_report = self.last_report.lock().unwrap();

        if last_report.elapsed() <= Duration::from_secs(10) {
            return;
        }

        for (stage, tether) in pipeline.tethers.iter() {
            match tether.check_state() {
                gasket::runtime::TetherState::Dropped => {
                    log::error!("[{}] stage tether has been dropped", stage);
                }
                gasket::runtime::TetherState::Blocked(_) => {
                    log::warn!("[{}] stage tehter is blocked or not reporting state", stage);
                }
                gasket::runtime::TetherState::Alive(state) => {
                    log::debug!("[{}] stage is alive with state: {:?}", stage, state);
                    match tether.read_metrics() {
                        Ok(readings) => {
                            for (key, value) in readings {
                                log::debug!("[{}] metric `{}` = {:?}", stage, key, value);
                            }
                        }
                        Err(err) => {
                            log::error!("[{}] error reading metrics: {}", stage, err)
                        }
                    }
                }
            }
        }

        *last_report = Instant::now();
    }
}

lazy_static! {
    static ref TUI_CONSOLE: TuiConsole = TuiConsole::new();
}

lazy_static! {
    static ref PLAIN_CONSOLE: PlainConsole = PlainConsole::new();
}

pub fn initialize(mode: &Option<Mode>) {
    match mode {
        Some(Mode::TUI) => log::set_logger(TUI_CONSOLE.deref())
            .map(|_| log::set_max_level(log::LevelFilter::Info))
            .unwrap(),
        _ => env_logger::init(),
    }
}

pub fn refresh(mode: &Option<Mode>, pipeline: &Pipeline) {
    match mode {
        Some(Mode::TUI) => TUI_CONSOLE.refresh(pipeline),
        _ => PLAIN_CONSOLE.refresh(pipeline),
    }
}
