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
    mut source: sources::Bootstrapper,
    mut reducer: reducers::Bootstrapper,
    mut storage: storage::Bootstrapper,
) -> Result<Pipeline, crate::Error> {
    let mut pipeline = Pipeline::new();

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

    let reader = storage.build_read_plugin();

    source.spawn(&mut pipeline, &reader);
    reducer.spawn_stages(&mut pipeline, reader);
    storage.spawn_stages(&mut pipeline);

    Ok(pipeline)
}
