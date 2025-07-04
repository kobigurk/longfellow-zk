use thiserror::Error;

#[derive(Error, Debug)]
pub enum LongfellowError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),
    
    #[error("Proof verification failed: {0}")]
    VerificationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Circuit error: {0}")]
    CircuitError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, LongfellowError>;