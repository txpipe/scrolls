pub mod bootstrap;
pub mod crosscut;
pub mod enrich;
pub mod model;
pub mod prelude;
pub mod reducers;
pub mod sources;
pub mod storage;

use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("ouroboros error: {0}")]
    OuroborosError(String),

    #[error("cbor error: {0}")]
    CborError(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[error("missing utxo: {0}")]
    MissingUtxo(String),

    #[error("source error: {0}")]
    SourceError(String),

    #[error("ledger error: {0}")]
    StorageError(String),

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

    pub fn network(error: impl Display) -> Error {
        Error::NetworkError(error.to_string())
    }

    pub fn cbor(error: impl Display) -> Error {
        Error::CborError(error.to_string())
    }

    pub fn ouroboros(error: impl Display) -> Error {
        Error::OuroborosError(error.to_string())
    }

    pub fn ledger(error: impl Display) -> Error {
        Error::LedgerError(error.to_string())
    }

    pub fn missing_utxo(utxo_key: impl Display) -> Error {
        Error::MissingUtxo(utxo_key.to_string())
    }

    pub fn source(error: impl Display) -> Error {
        Error::SourceError(error.to_string())
    }

    pub fn storage(error: impl Display) -> Error {
        Error::StorageError(error.to_string())
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
