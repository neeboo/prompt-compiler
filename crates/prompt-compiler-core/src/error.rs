//! Error types for the prompt compiler

use std::fmt;

pub type Result<T> = std::result::Result<T, PromptCompilerError>;

#[derive(Debug)]
pub enum PromptCompilerError {
    InvalidPrompt(String),
    CompilationError(String),
    OptimizationError(String),
    GenerationError(String),
    StorageError(String),
    NetworkError(String),
    SerializationError(serde_json::Error),
    ConfigError(String),
    IoError(std::io::Error),
    NumericalError(String),
    PromptNotFound { hash: String },
    InvalidHash(String),
    TimeError(std::time::SystemTimeError),
    ModelIncompatibility(String),
    WeightUpdateError(String),
}

impl fmt::Display for PromptCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrompt(msg) => write!(f, "Invalid prompt: {}", msg),
            Self::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            Self::OptimizationError(msg) => write!(f, "Optimization error: {}", msg),
            Self::GenerationError(msg) => write!(f, "Generation error: {}", msg),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::SerializationError(err) => write!(f, "Serialization error: {}", err),
            Self::ConfigError(err) => write!(f, "Configuration error: {}", err),
            Self::IoError(err) => write!(f, "IO error: {}", err),
            Self::NumericalError(err) => write!(f, "Numerical computation error: {}", err),
            Self::PromptNotFound { hash } => write!(f, "Prompt not found: {}", hash),
            Self::InvalidHash(err) => write!(f, "Invalid hash: {}", err),
            Self::TimeError(err) => write!(f, "Time error: {}", err),
            Self::ModelIncompatibility(err) => write!(f, "Model incompatibility: {}", err),
            Self::WeightUpdateError(err) => write!(f, "Weight update computation error: {}", err),
        }
    }
}

impl std::error::Error for PromptCompilerError {}

// Error conversions
impl From<serde_json::Error> for PromptCompilerError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err)
    }
}

impl From<std::io::Error> for PromptCompilerError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::time::SystemTimeError> for PromptCompilerError {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::TimeError(err)
    }
}

impl From<prompt_compiler_weights::WeightError> for PromptCompilerError {
    fn from(err: prompt_compiler_weights::WeightError) -> Self {
        Self::WeightUpdateError(err.to_string())
    }
}
