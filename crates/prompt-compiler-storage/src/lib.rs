//! Storage module - RocksDB persistence and DAG management

pub mod state_db;
pub mod dag;

pub use state_db::*;
pub use dag::*;

/// Storage errors
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rocksdb::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Crypto error: {0}")]
    CryptoError(#[from] prompt_compiler_crypto::CryptoError),
    
    #[error("Entry not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
