use pallas::network::{
    miniprotocols::{chainsync::TipFinder, run_agent, Point},
    multiplexer::StdChannelBuffer,
};

use crate::{crosscut, storage};

pub fn find_end_of_chain(
    chain: &crosscut::ChainWellKnownInfo,
    channel: &mut StdChannelBuffer,
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
    storage: &mut storage::ReadPlugin,
    channel: &mut StdChannelBuffer,
) -> Result<Option<Vec<Point>>, crate::Error> {
    // if we have a cursor available, it should override any other configuration
    // opiton
    match &storage.read_cursor()? {
        Some(x) => {
            log::info!("found existing cursor in storage plugin: {:?}", x);
            return Ok(Some(vec![x.clone().try_into()?]));
        }
        None => log::debug!("no cursor found in storage plugin"),
    };

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
