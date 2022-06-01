use pallas::{
    ledger::primitives::{alonzo, byron, probing, Era, Fragment},
    network::{
        miniprotocols::{chainsync::TipFinder, run_agent, Point},
        multiplexer::Channel,
    },
};

use crate::{crosscut, model, Error};

pub fn parse_block_content(body: &[u8]) -> Result<model::MultiEraBlock, Error> {
    match probing::probe_block_cbor_era(&body) {
        probing::Outcome::Matched(era) => match era {
            Era::Byron => {
                let primitive = byron::Block::decode_fragment(&body)?;
                let block = model::MultiEraBlock::Byron(primitive);
                Ok(block)
            }
            _ => {
                let primitive = alonzo::BlockWrapper::decode_fragment(&body)?;
                let block = model::MultiEraBlock::AlonzoCompatible(primitive);
                Ok(block)
            }
        },
        // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
        // assumption?
        probing::Outcome::GenesisBlock => {
            let primitive = byron::Block::decode_fragment(&body)?;
            let block = model::MultiEraBlock::Byron(primitive);
            Ok(block)
        }
        probing::Outcome::Inconclusive => {
            let msg = format!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
            return Err(Error::Message(msg));
        }
    }
}

pub fn find_end_of_chain(
    chain: &crosscut::ChainWellKnownInfo,
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
    chain: &crosscut::ChainWellKnownInfo,
    intersect: &crosscut::IntersectConfig,
    cursor: &crosscut::Cursor,
    channel: &mut Channel,
) -> Result<Option<Vec<Point>>, crate::Error> {
    // if we have a cursor available, it should override any other configuration
    // opiton
    if let Some(point) = cursor {
        return Ok(Some(vec![point.clone().try_into()?]));
    }

    match &intersect {
        crosscut::IntersectConfig::Origin => Ok(None),
        crosscut::IntersectConfig::Tip => {
            let tip = find_end_of_chain(chain, channel)?;
            Ok(Some(vec![tip]))
        }
        crosscut::IntersectConfig::Point(_, _) => {
            let point = intersect.get_point().expect("point value");
            Ok(Some(vec![point]))
        }
        crosscut::IntersectConfig::Fallbacks(_) => {
            let points = intersect.get_fallbacks().expect("fallback values");
            Ok(Some(points))
        }
    }
}
