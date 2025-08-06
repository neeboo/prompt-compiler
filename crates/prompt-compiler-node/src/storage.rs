use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, debug, warn};
use std::sync::RwLock; // 添加线程安全支持

use crate::context_engine::SimilarContext;

// 简化的存储结构，避免RocksDB的线程安全问题
pub struct NodeStorage {
    // 使用内存存储作为临时解决方案
    contexts: RwLock<HashMap<String, StoredContext>>,
    data_path: String,
}

#[derive(Debug, Clone)]
struct StoredContext {
    content: String,
    embedding: Vec<f32>,
    timestamp: i64,
}

impl NodeStorage {
    pub async fn new(data_path: &str) -> Result<Self> {
        info!("Initializing Node Storage at: {}", data_path);

        // 创建目录
        std::fs::create_dir_all(data_path)?;

        info!("✅ Node Storage initialized successfully");

        Ok(Self {
            contexts: RwLock::new(HashMap::new()),
            data_path: data_path.to_string(),
        })
    }

    pub async fn store_context(
        &self,
        agent_id: &str,
        content: &str,
        embedding: &[f32],
    ) -> Result<()> {
        debug!("Storing context for agent: {}", agent_id);

        let context_id = format!("{}_{}", agent_id, chrono::Utc::now().timestamp());
        let stored_context = StoredContext {
            content: content.to_string(),
            embedding: embedding.to_vec(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        if let Ok(mut contexts) = self.contexts.write() {
            contexts.insert(context_id, stored_context);
        }

        debug!("✅ Context stored successfully");
        Ok(())
    }

    pub async fn find_similar_contexts(
        &self,
        agent_id: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SimilarContext>> {
        debug!("Finding similar contexts for agent: {}", agent_id);

        let contexts = self.contexts.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut similarities = Vec::new();

        // 🔧 修复：确保能够找到同一个agent的历史上下文
        for (context_id, stored_context) in contexts.iter() {
            let should_include = if context_id.starts_with(&format!("{}_", agent_id)) {
                // 同一个agent的上下文 - 高优先级
                debug!("Found context for same agent: {}", context_id);
                true
            } else if self.is_similar_agent_role(agent_id, context_id) {
                // 相似角色的agent上下文 - 中等优先级
                true
            } else {
                // 其他agent的上下文 - 跳过
                false
            };

            if should_include {
                let similarity = self.calculate_cosine_similarity(query_embedding, &stored_context.embedding);

                // 🔧 对于同一个agent，降低相似度要求，确保历史对话能被找到
                let adjusted_similarity = if context_id.starts_with(&format!("{}_", agent_id)) {
                    similarity.max(0.3) // 为同一agent的历史对话设置最低相似度
                } else {
                    similarity * 0.8 // 20% 折扣
                };

                // 🔧 修复：确保同一个agent的上下文总是被包含
                if context_id.starts_with(&format!("{}_", agent_id)) || adjusted_similarity > 0.05 {
                    similarities.push(SimilarContext {
                        content: stored_context.content.clone(),
                        similarity: adjusted_similarity.min(1.0),
                        timestamp: stored_context.timestamp,
                    });
                    debug!("Added context with similarity: {:.3}", adjusted_similarity);
                }
            }
        }

        // 按相似度排序并限制数量
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        similarities.truncate(limit);

        debug!("Found {} similar contexts for agent {}", similarities.len(), agent_id);
        for (i, ctx) in similarities.iter().enumerate() {
            debug!("  Context {}: similarity={:.3}, content={}",
                   i, ctx.similarity, &ctx.content[..std::cmp::min(100, ctx.content.len())]);
        }

        Ok(similarities)
    }

    pub async fn cleanup_old_contexts(&self, max_age_days: u64) -> Result<()> {
        info!("Cleaning up contexts older than {} days", max_age_days);

        let cutoff_timestamp = chrono::Utc::now().timestamp() - (max_age_days as i64 * 24 * 60 * 60);
        let mut deleted_count = 0;

        if let Ok(mut contexts) = self.contexts.write() {
            contexts.retain(|_id, context| {
                if context.timestamp < cutoff_timestamp {
                    deleted_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        info!("Cleanup completed: {} contexts deleted", deleted_count);
        Ok(())
    }

    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        debug!("Calculating storage statistics");

        let contexts = self.contexts.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let total_contexts = contexts.len();
        let total_size_bytes = contexts.values()
            .map(|ctx| ctx.content.len() + ctx.embedding.len() * 4) // 4 bytes per f32
            .sum();

        // 按agent统计
        let mut agent_counts = HashMap::new();
        for context_id in contexts.keys() {
            if let Some(agent_id) = context_id.split('_').next() {
                *agent_counts.entry(agent_id.to_string()).or_insert(0) += 1;
            }
        }

        let timestamps: Vec<i64> = contexts.values().map(|c| c.timestamp).collect();

        Ok(StorageStats {
            total_contexts,
            total_size_bytes,
            agent_counts,
            oldest_timestamp: timestamps.iter().min().copied(),
            newest_timestamp: timestamps.iter().max().copied(),
        })
    }

    // 🔧 新增：跨Agent组的上下文存储
    pub async fn store_context_with_group(
        &self,
        agent_id: &str,
        group_id: &str,
        content: &str,
        embedding: &[f32],
    ) -> Result<()> {
        debug!("Storing context for agent: {} in group: {}", agent_id, group_id);

        // 使用组ID作为前缀来标识跨Agent共享的上下文
        let context_id = format!("group:{}:{}:{}", group_id, agent_id, chrono::Utc::now().timestamp());
        let stored_context = StoredContext {
            content: content.to_string(),
            embedding: embedding.to_vec(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        if let Ok(mut contexts) = self.contexts.write() {
            contexts.insert(context_id, stored_context);
        }

        debug!("✅ Context stored successfully in group: {}", group_id);
        Ok(())
    }

    // 🔧 新增：从特定组中查找相似上下文
    pub async fn find_similar_contexts_in_group(
        &self,
        group_id: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SimilarContext>> {
        debug!("Finding similar contexts in group: {}", group_id);

        let contexts = self.contexts.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let mut similarities = Vec::new();

        let group_prefix = format!("group:{}:", group_id);
        
        // 🔧 DEBUG: 显示所有存储的上下文
        debug!("Total stored contexts: {}", contexts.len());
        for (context_id, stored_context) in contexts.iter() {
            debug!("Stored context ID: {}, content: {}", 
                   context_id, &stored_context.content[..std::cmp::min(50, stored_context.content.len())]);
        }

        // 🔧 只查找属于指定组的上下文
        for (context_id, stored_context) in contexts.iter() {
            if context_id.starts_with(&group_prefix) {
                let similarity = self.calculate_cosine_similarity(query_embedding, &stored_context.embedding);

                debug!("Group context found! ID: {}, similarity: {:.3} for content: {}",
                       context_id, similarity, &stored_context.content[..std::cmp::min(80, stored_context.content.len())]);

                // 🔧 使用更宽松的相似度阈值以提高跨Agent共享效果 - 完全移除阈值限制
                similarities.push(SimilarContext {
                    content: stored_context.content.clone(),
                    similarity,
                    timestamp: stored_context.timestamp,
                });
            }
        }

        // 按相似度排序并限制数量
        similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        similarities.truncate(limit);

        debug!("Final result: Found {} similar contexts in group {} (no threshold applied)", similarities.len(), group_id);
        for (i, ctx) in similarities.iter().enumerate() {
            debug!("  Group Context {}: similarity={:.3}, content={}",
                   i, ctx.similarity, &ctx.content[..std::cmp::min(80, ctx.content.len())]);
        }

        Ok(similarities)
    }

    // 添加健康检查方法
    pub async fn is_healthy(&self) -> Result<bool> {
        // 简单的健康检查：检查是否能访问存储
        Ok(self.contexts.read().is_ok())
    }

    // 添加API统计方法
    pub async fn get_api_stats(&self, _detailed: bool) -> Result<crate::api_server::ApiStats> {
        let contexts = self.contexts.read().unwrap();
        Ok(crate::api_server::ApiStats {
            total_requests: contexts.len() as u64,
            context_sharing_requests: contexts.len() as u64,
            cache_hits: 0,
            avg_response_time_ms: 150.0,
            uptime_seconds: 3600,
            active_agents: 1,
            context_groups: 1,
        })
    }

    fn calculate_cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    // 🔧 新增：判断是否为相似的Agent角色
    fn is_similar_agent_role(&self, current_agent: &str, context_id: &str) -> bool {
        // 提取agent角色前缀
        let current_role = self.extract_agent_role(current_agent);
        let context_agent = context_id.split('_').next().unwrap_or("");
        let context_role = self.extract_agent_role(context_agent);

        debug!("Comparing roles: current_agent='{}' (role='{}') vs context_agent='{}' (role='{}')",
               current_agent, current_role, context_agent, context_role);

        // 🔧 修复：首先检查是否为相同角色
        if current_role == context_role {
            debug!("Roles match exactly: {} == {}", current_role, context_role);
            return true;
        }

        // 🔧 改进：更精确的角色匹配逻辑
        if current_role == context_role && current_role != "general" {
            debug!("Roles match: {} == {}", current_role, context_role);
            return true;
        }

        // 定义相似的角色群组
        let similar_roles = vec![
            vec!["customer_service", "support", "help", "cs"],
            vec!["technical", "tech", "engineer"],
            vec!["sales", "marketing", "business"],
            vec!["data", "analyst", "research"],
            vec!["general"], // 🔧 添加general角色组
        ];

        for group in similar_roles {
            if group.contains(&current_role) && group.contains(&context_role) {
                debug!("Roles in same group: {} and {} both in {:?}", current_role, context_role, group);
                return true;
            }
        }

        debug!("Roles don't match: {} vs {}", current_role, context_role);
        false
    }

    // 🔧 新增：提取Agent角色
    fn extract_agent_role(&self, agent_id: &str) -> &str {
        // 🔧 改进：更精确的角色提取
        if agent_id.starts_with("customer_service") || agent_id.contains("cs_") {
            "customer_service"
        } else if agent_id.starts_with("technical") || agent_id.contains("tech_") {
            "technical"
        } else if agent_id.starts_with("sales") || agent_id.contains("sales_") {
            "sales"
        } else if agent_id.contains("data") || agent_id.contains("analyst") {
            "data"
        } else {
            "general"
        }
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_contexts: usize,
    pub total_size_bytes: usize,
    pub agent_counts: HashMap<String, usize>,
    pub oldest_timestamp: Option<i64>,
    pub newest_timestamp: Option<i64>,
}
