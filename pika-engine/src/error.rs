use thiserror::Error;
use pika_core::error::PikaError;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Core error: {0}")]
    Core(#[from] PikaError),
    
    #[error("Engine error: {0}")]
    Engine(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>; 