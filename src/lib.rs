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
}
