use std::net::TcpStream;

use pallas::network::{
    miniprotocols::{self, handshake},
    multiplexer::Multiplexer,
};

pub struct Config {
    pub address: String,
    pub magic: u64,
    pub protocols: &'static [u16],
}

pub struct Transport {
    pub muxer: Multiplexer,
    pub version: handshake::VersionNumber,
}

impl Transport {
    fn connect_muxer(config: &Config) -> Result<Multiplexer, miniprotocols::Error> {
        log::debug!("connecting muxer");

        let tcp = TcpStream::connect(&config.address)?;
        tcp.set_nodelay(true)?;
        //tcp.set_keepalive_ms(Some(30_000u32))?;

        let muxer = Multiplexer::setup(tcp, config.protocols)?;

        Ok(muxer)
    }

    fn do_handshake(
        config: &Config,
        muxer: &mut Multiplexer,
    ) -> Result<handshake::VersionNumber, miniprotocols::Error> {
        log::debug!("doing handshake");

        let mut channel = muxer.use_channel(0);
        let versions = handshake::n2n::VersionTable::v6_and_above(config.magic);
        let agent =
            miniprotocols::run_agent(handshake::Initiator::initial(versions), &mut channel)?;
        log::info!("handshake output: {:?}", agent.output);

        match agent.output {
            handshake::Output::Accepted(version, _) => Ok(version),
            _ => Err("couldn't agree on handshake version".into()),
        }
    }

    pub fn setup(config: &Config) -> Result<Self, miniprotocols::Error> {
        let mut muxer = Self::connect_muxer(config)?;
        let version = Self::do_handshake(config, &mut muxer)?;

        Ok(Self { muxer, version })
    }
}
