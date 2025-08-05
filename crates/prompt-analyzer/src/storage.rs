use crate::analyzer::{PromptAnalysis, PromptComparison};
use crate::sequential::{OptimizationHistory, OptimizationStep};
use prompt_compiler_storage::{Result, StorageError, StateDB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Prompt 分析结果的持久化存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    /// 唯一ID
    pub id: String,
    /// 原始prompt
    pub prompt: String,
    /// 任务描述
    pub task: String,
    /// 分析结果
    pub analysis: PromptAnalysis,
    /// 创建时间
    pub timestamp: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// Prompt 比较记录
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

/// 优化历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    pub id: String,
    pub original_prompt: String,
    pub task: String,
    pub history: OptimizationHistory,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Prompt 分析器的存储层
pub struct PromptAnalysisStorage {
    db: StateDB,
}

impl PromptAnalysisStorage {
    /// 创建新的存储实例
    pub fn new(db_path: &str) -> Result<Self> {
        let db = StateDB::new(db_path)?;
        Ok(Self { db })
    }

    /// 存储单个分析结果
    pub fn store_analysis(&self, record: &AnalysisRecord) -> Result<()> {
        let key = format!("analysis:{}", record.id);
        let serialized = serde_json::to_vec(record)?;

        // 存储到自定义的 analyses column family
        // 注意：这需要扩展现有的 StateDB 来支持更多 column families
        self.store_with_prefix("analysis", &record.id, &serialized)?;

        // 创建时间索引
        let time_key = format!("analysis_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// 存储比较结果
    pub fn store_comparison(&self, record: &ComparisonRecord) -> Result<()> {
        let serialized = serde_json::to_vec(record)?;
        self.store_with_prefix("comparison", &record.id, &serialized)?;

        // 时间索引
        let time_key = format!("comparison_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// 存储优化历史
    pub fn store_optimization(&self, record: &OptimizationRecord) -> Result<()> {
        let serialized = serde_json::to_vec(record)?;
        self.store_with_prefix("optimization", &record.id, &serialized)?;

        // 时间索引
        let time_key = format!("optimization_time:{}:{}", record.timestamp, record.id);
        self.store_with_prefix("index", &time_key, record.id.as_bytes())?;

        Ok(())
    }

    /// 查询分析历史（按时间范围）
    pub fn get_analyses_by_timerange(&self, start: u64, end: u64) -> Result<Vec<AnalysisRecord>> {
        let mut records = Vec::new();

        // 这里需要实现时间范围查询逻辑
        // 类似于现有 StateDB 的 get_states_by_timerange 方法

        Ok(records)
    }

    /// 查询特定任务的分析历史
    pub fn get_analyses_by_task(&self, task: &str) -> Result<Vec<AnalysisRecord>> {
        let mut records = Vec::new();

        // 遍历所有分析记录，过滤指定任务
        // 这里可以优化为使用任务索引

        Ok(records)
    }

    /// 获取 prompt 优化统计
    pub fn get_prompt_optimization_stats(&self) -> Result<PromptOptimizationStats> {
        // 统计所有优化记录，计算总体指标
        Ok(PromptOptimizationStats {
            total_optimizations: 0,
            avg_improvement_rate: 0.0,
            avg_convergence_rate: 0.0,
            most_improved_tasks: vec![],
            optimization_success_rate: 0.0,
        })
    }

    /// 内部辅助方法：带前缀存储
    fn store_with_prefix(&self, prefix: &str, key: &str, data: &[u8]) -> Result<()> {
        let full_key = format!("{}:{}", prefix, key);
        // 这里需要访问底层 RocksDB 实例
        // 可能需要扩展 StateDB 来支持自定义前缀
        Ok(())
    }
}

/// Prompt 优化统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptimizationStats {
    pub total_optimizations: u64,
    pub avg_improvement_rate: f32,
    pub avg_convergence_rate: f32,
    pub most_improved_tasks: Vec<String>,
    pub optimization_success_rate: f32,
}

/// 便捷的记录创建函数
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
