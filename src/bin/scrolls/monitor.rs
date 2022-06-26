use std::time::Duration;

use scrolls::bootstrap::Pipeline;

fn should_stop(pipeline: &Pipeline) -> bool {
    pipeline
        .tethers
        .iter()
        .any(|(_, tether)| match tether.check_state() {
            gasket::runtime::TetherState::Alive(x) => match x {
                gasket::runtime::StageState::StandBy => true,
                _ => false,
            },
            _ => true,
        })
}

pub fn print_metrics(pipeline: &Pipeline) {
    for (name, tether) in pipeline.tethers.iter() {
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
}

pub fn monitor_while_alive(pipeline: Pipeline) {
    while !should_stop(&pipeline) {
        print_metrics(&pipeline);
        std::thread::sleep(Duration::from_secs(5));
    }

    for (name, tether) in pipeline.tethers {
        let state = tether.check_state();
        log::warn!("dismissing stage: {} with state {:?}", name, state);
        tether.dismiss_stage().expect("stage stops");

        // Can't join the stage because there's a risk of deadlock, usually
        // because a stage gets stuck sending into a port which depends on a
        // different stage not yet dismissed. The solution is to either create a
        // DAG of dependencies and dismiss in the correct order, or implement a
        // 2-phase teardown where ports are disconnected and flushed
        // before joining the stage.

        //tether.join_stage();
    }
}
