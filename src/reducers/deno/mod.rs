use std::path::PathBuf;

use deno_runtime::deno_core::{self, op2, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker as DenoWorker, WorkerOptions};
use gasket::framework::*;
use pallas::interop::utxorpc::{map_block, map_tx_output};
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use serde::Deserialize;
use serde_json::json;
use tracing::trace;
use utxorpc::proto::cardano::v1 as u5c;

use crate::framework::model::CRDTCommand;
use crate::framework::*;

const SYNC_CALL_SNIPPET: &str = r#"Deno[Deno.internal].core.ops.op_put_record(reduce(Deno[Deno.internal].core.ops.op_pop_record()));"#;

const ASYNC_CALL_SNIPPET: &str = r#"reduce(Deno[Deno.internal].core.ops.op_pop_record()).then(x => Deno[Deno.internal].core.ops.op_put_record(x));"#;

deno_core::extension!(deno_reducer, ops = [op_pop_record, op_put_record]);

#[op2]
#[serde]
pub fn op_pop_record(state: &mut OpState) -> Result<serde_json::Value, deno_core::error::AnyError> {
    let block: u5c::Block = state.take();
    Ok(json!(block))
}

#[op2]
pub fn op_put_record(
    state: &mut OpState,
    #[serde] value: serde_json::Value,
) -> Result<(), deno_core::error::AnyError> {
    match value {
        serde_json::Value::Null => (),
        _ => state.put(value),
    };

    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    main_module: String,
    use_async: bool,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            main_module: PathBuf::from(self.main_module),
            call_snippet: if self.use_async {
                ASYNC_CALL_SNIPPET
            } else {
                SYNC_CALL_SNIPPET
            },
            ..Default::default()
        };

        Ok(stage)
    }
}

async fn setup_deno(main_module: &PathBuf) -> Result<DenoWorker, WorkerError> {
    let empty_module = deno_core::ModuleSpecifier::parse("data:text/javascript;base64,").unwrap();

    let mut deno = DenoWorker::bootstrap_from_options(
        empty_module,
        PermissionsContainer::allow_all(),
        WorkerOptions {
            extensions: vec![deno_reducer::init_ops()],
            ..Default::default()
        },
    );

    let code = deno_core::FastString::from(std::fs::read_to_string(main_module).unwrap());

    deno.js_runtime
        .load_side_module(
            &ModuleSpecifier::parse("scrolls:reducer").unwrap(),
            Some(code),
        )
        .await
        .unwrap();

    let runtime_code = deno_core::FastString::from_static(include_str!("./runtime.js"));

    deno.execute_script("[scrolls:runtime.js]", runtime_code)
        .or_panic()?;
    deno.run_event_loop(false).await.unwrap();

    Ok(deno)
}

#[derive(Default, Stage)]
#[stage(name = "reducer-deno", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    main_module: PathBuf,
    call_snippet: &'static str,

    pub input: ReducerInputPort,
    pub output: ReducerOutputPort,
    #[metric]
    ops_count: gasket::metrics::Counter,
}

pub struct Worker {
    runtime: DenoWorker,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let runtime = setup_deno(&stage.main_module).await?;
        Ok(Self { runtime })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let record = unit.record();
        if record.is_none() {
            return Ok(());
        }

        let record = record.unwrap();

        match record {
            Record::EnrichedBlockPayload(block, ctx) => {
                let block = MultiEraBlock::decode(block)
                    .map_err(Error::cbor)
                    .or_panic()?;
                let mut block = map_block(&block);
                
                for tx in block.body.as_mut().unwrap().tx.iter_mut() {
                    for input in tx.inputs.iter_mut() {
                        if input.tx_hash.len() == 32 {
                            let mut hash_bytes = [0u8; 32];
                            hash_bytes.copy_from_slice(&input.tx_hash);
        
                            let tx_hash = pallas::crypto::hash::Hash::from(hash_bytes);
                            let output_index = input.output_index as u64;
        
                            let output_ref = OutputRef::new(tx_hash, output_index);
                            if let Ok(output) = ctx.find_utxo(&output_ref) {
                                input.as_output = Some(map_tx_output(&output));
                            }
                        }
                    }
                }

                let deno = &mut self.runtime;

                trace!(?record, "sending record to js runtime");
                deno.js_runtime.op_state().borrow_mut().put(block);

                let script = deno_core::FastString::from_static(stage.call_snippet);
                deno.execute_script("<anon>", script).or_panic()?;
                deno.run_event_loop(false).await.unwrap();

                let out: Option<serde_json::Value> =
                    deno.js_runtime.op_state().borrow_mut().try_take();

                trace!(?out, "received record from js runtime");
                if let Some(crdt_json) = out {
                    let commands: Vec<CRDTCommand> =
                        serde_json::from_value(crdt_json).or_panic()?;
                    let evt =
                        ChainEvent::apply(unit.point().clone(), Record::CRDTCommand(commands));
                    stage.output.send(evt).await.or_retry()?;
                }
            }
            _ => todo!(),
        };

        Ok(())
    }
}
