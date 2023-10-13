use crate::sources;

use gasket::{messaging::connect_ports, runtime::Tether};

pub struct Pipeline {
    pub tethers: Vec<Tether>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            tethers: Vec::new(),
        }
    }

    pub fn register_stage(&mut self, tether: Tether) {
        self.tethers.push(tether);
    }
}

pub fn build(
    mut source: sources::Bootstrapper,
    // mut enrich: enrich::Bootstrapper,
    // mut reducer: reducers::Bootstrapper,
    // mut storage: storage::Bootstrapper,
) -> Result<Pipeline, crate::Error> {
    // let cursor = storage.build_cursor();

    let mut pipeline = Pipeline::new();

    // connect_ports(source.borrow_output_port(), enrich.borrow_input_port(), 100);

    // connect_ports(
    //     enrich.borrow_output_port(),
    //     reducer.borrow_input_port(),
    //     100,
    // );

    // connect_ports(
    //     reducer.borrow_output_port(),
    //     storage.borrow_input_port(),
    //     100,
    // );

    source.spawn_stages(&mut pipeline, cursor);
    // enrich.spawn_stages(&mut pipeline);
    // reducer.spawn_stages(&mut pipeline);
    // storage.spawn_stages(&mut pipeline);

    Ok(pipeline)
}
