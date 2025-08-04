//! SDK for integrating prompt compiler into other applications

use prompt_compiler_core::{PromptCompiler, PromptIR, CompiledState};
use prompt_compiler_storage::StateDB;
use std::sync::Arc;
use tokio::sync::Mutex;

/// High-level SDK client for prompt compilation
pub struct PromptCompilerSDK {
    compiler: PromptCompiler,
    storage: Option<Arc<Mutex<StateDB>>>,
}

/// SDK configuration
#[derive(Debug, Clone)]
pub struct SDKConfig {
    pub storage_path: Option<String>,
    pub enable_storage: bool,
    pub default_model: String,
    pub default_budget: u32,
}

impl Default for SDKConfig {
    fn default() -> Self {
        Self {
            storage_path: None,
            enable_storage: false,
            default_model: "gpt-4".to_string(),
            default_budget: 1000,
        }
    }
}

impl PromptCompilerSDK {
    /// Create new SDK instance
    pub async fn new(config: SDKConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let compiler = PromptCompiler::new();
        
        let storage = if config.enable_storage {
            let storage_path = config.storage_path
                .unwrap_or_else(|| "./prompt_compiler_sdk.db".to_string());
            let db = StateDB::new(&storage_path)?;
            Some(Arc::new(Mutex::new(db)))
        } else {
            None
        };

        Ok(Self {
            compiler,
            storage,
        })
    }

    /// Compile a prompt
    pub async fn compile(&self, prompt: &str) -> Result<CompiledState, Box<dyn std::error::Error>> {
        let compiled = self.compiler.compile(prompt)?;
        
        // Store if storage is enabled
        if let Some(storage) = &self.storage {
            let hash = prompt_compiler_crypto::Hash::from_string(prompt);
            let storage_guard = storage.lock().await;
            
            // Convert to storable format
            let stored_state = prompt_compiler_storage::StoredState {
                version: compiled.version.clone(),
                content: serde_json::to_vec(&compiled)?,
                created_at: compiled.created_at,
                metadata: compiled.compilation_metadata.clone(),
            };
            
            storage_guard.store_state(hash.as_str(), &stored_state)?;
        }
        
        Ok(compiled)
    }

    /// Analyze a prompt
    pub async fn analyze(&self, prompt: &str) -> Result<f32, Box<dyn std::error::Error>> {
        // Simple analysis score
        Ok(prompt.len() as f32 / 100.0)
    }

    /// Get compilation history
    pub async fn get_history(&self, limit: Option<usize>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if let Some(storage) = &self.storage {
            let storage_guard = storage.lock().await;
            let hashes = storage_guard.list_all_hashes()?;
            
            let limited_hashes = if let Some(l) = limit {
                hashes.into_iter().take(l).collect()
            } else {
                hashes
            };
            
            Ok(limited_hashes)
        } else {
            Ok(Vec::new())
        }
    }
}
