use crate::{reducers, sources, storage};

use gasket::{messaging::connect_ports, runtime::Tether};

type NamedTether = (&'static str, Tether);

pub struct Pipeline {
    pub tethers: Vec<NamedTether>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            tethers: Vec::new(),
        }
    }

    pub fn register_stage(&mut self, name: &'static str, tether: Tether) {
        self.tethers.push((name, tether));
    }
}

pub fn build(
    mut source: sources::Plugin,
    mut reducers: Vec<reducers::Plugin>,
    mut storage: storage::Plugin,
) -> Pipeline {
    let mut pipeline = Pipeline::new();

    for reducer in reducers.iter_mut() {
        connect_ports(
            source.borrow_output_port(),
            reducer.borrow_input_port(),
            100,
        );

        connect_ports(
            reducer.borrow_output_port(),
            storage.borrow_input_port(),
            100,
        );
    }

    source.spawn(&mut pipeline);
    reducers.into_iter().for_each(|r| r.spawn(&mut pipeline));
    storage.spawn(&mut pipeline);

    pipeline
}
