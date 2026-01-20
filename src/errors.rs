use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KirokuError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Git error: {0}")]
    Git(String),
    #[error("Environment error: {0}")]
    Env(String),
}
