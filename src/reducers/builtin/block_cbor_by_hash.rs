use pallas::codec::minicbor::Encode;
use pallas::codec::utils::CborWrap;
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::babbage::{PlutusData, PseudoDatumOption};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{
    MultiEraBlock, Era, MultiEraOutput, MultiEraPolicyAssets, MultiEraTx, OriginalHash,
};
use serde::Deserialize;
use serde_json::json;

use crate::crosscut::filters::BlockPattern;
use crate::framework::model::CRDTCommand;
use crate::framework::{model, Error};

use super::{ReducerConfigTrait, ReducerTrait};

#[derive(Deserialize)]
pub struct Config {
    pub filter: Vec<String>,
    pub prefix: Option<String>,
    pub address_as_key: Option<bool>,
}
impl ReducerConfigTrait for Config {
    fn plugin(self) -> Box<dyn ReducerTrait> {
        let reducer = Reducer { config: self };
        Box::new(reducer)
    }
}

pub struct Reducer {
    config: Config,
}

#[async_trait::async_trait]
impl ReducerTrait for Reducer {
    async fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        _ctx: &model::BlockContext,
    ) -> Result<Vec<CRDTCommand>, Error> {
        let prefix = self.config.prefix.as_deref();
        let mut commands: Vec<CRDTCommand> = Vec::new();
        let era = block.era();
        let value: Vec<u8> = match era {
            Era::Byron => {
                block.as_byron().unwrap().encode_fragment().unwrap()
            }
            Era::Alonzo => {
                block.as_alonzo().unwrap().encode_fragment().unwrap()
            }
            Era::Babbage => {
                block.as_babbage().unwrap().encode_fragment().unwrap()
            }
            _ => {
                return Err(Error::CborError("Unsupported era".to_string()));
            }
        };
        let mut s = String::new();
        for x in &value {
          s.push_str(&format!("{:02x}", x));
        }
        println!("cbor: {}", s);
        let crdt = model::CRDTCommand::any_write_wins(
            prefix,
            block.hash(),
            value
        );
        commands.push(crdt);
        Ok(commands)
    }
}
