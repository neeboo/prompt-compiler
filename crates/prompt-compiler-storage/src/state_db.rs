//! State database - RocksDB storage for compiled states

use crate::{Result, StorageError};
use prompt_compiler_crypto::{Hash, Signature};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 语义压缩的上下文片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub id: String,
    pub content_hash: String, // 改为String类型避免Hash类型问题
    pub compressed_embedding: Vec<f32>, // 压缩后的语义表示
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
    pub access_count: u64,
    pub last_accessed: u64,
    pub semantic_tags: Vec<String>,
}

/// 上下文注入策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextInjectionStrategy {
    /// 直接发送给LLM
    DirectSend { max_tokens: usize },
    /// 注入到语义空间
    SemanticInject { similarity_threshold: f32 },
    /// 混合策略
    Hybrid {
        direct_ratio: f32,
        semantic_ratio: f32
    },
}

/// Compiled state for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredState {
    pub version: String,
    pub content: Vec<u8>,
    pub created_at: u64,
    pub metadata: HashMap<String, String>,
    /// 关联的语义块ID
    pub semantic_chunks: Vec<String>,
    /// 注入策略
    pub injection_strategy: ContextInjectionStrategy,
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
    /// 语义压缩统计
    pub semantic_compression_ratio: f32,
    pub avg_chunk_reuse_rate: f32,
    pub context_injection_success_rate: f32,
}

/// RocksDB state database
pub struct StateDB {
    db: DB,
    cf_handles: HashMap<String, rocksdb::ColumnFamily>,
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
            ColumnFamilyDescriptor::new("semantic_chunks", Options::default()),
            ColumnFamilyDescriptor::new("embeddings", Options::default()),
            ColumnFamilyDescriptor::new("stats", Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;

        let cf_handles = HashMap::new();

        Ok(StateDB { db, cf_handles })
    }

    /// 存储语义块
    pub fn store_semantic_chunk(&self, chunk: &SemanticChunk) -> Result<()> {
        let cf = self.db.cf_handle("semantic_chunks")
            .ok_or_else(|| StorageError::ColumnFamilyNotFound("semantic_chunks".to_string()))?;

        let serialized = bincode::serialize(chunk)?;

        self.db.put_cf(&cf, &chunk.id, &serialized)?;

        println!("📦 存储语义块: {} (压缩比: {:.2}%)",
                chunk.id, chunk.compression_ratio * 100.0);
        Ok(())
    }

    /// 根据语义相似度检索上下文
    pub fn retrieve_by_semantic_similarity(
        &self,
        query_embedding: &[f32],
        threshold: f32,
        limit: usize
    ) -> Result<Vec<SemanticChunk>> {
        let cf = self.db.cf_handle("semantic_chunks")
            .ok_or_else(|| StorageError::ColumnFamilyNotFound("semantic_chunks".to_string()))?;

        let mut results = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (_, value) = item?;
            let chunk: SemanticChunk = bincode::deserialize(&value)?;

            // 计算余弦相似度
            let similarity = Self::cosine_similarity(query_embedding, &chunk.compressed_embedding);

            if similarity >= threshold {
                results.push((chunk, similarity));
            }
        }

