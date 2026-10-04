use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("user action required: {0}")]
    UserAction(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("partial metadata: {0}")]
    PartialMeta(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
