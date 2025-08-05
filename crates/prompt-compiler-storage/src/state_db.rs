//! State database - RocksDB storage for compiled states

use crate::{Result, StorageError};
use prompt_compiler_crypto::{Hash, Signature};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compiled state for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredState {
    pub version: String,
    pub content: Vec<u8>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
}

/// Signed compiled state for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedStoredState {
    pub state: StoredState,
    pub hash: Hash,
    pub signature: Option<Signature>,
}

/// Compilation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStats {
    pub total_compilations: u64,
    pub avg_compilation_time_ms: f64,
    pub avg_weight_updates_per_prompt: f32,
    pub most_common_targets: Vec<String>,
    pub convergence_rate: f32,
}

/// RocksDB state database
pub struct StateDB {
    db: DB,
    _cf_handles: HashMap<String, rocksdb::ColumnFamily>,
}

impl StateDB {
    /// Create new state database
    pub fn new(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new("states", Options::default()),
            ColumnFamilyDescriptor::new("versions", Options::default()),
            ColumnFamilyDescriptor::new("signatures", Options::default()),
            ColumnFamilyDescriptor::new("metadata", Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;

        Ok(Self {
            db,
            _cf_handles: HashMap::new(),
        })
    }

    /// Store compiled state
    pub fn store_state(&self, hash: &str, state: &StoredState) -> Result<()> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        let serialized = serde_json::to_vec(state)?;
        self.db.put_cf(cf, hash.as_bytes(), serialized)?;

        // Store time index
        let timestamp_key = format!("time:{}", state.created_at);
        self.db
            .put_cf(cf, timestamp_key.as_bytes(), hash.as_bytes())?;

        Ok(())
    }

    /// Get compiled state
    pub fn get_state(&self, hash: &str) -> Result<Option<StoredState>> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        if let Some(data) = self.db.get_cf(cf, hash.as_bytes())? {
            let state: StoredState = serde_json::from_slice(&data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    /// Store signed state
    pub fn store_signed_state(&self, hash: &str, signed_state: &SignedStoredState) -> Result<()> {
        let cf = self.db.cf_handle("signatures").ok_or_else(|| {
            StorageError::InvalidData("signatures column family not found".to_string())
        })?;

        let serialized = serde_json::to_vec(signed_state)?;
        self.db.put_cf(cf, hash.as_bytes(), serialized)?;

        Ok(())
    }

    /// Get signed state
    pub fn get_signed_state(&self, hash: &str) -> Result<Option<SignedStoredState>> {
        let cf = self.db.cf_handle("signatures").ok_or_else(|| {
            StorageError::InvalidData("signatures column family not found".to_string())
        })?;

        if let Some(data) = self.db.get_cf(cf, hash.as_bytes())? {
            let signed_state: SignedStoredState = serde_json::from_slice(&data)?;
            Ok(Some(signed_state))
        } else {
            Ok(None)
        }
    }

    /// Query states by time range
    pub fn get_states_by_timerange(&self, start: u64, end: u64) -> Result<Vec<StoredState>> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        let mut states = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);

            if key_str.starts_with("time:") {
                let timestamp: u64 = key_str[5..].parse().unwrap_or(0);
                if timestamp >= start && timestamp <= end {
                    let hash = String::from_utf8_lossy(&value);
                    if let Some(state) = self.get_state(&hash)? {
                        states.push(state);
                    }
                }
            }
        }

        states.sort_by_key(|s| s.created_at);
        Ok(states)
    }

    /// Get compilation statistics
    pub fn get_compilation_stats(&self) -> Result<CompilationStats> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        let mut total_compilations = 0u64;
        let target_models: HashMap<String, u32> = HashMap::new();

        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key);

            // Skip time index keys
            if key_str.starts_with("time:") {
                continue;
            }

            if let Ok(_state) = serde_json::from_slice::<StoredState>(&value) {
                total_compilations += 1;
                // Could extract target models from metadata
            }
        }

        let most_common_targets: Vec<String> = {
            let mut models: Vec<_> = target_models.into_iter().collect();
            models.sort_by(|a, b| b.1.cmp(&a.1));
            models.into_iter().take(5).map(|(name, _)| name).collect()
        };

        Ok(CompilationStats {
            total_compilations,
            avg_compilation_time_ms: 0.0, // Needs actual measurement
            avg_weight_updates_per_prompt: 0.0, // Needs actual calculation
            most_common_targets,
            convergence_rate: 0.8, // Needs actual calculation
        })
    }

    /// Delete state
    pub fn delete_state(&self, hash: &str) -> Result<()> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        // Get state to find timestamp
        if let Some(state) = self.get_state(hash)? {
            let timestamp_key = format!("time:{}", state.created_at);
            self.db.delete_cf(cf, timestamp_key.as_bytes())?;
        }

        self.db.delete_cf(cf, hash.as_bytes())?;

        Ok(())
    }

    /// List all state hashes
    pub fn list_all_hashes(&self) -> Result<Vec<String>> {
        let cf = self.db.cf_handle("states").ok_or_else(|| {
            StorageError::InvalidData("states column family not found".to_string())
        })?;

        let mut hashes = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) = item?;
            let key_str = String::from_utf8_lossy(&key);

            // Skip time index keys
            if !key_str.starts_with("time:") {
                hashes.push(key_str.to_string());
            }
        }

        Ok(hashes)
    }
}
