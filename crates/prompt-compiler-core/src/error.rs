use thiserror::Error;

pub type Result<T> = std::result::Result<T, PromptCompilerError>;

#[derive(Error, Debug)]
pub enum PromptCompilerError {
    #[error("Compilation error: {0}")]
    CompilationError(String),
    
    #[error("Weight update computation error: {0}")]
    WeightUpdateError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Numerical computation error: {0}")]
    NumericalError(String),
    
    #[error("Prompt not found: {hash}")]
    PromptNotFound { hash: String },
    
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    
    #[error("Time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
    
    #[error("Model incompatibility: {0}")]
    ModelIncompatibility(String),
}
