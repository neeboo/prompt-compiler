// Context Compression & Sharing Infrastructure Architecture
// 作为Agent和LLM之间的独立基础设施

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 🏗️ 核心架构组件 - Prompt Compiler作为LLM代理层
///
/// 系统架构：Agent通过PC与LLM交互，PC负责所有的优化和管理
///
/// ```
/// ┌─────────────┐    ┌─────────────────────────────────────┐    ┌─────────────┐
/// │   Agent A   │───▶│        Prompt Compiler              │───▶│   LLM API   │
/// └─────────────┘    │       (智能代理层)                   │    └─────────────┘
/// ┌─────────────┐    │                                     │          │
/// │   Agent B   │───▶│  ┌─────────────────┐                │          │
/// └─────────────┘    │  │ Context Manager │  ◀────────────────────────┘
/// ┌─────────────┐    │  └─────────────────┘                │
/// │   Agent C   │───▶│  ┌─────────────────┐                │
/// └─────────────┘    │  │ Response Proxy  │                │
///                    │  └─────────────────┘                │
///                    │  ┌─────────────────┐                │
///                    │  │ Weight Dynamics │                │
///                    │  └─────────────────┘                │
///                    │  ┌─────────────────┐                │
///                    │  │ Storage Engine  │                │
///                    │  └─────────────────┘                │
///                    │  ┌─────────────────┐                │
///                    │  │ Semantic Cache  │                │
///                    │  └─────────────────┘                │
///                    └─────────────────────────────────────┘
///                                │
///                                ▼
///                    ┌─────────────────────────┐
///                    │    Shared Knowledge     │
///                    │    Base (RocksDB)       │
///                    └─────────────────────────┘
/// ```
///
/// 🎯 关键点：
/// 1. Agent不直接访问LLM，所有请求都通过PC代理
/// 2. PC负责上下文压缩、知识共享、成本优化
/// 3. PC管理所有与LLM的交互，包括缓存、降级、重试
/// 4. Agent只需要关注业务逻辑，不需要考虑效率问题

/// Agent请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent_id: String,
    pub session_id: String,
    pub context: AgentContext,
    pub query: String,
    pub metadata: RequestMetadata,
}

/// Agent上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub agent_type: String,        // "customer_service", "technical_support", etc.
    pub user_profile: Option<UserProfile>,
    pub conversation_history: Vec<ConversationTurn>,
    pub domain_knowledge: Vec<String>,
    pub preferences: HashMap<String, String>,
}

/// 用户档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub demographics: HashMap<String, String>,
    pub interaction_history: Vec<String>,
    pub preferences: HashMap<String, String>,
    pub satisfaction_scores: Vec<f64>,
}

/// 对话轮次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub timestamp: u64,
    pub user_message: String,
    pub agent_response: String,
    pub quality_score: f64,
}

/// 请求元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub priority: Priority,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub context_budget: Option<u32>, // 上下文token预算
    pub quality_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// 压缩后的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedContext {
    pub context_id: String,
    pub semantic_vector: Vec<f64>,
    pub key_information: String, // 关键信息摘要
    pub compression_ratio: f64,
    pub original_tokens: u32,
    pub compressed_tokens: u32,
    pub reuse_count: u32,
    pub last_accessed: u64,
}

/// LLM请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedLLMRequest {
    pub optimized_prompt: String,
    pub context_references: Vec<String>, // 上下文引用ID
    pub metadata: LLMRequestMetadata,
    pub fallback_context: Option<String>, // 降级方案
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequestMetadata {
    pub original_tokens: u32,
    pub optimized_tokens: u32,
    pub compression_ratio: f64,
    pub context_reuse_percentage: f64,
    pub estimated_cost: f64,
}

