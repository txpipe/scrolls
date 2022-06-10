use pallas::network::{
    miniprotocols::{self, handshake},
    multiplexer,
};

pub struct Transport {
    pub channel5: multiplexer::StdChannelBuffer,
    pub version: handshake::VersionNumber,
}

impl Transport {
    fn do_handshake(
        channel: &mut multiplexer::StdChannelBuffer,
        magic: u64,
    ) -> Result<handshake::VersionNumber, crate::Error> {
        log::debug!("doing handshake");

        let versions = handshake::n2c::VersionTable::v1_and_above(magic);
        let agent = miniprotocols::run_agent(handshake::Initiator::initial(versions), channel)
            .map_err(crate::Error::ouroboros)?;

        log::info!("handshake output: {:?}", agent.output);

        match agent.output {
            handshake::Output::Accepted(version, _) => Ok(version),
            _ => Err(crate::Error::ouroboros(
                "couldn't agree on handshake version",
            )),
        }
    }

    pub fn setup(address: &str, magic: u64) -> Result<Self, crate::Error> {
        log::debug!("connecting muxer");

        let bearer =
            multiplexer::bearers::Bearer::connect_unix(address).map_err(crate::Error::network)?;
        let mut plexer = multiplexer::StdPlexer::new(bearer);

        let mut channel0 = plexer.use_channel(0).into();
        let channel5 = plexer.use_channel(5).into();

        plexer.muxer.spawn();
        plexer.demuxer.spawn();

        let version = Self::do_handshake(&mut channel0, magic)?;

        Ok(Self { channel5, version })
    }
}
