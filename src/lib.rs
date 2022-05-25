pub mod bootstrap;
pub mod crosscut;
pub mod model;
pub mod reducers;
pub mod sources;
pub mod storage;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("network transport error: {0}")]
    TransportError(String),

    #[error("ouroboros error: {0}")]
    OuroborosError(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[error("{0}")]
    Message(String),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    pub fn config(text: impl Into<String>) -> Error {
        Error::ConfigError(text.into())
    }

    pub fn message(text: impl Into<String>) -> Error {
        Error::Message(text.into())
    }

    pub fn ouroboros(error: Box<dyn std::error::Error>) -> Error {
        Error::OuroborosError(format!("{}", error))
    }

    pub fn custom(error: Box<dyn std::error::Error>) -> Error {
        Error::Custom(format!("{}", error))
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::custom(err)
    }
}
