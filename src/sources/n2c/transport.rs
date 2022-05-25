use std::os::unix::net::UnixStream;

use pallas::network::{
    miniprotocols::{self, handshake},
    multiplexer::Multiplexer,
};

pub struct Transport {
    pub muxer: Multiplexer,
    pub version: handshake::VersionNumber,
}

impl Transport {
    fn connect_muxer(address: &str) -> Result<Multiplexer, miniprotocols::Error> {
        log::debug!("connecting muxer");
        let unix = UnixStream::connect(address)?;
        let muxer = Multiplexer::setup(unix, &[0, 5])?;

        Ok(muxer)
    }

    fn do_handshake(
        muxer: &mut Multiplexer,
        magic: u64,
    ) -> Result<handshake::VersionNumber, miniprotocols::Error> {
        log::debug!("doing handshake");

        let mut channel = muxer.use_channel(0);
        let versions = handshake::n2c::VersionTable::v1_and_above(magic);
        let agent =
            miniprotocols::run_agent(handshake::Initiator::initial(versions), &mut channel)?;
        log::info!("handshake output: {:?}", agent.output);

        match agent.output {
            handshake::Output::Accepted(version, _) => Ok(version),
            _ => Err("couldn't agree on handshake version".into()),
        }
    }

    pub fn setup(address: &str, magic: u64) -> Result<Self, miniprotocols::Error> {
        let mut muxer = Self::connect_muxer(address)?;
        let version = Self::do_handshake(&mut muxer, magic)?;

        Ok(Self { muxer, version })
    }
}