/// 系统响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureResponse {
    pub agent_id: String,
    pub session_id: String,
    pub response: String,
    pub metrics: ResponseMetrics,
    pub context_updates: Vec<ContextUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    pub tokens_saved: u32,
    pub processing_time_ms: u64,
    pub compression_ratio: f64,
    pub cache_hit_rate: f64,
    pub quality_score: f64,
    pub cost_reduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    pub context_id: String,
    pub update_type: UpdateType,
    pub new_information: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    UserProfile,
    DomainKnowledge,
    ConversationHistory,
    Preferences,
}

/// 🎯 核心基础设施 - Context Compression Infrastructure
pub struct ContextCompressionInfrastructure {
    context_manager: Arc<RwLock<ContextManager>>,
    weight_dynamics: Arc<RwLock<WeightDynamicsEngine>>,
    storage_engine: Arc<RwLock<StorageEngine>>,
    semantic_cache: Arc<RwLock<SemanticCache>>,
    llm_interface: Arc<RwLock<LLMInterface>>,
    metrics_collector: Arc<RwLock<MetricsCollector>>,
}

impl ContextCompressionInfrastructure {
    /// 🚀 完整的处理workflow
    pub async fn process_agent_request(
        &self,
        request: AgentRequest,
    ) -> Result<InfrastructureResponse, Box<dyn Error + Send + Sync>> {

        println!("🎯 Processing request from agent: {}", request.agent_id);

        // Phase 1: Context Analysis & Compression
        let compressed_context = self.analyze_and_compress_context(&request).await?;

        // Phase 2: Semantic Cache Lookup
        let cache_result = self.check_semantic_cache(&request, &compressed_context).await?;

        // Phase 3: Context Sharing & Cross-Agent Learning
        let shared_knowledge = self.retrieve_shared_knowledge(&request).await?;

        // Phase 4: Prompt Optimization
        let optimized_request = self.optimize_prompt(
            &request,
            &compressed_context,
            &shared_knowledge,
            cache_result.as_ref(),
        ).await?;

        // Phase 5: LLM Interaction (if cache miss)
        let llm_response = if cache_result.is_none() {
            Some(self.interact_with_llm(&optimized_request).await?)
        } else {
            None
        };

        // Phase 6: Response Processing & Context Update
        let final_response = self.process_response(
            &request,
            cache_result.or(llm_response),
            &compressed_context,
        ).await?;

        // Phase 7: Knowledge Base Update
        self.update_knowledge_base(&request, &final_response).await?;

        // Phase 8: Metrics Collection
        self.collect_metrics(&request, &final_response).await?;

        Ok(final_response)
    }

    /// Phase 1: 上下文分析与压缩
    async fn analyze_and_compress_context(
        &self,
        request: &AgentRequest,
    ) -> Result<CompressedContext, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 1: Analyzing and compressing context...");

        let context_manager = self.context_manager.read().await;

        // 1.1 分析上下文复杂度
        let complexity_score = context_manager.analyze_context_complexity(&request.context);

        // 1.2 识别可压缩的信息
        let compressible_parts = context_manager.identify_compressible_information(&request.context);

        // 1.3 应用语义压缩算法
        let semantic_vector = context_manager.generate_semantic_vector(&request.context);

        // 1.4 生成关键信息摘要
        let key_information = context_manager.extract_key_information(&request.context);

        // 1.5 计算压缩比率
        let original_tokens = context_manager.estimate_tokens(&request.context);
        let compressed_tokens = context_manager.estimate_compressed_tokens(&key_information);
        let compression_ratio = 1.0 - (compressed_tokens as f64 / original_tokens as f64);

        println!("   ✅ Compression ratio: {:.1}% ({} → {} tokens)",
                compression_ratio * 100.0, original_tokens, compressed_tokens);

