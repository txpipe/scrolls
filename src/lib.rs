pub mod bootstrap;
pub mod collections;
pub mod crosscut;
pub mod model;
pub mod sources;
pub mod storage;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("network transport error: {0}")]
    TransportError(String),

    #[error("{0}")]
    Message(String),
}

impl Error {
    pub fn message(text: impl Into<String>) -> Error {
        Error::Message(text.into())
    }
}
