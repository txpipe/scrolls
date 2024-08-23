use pallas_miniprotocols::{
    chainsync::{self, N2CClient, N2NClient},
    Point,
};
use pallas_multiplexer::StdChannel;
use std::convert::TryInto;

use crate::{crosscut, storage};

macro_rules! define_chainsync_start {
    ($fn:ident, $client:ident) => {
        pub fn $fn(
            intersect: &crosscut::IntersectConfig,
            cursor: &mut storage::Cursor,
            client: &mut $client<StdChannel>,
        ) -> Result<Option<Point>, crate::Error> {
            match cursor.last_point()? {
                Some(x) => {
                    log::info!("found existing cursor in storage plugin: {:?}", x);
                    let point = x.try_into()?;
                    let (point, _) = client
                        .find_intersect(vec![point])
                        .map_err(crate::Error::ouroboros)?;
                    return Ok(point);
                }
                None => log::info!("no cursor found in storage plugin"),
            };

            match &intersect {
                crosscut::IntersectConfig::Origin => {
                    let point = client.intersect_origin().map_err(crate::Error::ouroboros)?;
                    Ok(Some(point))
                }
                crosscut::IntersectConfig::Tip => {
                    let point = client.intersect_tip().map_err(crate::Error::ouroboros)?;
                    Ok(Some(point))
                }
                crosscut::IntersectConfig::Point(_, _) => {
                    let point = intersect.get_point().expect("point value");
                    let (point, _) = client
                        .find_intersect(vec![point])
                        .map_err(crate::Error::ouroboros)?;
                    Ok(point)
                }
                crosscut::IntersectConfig::Fallbacks(_) => {
                    let points = intersect.get_fallbacks().expect("fallback values");
                    let (point, _) = client
                        .find_intersect(points)
                        .map_err(crate::Error::ouroboros)?;
                    Ok(point)
                }
            }
        }
    };
}

define_chainsync_start!(define_chainsync_start_n2c, N2CClient);
define_chainsync_start!(define_chainsync_start_n2n, N2NClient);