        Ok(CompressedContext {
            context_id: format!("ctx_{}_{}", request.agent_id, request.session_id),
            semantic_vector,
            key_information,
            compression_ratio,
            original_tokens,
            compressed_tokens,
            reuse_count: 0,
            last_accessed: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Phase 2: 语义缓存检查
    async fn check_semantic_cache(
        &self,
        request: &AgentRequest,
        compressed_context: &CompressedContext,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 2: Checking semantic cache...");

        let cache = self.semantic_cache.read().await;

        // 2.1 计算查询的语义向量
        let query_vector = cache.generate_query_vector(&request.query, &compressed_context.semantic_vector);

        // 2.2 在缓存中搜索相似的查询
        let similar_entries = cache.find_similar_queries(&query_vector, 0.85); // 85%相似度阈值

        if let Some(cache_hit) = similar_entries.first() {
            println!("   🎯 Cache HIT! Similarity: {:.1}%", cache_hit.similarity * 100.0);

            // 2.3 验证缓存条目是否仍然有效
            if cache.is_cache_valid(&cache_hit.entry, compressed_context) {
                return Ok(Some(cache_hit.response.clone()));
            }
        }

        println!("   ❌ Cache MISS");
        Ok(None)
    }

    /// Phase 3: 共享知识检索
    async fn retrieve_shared_knowledge(
        &self,
        request: &AgentRequest,
    ) -> Result<SharedKnowledge, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 3: Retrieving shared knowledge...");

        let storage = self.storage_engine.read().await;

        // 3.1 检索相同类型Agent的经验
        let agent_type_knowledge = storage.get_agent_type_knowledge(&request.context.agent_type).await?;

        // 3.2 检索用户相关的历史经验
        let user_knowledge = if let Some(profile) = &request.context.user_profile {
            storage.get_user_knowledge(&profile.user_id).await?
        } else {
            Vec::new()
        };

        // 3.3 检索领域相关知识
        let domain_knowledge = storage.get_domain_knowledge(&request.context.domain_knowledge).await?;

        // 3.4 跨Agent协作知识
        let cross_agent_knowledge = storage.get_cross_agent_knowledge(&request.agent_id).await?;

        println!("   ✅ Retrieved {} knowledge entries",
                agent_type_knowledge.len() + user_knowledge.len() + domain_knowledge.len());

        Ok(SharedKnowledge {
            agent_type_knowledge,
            user_knowledge,
            domain_knowledge,
            cross_agent_knowledge,
        })
    }

    /// Phase 4: Prompt优化
    async fn optimize_prompt(
        &self,
        request: &AgentRequest,
        compressed_context: &CompressedContext,
        shared_knowledge: &SharedKnowledge,
        cached_response: Option<&String>,
    ) -> Result<OptimizedLLMRequest, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 4: Optimizing prompt...");

        if cached_response.is_some() {
            println!("   ⚡ Using cached response, skipping LLM call");
            return Ok(OptimizedLLMRequest {
                optimized_prompt: "CACHED_RESPONSE".to_string(),
                context_references: vec![compressed_context.context_id.clone()],
                metadata: LLMRequestMetadata {
                    original_tokens: compressed_context.original_tokens,
                    optimized_tokens: 0, // 缓存命中，无需token
                    compression_ratio: 1.0,
                    context_reuse_percentage: 1.0,
                    estimated_cost: 0.0,
                },
                fallback_context: None,
            });
        }

        let context_manager = self.context_manager.read().await;

        // 4.1 构建优化的prompt
        let base_prompt = context_manager.build_base_prompt(&request.context.agent_type);

        // 4.2 注入压缩的上下文
        let context_injection = context_manager.inject_compressed_context(compressed_context);

        // 4.3 添加共享知识引用
        let knowledge_references = context_manager.build_knowledge_references(shared_knowledge);

        // 4.4 个性化调整
        let personalization = if let Some(profile) = &request.context.user_profile {
            context_manager.build_personalization(profile)
        } else {
            String::new()
        };

        // 4.5 组装最终prompt
        let optimized_prompt = format!(
            "{}\n{}\n{}\n{}\nQuery: {}",
            base_prompt,
            context_injection,
            knowledge_references,
            personalization,
            request.query
        );

        let optimized_tokens = context_manager.estimate_prompt_tokens(&optimized_prompt);
        let compression_ratio = 1.0 - (optimized_tokens as f64 / compressed_context.original_tokens as f64);

        println!("   ✅ Prompt optimized: {} → {} tokens ({:.1}% compression)",
                compressed_context.original_tokens, optimized_tokens, compression_ratio * 100.0);

        Ok(OptimizedLLMRequest {
            optimized_prompt,
            context_references: vec![compressed_context.context_id.clone()],
            metadata: LLMRequestMetadata {
                original_tokens: compressed_context.original_tokens,
                optimized_tokens,
                compression_ratio,
                context_reuse_percentage: shared_knowledge.calculate_reuse_percentage(),
                estimated_cost: (optimized_tokens as f64 / 1000.0) * 0.03, // GPT-4 pricing
            },
            fallback_context: Some(compressed_context.key_information.clone()),
        })
    }

