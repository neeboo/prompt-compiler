use crate::analyzer::{PromptAnalysis, PromptComparison};
use crate::sequential::{OptimizationHistory, OptimizationStep};
use prompt_compiler_storage::{Result, StorageError, StateDB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Persistent storage for prompt analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    /// Unique ID
    pub id: String,
    /// Original prompt
    pub prompt: String,
    /// Task description
    pub task: String,
    /// Analysis results
    pub analysis: PromptAnalysis,
    /// Creation timestamp
    pub timestamp: u64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Prompt comparison record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub id: String,
    pub prompt_a: String,
    pub prompt_b: String,
    pub task: String,
    pub comparison: PromptComparison,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Optimization history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    pub id: String,
    pub original_prompt: String,
    pub task: String,
    pub history: OptimizationHistory,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Storage layer for prompt analyzer
pub struct PromptAnalysisStorage {
    db: StateDB,
}

impl PromptAnalysisStorage {
    /// Create new storage instance
    pub fn new(db_path: &str) -> Result<Self> {
        let db = StateDB::new(db_path)?;
        Ok(Self { db })
    }

    /// Store single analysis result
    pub fn store_analysis(&self, record: &AnalysisRecord) -> Result<()> {
        let key = format!("analysis:{}", record.id);
        let serialized = serde_json::to_vec(record)?;

        // Store to custom analyses column family
        // Note: This requires extending existing StateDB to support more column families
        self.store_with_prefix("analysis", &record.id, &serialized)?;

        // Create time index
        let time_key = format!("analysis_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// Store comparison results
    pub fn store_comparison(&self, record: &ComparisonRecord) -> Result<()> {
        let serialized = serde_json::to_vec(record)?;
        self.store_with_prefix("comparison", &record.id, &serialized)?;

        // Time index
        let time_key = format!("comparison_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// Store optimization history
    pub fn store_optimization(&self, record: &OptimizationRecord) -> Result<()> {
        let serialized = serde_json::to_vec(record)?;
        self.store_with_prefix("optimization", &record.id, &serialized)?;

        // Time index
        let time_key = format!("optimization_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// Query analysis history by time range
    pub fn get_analyses_by_timerange(&self, start: u64, end: u64) -> Result<Vec<AnalysisRecord>> {
        let mut records = Vec::new();

        // Time range query logic needs to be implemented here
        // Similar to existing StateDB's get_states_by_timerange method

        Ok(records)
    }

    /// Query analysis history for specific task
    pub fn get_analyses_by_task(&self, task: &str) -> Result<Vec<AnalysisRecord>> {
        let mut records = Vec::new();

        // Iterate through all analysis records, filter by specified task
        // This can be optimized to use task indexing

        Ok(records)
    }

    /// Get prompt optimization statistics
    pub fn get_prompt_optimization_stats(&self) -> Result<PromptOptimizationStats> {
        // Aggregate all optimization records, calculate overall metrics
        Ok(PromptOptimizationStats {
            total_optimizations: 0,
            avg_improvement_rate: 0.0,
            avg_convergence_rate: 0.0,
            most_improved_tasks: vec![],
            optimization_success_rate: 0.0,
        })
    }

    /// Internal helper method: store with prefix
    fn store_with_prefix(&self, prefix: &str, key: &str, data: &[u8]) -> Result<()> {
        let full_key = format!("{}:{}", prefix, key);
        // This needs access to underlying RocksDB instance
        // May need to extend StateDB to support custom prefixes
        Ok(())
    }
}

/// Prompt optimization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptimizationStats {
    pub total_optimizations: u64,
    pub avg_improvement_rate: f32,
    pub avg_convergence_rate: f32,
    pub most_improved_tasks: Vec<String>,
    pub optimization_success_rate: f32,
}

/// Convenient record creation functions
impl AnalysisRecord {
    pub fn new(prompt: String, task: String, analysis: PromptAnalysis) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("analysis_{}", timestamp);

        Self {
            id,
            prompt,
            task,
            analysis,
            timestamp,
            metadata: HashMap::new(),
        }
    }
}

impl ComparisonRecord {
    pub fn new(prompt_a: String, prompt_b: String, task: String, comparison: PromptComparison) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("comparison_{}", timestamp);

        Self {
            id,
            prompt_a,
            prompt_b,
            task,
            comparison,
            timestamp,
            metadata: HashMap::new(),
        }
    }
}

impl OptimizationRecord {
    pub fn new(original_prompt: String, task: String, history: OptimizationHistory) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("optimization_{}", timestamp);

        Self {
            id,
            original_prompt,
            task,
            history,
            timestamp,
            metadata: HashMap::new(),
        }
    }
}