        // 按相似度排序并限制结果数量
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        Ok(results.into_iter().map(|(chunk, _)| chunk).collect())
    }

    /// 实施上下文注入策略
    pub fn inject_context(
        &self,
        base_prompt: &str,
        strategy: &ContextInjectionStrategy,
        query_embedding: &[f32]
    ) -> Result<String> {
        match strategy {
            ContextInjectionStrategy::DirectSend { max_tokens } => {
                // 直接拼接到prompt中
                let chunks = self.retrieve_by_semantic_similarity(query_embedding, 0.7, 5)?;
                let context = self.compile_context_for_direct_send(&chunks, *max_tokens)?;
                Ok(format!("{}\n\n# 相关上下文:\n{}\n\n# 查询:\n{}",
                          context, self.format_chunks_for_llm(&chunks), base_prompt))
            },

            ContextInjectionStrategy::SemanticInject { similarity_threshold } => {
                // 注入到语义空间（模拟）
                let chunks = self.retrieve_by_semantic_similarity(
                    query_embedding,
                    *similarity_threshold,
                    10
                )?;

                let semantic_context = self.create_semantic_injection(&chunks)?;
                Ok(format!("{}⚡语义注入: {} 个相关块⚡\n{}",
                          semantic_context, chunks.len(), base_prompt))
            },

            ContextInjectionStrategy::Hybrid { direct_ratio, semantic_ratio: _ } => {
                // 混合策略
                let chunks = self.retrieve_by_semantic_similarity(query_embedding, 0.6, 8)?;
                let direct_count = (chunks.len() as f32 * direct_ratio) as usize;

                let direct_chunks = &chunks[..direct_count.min(chunks.len())];
                let semantic_chunks = &chunks[direct_count..];

                let direct_context = self.format_chunks_for_llm(direct_chunks);
                let semantic_injection = self.create_semantic_injection(semantic_chunks)?;

                Ok(format!("{}🔀混合注入🔀\n# 直接上下文:\n{}\n\n# 查询:\n{}",
                          semantic_injection, direct_context, base_prompt))
            }
        }
    }

    /// 压缩并存储新的上下文
    pub fn compress_and_store_context(
        &self,
        content: &str,
        embedding: Vec<f32>
    ) -> Result<SemanticChunk> {
        let original_size = content.len();

        // 简单的压缩模拟（在实际应用中会使用更复杂的语义压缩）
        let compressed_embedding = Self::compress_embedding(&embedding, 128)?;
        let compressed_size = compressed_embedding.len() * 4; // f32 = 4 bytes

        let chunk = SemanticChunk {
            id: format!("chunk_{}", chrono::Utc::now().timestamp_millis()),
            content_hash: format!("{:x}", md5::compute(content.as_bytes())), // 使用md5作为简单hash
            compressed_embedding,
            original_size,
            compressed_size,
            compression_ratio: compressed_size as f32 / original_size as f32,
            access_count: 0,
            last_accessed: chrono::Utc::now().timestamp() as u64,
            semantic_tags: Self::extract_semantic_tags(content),
        };

        self.store_semantic_chunk(&chunk)?;
        Ok(chunk)
    }

    /// 更新编译统计
    pub fn update_compilation_stats(&self, stats: &CompilationStats) -> Result<()> {
        let cf = self.db.cf_handle("stats")
            .ok_or_else(|| StorageError::ColumnFamilyNotFound("stats".to_string()))?;

        let serialized = bincode::serialize(stats)?;

        self.db.put_cf(&cf, b"compilation_stats", &serialized)?;

        println!("📊 更新编译统计: 收敛率 {:.2}%, 压缩比 {:.2}%",
                stats.convergence_rate * 100.0,
                stats.semantic_compression_ratio * 100.0);
        Ok(())
    }

    /// 存储编译状态（为SDK兼容性添加）
    pub fn store_state(&self, hash: &str, state: &StoredState) -> Result<()> {
        let cf = self.db.cf_handle("states")
            .ok_or_else(|| StorageError::ColumnFamilyNotFound("states".to_string()))?;

        let serialized = bincode::serialize(state)?;
        self.db.put_cf(&cf, hash.as_bytes(), &serialized)?;

        println!("💾 存储编译状态: {}", hash);
        Ok(())
    }

    /// 列出所有哈希值（为SDK兼容性添加）
    pub fn list_all_hashes(&self) -> Result<Vec<String>> {
        let cf = self.db.cf_handle("states")
            .ok_or_else(|| StorageError::ColumnFamilyNotFound("states".to_string()))?;

        let mut hashes = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) = item?;
            if let Ok(hash_str) = String::from_utf8(key.to_vec()) {
                hashes.push(hash_str);
            }
        }

        Ok(hashes)
    }

    // 辅助方法
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn compress_embedding(embedding: &[f32], target_dim: usize) -> Result<Vec<f32>> {
        // 简单的维度压缩（PCA的简化版本）
        if embedding.len() <= target_dim {
            return Ok(embedding.to_vec());
        }

        let chunk_size = embedding.len() / target_dim;
        let compressed: Vec<f32> = embedding
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect();

        Ok(compressed)
    }

    fn extract_semantic_tags(content: &str) -> Vec<String> {
        // 简单的关键词提取
        content
            .split_whitespace()
            .filter(|word| word.len() > 4)
            .take(5)
            .map(|s| s.to_lowercase())
            .collect()
    }

    fn format_chunks_for_llm(&self, chunks: &[SemanticChunk]) -> String {
        chunks.iter()
            .enumerate()
            .map(|(i, chunk)| {
                format!("## 上下文片段 {}\n标签: {:?}\n使用次数: {}\n",
                       i + 1, chunk.semantic_tags, chunk.access_count)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn create_semantic_injection(&self, chunks: &[SemanticChunk]) -> Result<String> {
        let total_compression = chunks.iter()
            .map(|c| c.compression_ratio)
            .sum::<f32>() / chunks.len() as f32;

        Ok(format!("🧠[语义空间注入: {} 块, 平均压缩比 {:.1}%]",
                  chunks.len(), total_compression * 100.0))
    }

    fn compile_context_for_direct_send(&self, chunks: &[SemanticChunk], max_tokens: usize) -> Result<String> {
        let mut context = String::new();
        let mut token_count = 0;

        for chunk in chunks {
            let chunk_info = format!("压缩比: {:.1}% | 标签: {:?}",
                                   chunk.compression_ratio * 100.0,
                                   chunk.semantic_tags);

            if token_count + chunk_info.len() > max_tokens {
                break;
            }

            context.push_str(&chunk_info);
            context.push('\n');
            token_count += chunk_info.len();
        }

        Ok(context)
    }
}