    /// Phase 5: LLM交互
    async fn interact_with_llm(
        &self,
        optimized_request: &OptimizedLLMRequest,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 5: Interacting with LLM...");

        let llm_interface = self.llm_interface.read().await;

        // 5.1 发送优化后的请求到LLM
        let response = llm_interface.send_request(&optimized_request.optimized_prompt).await?;

        // 5.2 验证响应质量
        let quality_score = llm_interface.evaluate_response_quality(&response);

        // 5.3 如果质量不足，使用fallback
        if quality_score < 0.7 {
            if let Some(fallback) = &optimized_request.fallback_context {
                println!("   ⚠️ Low quality response, using fallback context");
                let fallback_prompt = format!("{}\n{}", fallback, optimized_request.optimized_prompt);
                return llm_interface.send_request(&fallback_prompt).await;
            }
        }

        println!("   ✅ LLM response received (quality: {:.1}%)", quality_score * 100.0);
        Ok(response)
    }

    /// Phase 6: 响应处理
    async fn process_response(
        &self,
        request: &AgentRequest,
        llm_response: Option<String>,
        compressed_context: &CompressedContext,
    ) -> Result<InfrastructureResponse, Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 6: Processing response...");

        let response_text = llm_response.unwrap_or_else(|| "Cached response used".to_string());

        // 6.1 计算性能指标
        let metrics = ResponseMetrics {
            tokens_saved: compressed_context.original_tokens.saturating_sub(compressed_context.compressed_tokens),
            processing_time_ms: 50, // 模拟处理时间
            compression_ratio: compressed_context.compression_ratio,
            cache_hit_rate: if llm_response.is_none() { 1.0 } else { 0.0 },
            quality_score: 0.95, // 模拟质量分数
            cost_reduction: 0.54, // 基于benchmark结果
        };

        // 6.2 生成上下文更新
        let context_updates = vec![
            ContextUpdate {
                context_id: compressed_context.context_id.clone(),
                update_type: UpdateType::ConversationHistory,
                new_information: format!("Q: {} A: {}", request.query, response_text),
                confidence: 0.9,
            }
        ];

        println!("   ✅ Response processed with {:.1}% cost reduction",
                metrics.cost_reduction * 100.0);

