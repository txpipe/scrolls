use pallas::{
    ledger::primitives::{alonzo, byron, probing, Era, Fragment},
    network::{
        miniprotocols::{chainsync::TipFinder, run_agent, Point},
        multiplexer::Channel,
    },
};

use crate::{
    crosscut::{self, ChainWellKnownInfo, IntersectConfig},
    model::MultiEraBlock,
    Error,
};

pub fn parse_block_content(body: &[u8]) -> Result<MultiEraBlock, Error> {
    match probing::probe_block_cbor_era(&body) {
        probing::Outcome::Matched(era) => match era {
            Era::Byron => {
                let primitive = byron::Block::decode_fragment(&body)?;
                let block = MultiEraBlock::Byron(primitive);
                Ok(block)
            }
            _ => {
                let primitive = alonzo::BlockWrapper::decode_fragment(&body)?;
                let block = MultiEraBlock::AlonzoCompatible(primitive);
                Ok(block)
            }
        },
        // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
        // assumption?
        probing::Outcome::GenesisBlock => {
            let primitive = byron::Block::decode_fragment(&body)?;
            let block = MultiEraBlock::Byron(primitive);
            Ok(block)
        }
        probing::Outcome::Inconclusive => {
            let msg = format!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
            return Err(Error::Message(msg));
        }
    }
}

pub fn find_end_of_chain(
    chain: &ChainWellKnownInfo,
    channel: &mut Channel,
) -> Result<Point, crate::Error> {
    let point = Point::Specific(
        chain.shelley_known_slot,
        hex::decode(&chain.shelley_known_hash)
            .map_err(|_| crate::Error::config("can't decode shelley known hash"))?,
    );

    let agent = TipFinder::initial(point);
    let agent = run_agent(agent, channel).map_err(crate::Error::ouroboros)?;

    match agent.output {
        Some(tip) => Ok(tip.0),
        None => Err(crate::Error::message("failure acquiring end of chain")),
    }
}

pub fn define_known_points(
    chain: &ChainWellKnownInfo,
    intersect: &IntersectConfig,
    channel: &mut Channel,
) -> Result<Option<Vec<Point>>, crate::Error> {
    match &intersect {
        crosscut::IntersectConfig::Origin => Ok(None),
        crosscut::IntersectConfig::Tip => {
            let tip = find_end_of_chain(chain, channel)?;
            Ok(Some(vec![tip]))
        }
        crosscut::IntersectConfig::Point(x) => {
            let point = x.clone().try_into()?;
            Ok(Some(vec![point]))
        }
        crosscut::IntersectConfig::Fallbacks(x) => {
            let points: Result<Vec<_>, _> = x.iter().cloned().map(|x| x.try_into()).collect();
            Ok(Some(points?))
        }
    }
}