        Ok(InfrastructureResponse {
            agent_id: request.agent_id.clone(),
            session_id: request.session_id.clone(),
            response: response_text,
            metrics,
            context_updates,
        })
    }

    /// Phase 7: 知识库更新
    async fn update_knowledge_base(
        &self,
        request: &AgentRequest,
        response: &InfrastructureResponse,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 7: Updating knowledge base...");

        let storage = self.storage_engine.write().await;
        let weight_dynamics = self.weight_dynamics.write().await;

        // 7.1 更新权重动力学
        weight_dynamics.update_weights(
            &request.agent_id,
            &request.query,
            &response.response,
            response.metrics.quality_score,
        ).await?;

        // 7.2 存储新的经验
        storage.store_interaction(request, response).await?;

        // 7.3 更新跨Agent共享知识
        if response.metrics.quality_score > 0.8 {
            storage.add_to_shared_knowledge(
                &request.context.agent_type,
                &request.query,
                &response.response,
                response.metrics.quality_score,
            ).await?;
        }

        // 7.4 更新语义缓存
        let cache = self.semantic_cache.write().await;
        cache.add_to_cache(&request.query, &response.response, response.metrics.quality_score).await?;

        println!("   ✅ Knowledge base updated");
        Ok(())
    }

    /// Phase 8: 指标收集
    async fn collect_metrics(
        &self,
        request: &AgentRequest,
        response: &InfrastructureResponse,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        println!("🔍 Phase 8: Collecting metrics...");

        let metrics_collector = self.metrics_collector.write().await;

        // 8.1 记录性能指标
        metrics_collector.record_performance(&response.metrics).await?;

        // 8.2 记录Agent使用模式
        metrics_collector.record_agent_pattern(&request.agent_id, &request.context.agent_type).await?;

        // 8.3 记录成本节约
        metrics_collector.record_cost_savings(response.metrics.cost_reduction).await?;

        // 8.4 记录质量分数
        metrics_collector.record_quality_score(response.metrics.quality_score).await?;

        println!("   ✅ Metrics collected");
        Ok(())
    }
}

/// 🔧 支持组件定义 (简化版)

struct ContextManager;
impl ContextManager {
    fn analyze_context_complexity(&self, _context: &AgentContext) -> f64 { 0.8 }
    fn identify_compressible_information(&self, _context: &AgentContext) -> Vec<String> { vec![] }
    fn generate_semantic_vector(&self, _context: &AgentContext) -> Vec<f64> { vec![0.1; 128] }
    fn extract_key_information(&self, _context: &AgentContext) -> String { "Key info".to_string() }
    fn estimate_tokens(&self, _context: &AgentContext) -> u32 { 500 }
    fn estimate_compressed_tokens(&self, _info: &str) -> u32 { 200 }
    fn build_base_prompt(&self, agent_type: &str) -> String { format!("You are a {} agent.", agent_type) }
    fn inject_compressed_context(&self, _context: &CompressedContext) -> String { "Context: ...".to_string() }
    fn build_knowledge_references(&self, _knowledge: &SharedKnowledge) -> String { "Knowledge: ...".to_string() }
    fn build_personalization(&self, _profile: &UserProfile) -> String { "Personalization: ...".to_string() }
    fn estimate_prompt_tokens(&self, prompt: &str) -> u32 { (prompt.len() / 4) as u32 }
}

struct WeightDynamicsEngine;
impl WeightDynamicsEngine {
    async fn update_weights(&self, _agent_id: &str, _query: &str, _response: &str, _quality: f64) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
}

struct StorageEngine;
impl StorageEngine {
    async fn get_agent_type_knowledge(&self, _agent_type: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> { Ok(vec![]) }
    async fn get_user_knowledge(&self, _user_id: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> { Ok(vec![]) }
    async fn get_domain_knowledge(&self, _domains: &[String]) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> { Ok(vec![]) }
    async fn get_cross_agent_knowledge(&self, _agent_id: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> { Ok(vec![]) }
    async fn store_interaction(&self, _request: &AgentRequest, _response: &InfrastructureResponse) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
    async fn add_to_shared_knowledge(&self, _agent_type: &str, _query: &str, _response: &str, _quality: f64) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
}

struct SemanticCache;
impl SemanticCache {
    fn generate_query_vector(&self, _query: &str, _context_vector: &[f64]) -> Vec<f64> { vec![0.2; 128] }
    fn find_similar_queries(&self, _vector: &[f64], _threshold: f64) -> Vec<CacheEntry> { vec![] }
    fn is_cache_valid(&self, _entry: &CacheEntry, _context: &CompressedContext) -> bool { true }
    async fn add_to_cache(&self, _query: &str, _response: &str, _quality: f64) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
}

struct LLMInterface;
impl LLMInterface {
    async fn send_request(&self, _prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("LLM response".to_string())
    }
    fn evaluate_response_quality(&self, _response: &str) -> f64 { 0.9 }
}

struct MetricsCollector;
impl MetricsCollector {
    async fn record_performance(&self, _metrics: &ResponseMetrics) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
    async fn record_agent_pattern(&self, _agent_id: &str, _agent_type: &str) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
    async fn record_cost_savings(&self, _cost_reduction: f64) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
    async fn record_quality_score(&self, _quality: f64) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
}

#[derive(Debug, Clone)]
struct SharedKnowledge {
    agent_type_knowledge: Vec<String>,
    user_knowledge: Vec<String>,
    domain_knowledge: Vec<String>,
    cross_agent_knowledge: Vec<String>,
}

impl SharedKnowledge {
    fn calculate_reuse_percentage(&self) -> f64 {
        let total_entries = self.agent_type_knowledge.len() +
                           self.user_knowledge.len() +
                           self.domain_knowledge.len() +
                           self.cross_agent_knowledge.len();
        if total_entries > 0 { 0.8 } else { 0.0 }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    similarity: f64,
    response: String,
    entry: String,
}

/// 🚀 Demo运行示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("🏗️ Context Compression Infrastructure Demo");
    println!("{}", "=".repeat(60));

    // 初始化基础设施
    let infrastructure = ContextCompressionInfrastructure {
        context_manager: Arc::new(RwLock::new(ContextManager)),
        weight_dynamics: Arc::new(RwLock::new(WeightDynamicsEngine)),
        storage_engine: Arc::new(RwLock::new(StorageEngine)),
        semantic_cache: Arc::new(RwLock::new(SemanticCache)),
        llm_interface: Arc::new(RwLock::new(LLMInterface)),
        metrics_collector: Arc::new(RwLock::new(MetricsCollector)),
    };

    // 模拟Agent请求
    let agent_request = AgentRequest {
        agent_id: "cs_agent_001".to_string(),
        session_id: "session_12345".to_string(),
        context: AgentContext {
            agent_type: "customer_service".to_string(),
            user_profile: Some(UserProfile {
                user_id: "user_zhang".to_string(),
                demographics: [("age".to_string(), "35".to_string())].iter().cloned().collect(),
                interaction_history: vec!["Previous login issue resolved".to_string()],
                preferences: [("communication_style".to_string(), "technical".to_string())].iter().cloned().collect(),
                satisfaction_scores: vec![0.9, 0.8, 0.95],
            }),
            conversation_history: vec![
                ConversationTurn {
                    timestamp: 1234567890,
                    user_message: "I can't login".to_string(),
                    agent_response: "Please clear your cache".to_string(),
                    quality_score: 0.9,
                }
            ],
            domain_knowledge: vec!["login_issues".to_string(), "password_reset".to_string()],
            preferences: HashMap::new(),
        },
        query: "Customer reports login failure again, same as last week".to_string(),
        metadata: RequestMetadata {
            priority: Priority::High,
            max_tokens: Some(150),
            temperature: Some(0.7),
            context_budget: Some(300),
            quality_threshold: Some(0.8),
        },
    };

    // 处理请求
    let response = infrastructure.process_agent_request(agent_request).await?;

    // 显示结果
    println!("\n📋 Final Response:");
    println!("   Agent: {}", response.agent_id);
    println!("   Response: {}", response.response);
    println!("   Tokens Saved: {}", response.metrics.tokens_saved);
    println!("   Cost Reduction: {:.1}%", response.metrics.cost_reduction * 100.0);
    println!("   Quality Score: {:.1}%", response.metrics.quality_score * 100.0);

    println!("\n✨ Infrastructure demo completed!");
    Ok(())
}
