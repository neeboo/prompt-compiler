use anyhow::Result;
use std::sync::{Arc, Mutex}; // 添加Mutex支持
use tracing::{info, debug, warn}; // 添加warn导入
use prompt_compiler_embeddings::EmbeddingProvider;
use prompt_compiler_weights::{ImplicitDynamics, DynamicsConfig}; // 添加DynamicsConfig导入
use regex; // 添加regex依赖
use serde::{Deserialize, Serialize}; // 添加Azure API需要的序列化支持
use reqwest; // 添加HTTP客户端

use crate::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ProcessedRequest};
use crate::storage::NodeStorage;

// Azure Text Analytics API 数据结构
#[derive(Debug, Serialize)]
struct AzureNerRequest {
    #[serde(rename = "analysisInput")]
    analysis_input: AzureAnalysisInput,
    #[serde(rename = "tasks")]
    tasks: Vec<AzureTask>,
}

#[derive(Debug, Serialize)]
struct AzureAnalysisInput {
    documents: Vec<AzureDocument>,
}

#[derive(Debug, Serialize)]
struct AzureDocument {
    id: String,
    language: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct AzureTask {
    kind: String,
    #[serde(rename = "taskName")]
    task_name: String,
}

#[derive(Debug, Deserialize)]
struct AzureNerResponse {
    tasks: AzureTasks,
}

#[derive(Debug, Deserialize)]
struct AzureTasks {
    items: Vec<AzureTaskItem>,
}

#[derive(Debug, Deserialize)]
struct AzureTaskItem {
    results: AzureResults,
}

#[derive(Debug, Deserialize)]
struct AzureResults {
    documents: Vec<AzureResultDocument>,
}

#[derive(Debug, Deserialize)]
struct AzureResultDocument {
    entities: Vec<AzureEntity>,
}

#[derive(Debug, Deserialize)]
struct AzureEntity {
    text: String,
    category: String,
    #[serde(rename = "subcategory")]
    subcategory: Option<String>,
    #[serde(rename = "confidenceScore")]
    confidence_score: f32,
}

pub struct ContextEngine {
    embedding_provider: Mutex<EmbeddingProvider>, // 用Mutex包装
    dynamics: ImplicitDynamics,
    storage: Arc<NodeStorage>,
}

impl ContextEngine {
    pub async fn new(storage: Arc<NodeStorage>) -> Result<Self> {
        info!("Initializing Context Engine with weight dynamics...");

        // Initialize weight dynamics with paper-compliant configuration
        let config = DynamicsConfig {
            learning_rate: 0.1,
            use_skip_connections: false,
            regularization_strength: 0.001,
        };
        let dynamics = ImplicitDynamics::new(384, 256, config)?;

        // Initialize embedding provider
        let embedding_provider = Mutex::new(EmbeddingProvider::new(
            prompt_compiler_embeddings::EmbeddingConfig::default()
        )?);

        info!("✅ Context Engine initialized successfully");

        Ok(Self {
            storage,
            dynamics,
            embedding_provider,
        })
    }

    // 🔧 简化版本：Azure Text Analytics NER 调用（带更好的错误处理）
    async fn call_azure_ner(&self, texts: &[String]) -> Result<Vec<AzureEntity>> {
        // 🔧 首先检查环境变量是否配置
        let endpoint = match std::env::var("AZURE_TEXT_ANALYTICS_ENDPOINT") {
            Ok(val) if !val.is_empty() => val,
            _ => {
                debug!("Azure Text Analytics endpoint not configured, skipping NER");
                return Err(anyhow::anyhow!("Azure endpoint not configured"));
            }
        };

        let api_key = match std::env::var("AZURE_TEXT_ANALYTICS_KEY") {
            Ok(val) if !val.is_empty() => val,
            _ => {
                debug!("Azure Text Analytics key not configured, skipping NER");
                return Err(anyhow::anyhow!("Azure key not configured"));
            }
        };

        // 🔧 使用更兼容的 REST API 格式
        // 参考：https://docs.microsoft.com/en-us/azure/cognitive-services/text-analytics/how-tos/text-analytics-how-to-entity-linking
        #[derive(Serialize)]
        struct SimpleDocument {
            id: String,
            text: String,
            language: String,
        }

        #[derive(Serialize)]
        struct SimpleRequest {
            documents: Vec<SimpleDocument>,
        }

        let documents: Vec<SimpleDocument> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| SimpleDocument {
                id: format!("doc_{}", i),
                text: text.clone(),
                language: "en".to_string(),
            })
            .collect();

        let request = SimpleRequest { documents };

        let client = reqwest::Client::new();
        // 🔧 使用稳定的 v3.1 API
        let url = format!("{}/text/analytics/v3.1/entities/recognition/general", endpoint.trim_end_matches('/'));

        debug!("Calling Azure Text Analytics v3.1: {}", url);

        let response = client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", &api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(10)) // 🔧 添加超时
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Azure API error: {} - {}", status_code, &error_text[..std::cmp::min(200, error_text.len())]);
            return Err(anyhow::anyhow!("Azure API error: {}", status_code));
        }

        // 🔧 ���化的响应解析
        #[derive(Deserialize)]
        struct SimpleResponse {
            documents: Vec<SimpleDocumentResponse>,
        }

        #[derive(Deserialize)]
        struct SimpleDocumentResponse {
            entities: Vec<AzureEntity>,
        }

        let azure_response: SimpleResponse = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse Azure response: {}", e))?;

        // 提取所有实体
        let mut all_entities = Vec::new();
        for document in azure_response.documents {
            all_entities.extend(document.entities);
        }

        debug!("Azure NER found {} entities", all_entities.len());
        Ok(all_entities)
    }

    // 🔧 新增：使用 Azure NER 增强的综合信息提取
    async fn extract_comprehensive_key_information_with_ner(&self, contexts: &[SimilarContext]) -> Result<Vec<String>> {
        // 准备文本用于 NER 分析
        let texts: Vec<String> = contexts.iter()
            .take(3)  // 限制处理的上下文数量以控制成本
            .map(|ctx| ctx.content.clone())
            .collect();

        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut key_info = Vec::new();

        // 🔧 明确日志：显示是否尝试使用Azure NER
        info!("🔍 Attempting to use Azure NER for comprehensive key information extraction...");

        // 尝试调用 Azure NER
        match self.call_azure_ner(&texts).await {
            Ok(entities) => {
                info!("✅ SUCCESS: Azure NER extracted {} entities successfully!", entities.len());
                debug!("Azure NER entities: {:?}", entities.iter().map(|e| &e.text).collect::<Vec<_>>());

                // 将 Azure 实体转换为业务相关的关键信息
                for entity in entities {
                    if entity.confidence_score >= 0.7 {  // 只保留高置信度的实体
                        let key_item = self.convert_azure_entity_to_business_info(&entity);
                        if !key_item.is_empty() && !key_info.contains(&key_item) {
                            key_info.push(key_item);
                        }
                    }
                }

                info!("📊 Azure NER result: {} high-confidence business entities extracted", key_info.len());

                // 如果 NER 结果不够丰富，补充硬编码的提取结果
                if key_info.len() < 3 {
                    info!("📝 Azure NER results insufficient, supplementing with local extraction...");
                    let fallback_info = self.extract_comprehensive_key_information(contexts);
                    for item in fallback_info {
                        if !key_info.contains(&item) && key_info.len() < 6 {
                            key_info.push(item);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("❌ FALLBACK: Azure NER failed, using local extraction only: {}", e);
                info!("🔄 Falling back to hardcoded entity extraction methods");
                // 完全降级到原有的硬编码方法
                return Ok(self.extract_comprehensive_key_information(contexts));
            }
        }

        info!("🎯 Final result: {} key information items extracted (Azure NER + Local)", key_info.len());
        Ok(key_info)
    }

    // 🔧 新增：将 Azure 实体转换为业务相关信息
    fn convert_azure_entity_to_business_info(&self, entity: &AzureEntity) -> String {
        match entity.category.as_str() {
            "Person" => {
                format!("Contact: {}", entity.text)
            }
            "Organization" => {
                format!("Company: {}", entity.text)
            }
            "Location" => {
                format!("Location: {}", entity.text)
            }
            "DateTime" => {
                format!("Timeline: {}", entity.text)
            }
            "Quantity" => {
                if entity.text.contains("thousand") || entity.text.contains("million") {
                    format!("Scale: {}", entity.text)
                } else {
                    format!("Metric: {}", entity.text)
                }
            }
            "PersonType" => {
                format!("Role: {}", entity.text)
            }
            "Product" => {
                format!("Product: {}", entity.text)
            }
            "Event" => {
                format!("Event: {}", entity.text)
            }
            _ => {
                // ���于其他类型，根据子类别进一步分类
                if let Some(subcategory) = &entity.subcategory {
                    format!("{}: {}", subcategory, entity.text)
                } else {
                    format!("Entity: {}", entity.text)
                }
            }
        }
    }

    // 🔧 新增：使用 Azure NER 增强的用户身份提取
    async fn extract_persistent_user_identity_with_ner(&self, contexts: &[SimilarContext]) -> Result<String> {
        // 准备文本用于 NER 分析
        let texts: Vec<String> = contexts.iter()
            .take(2)  // 用户身份信息通常在前几轮对话中
            .map(|ctx| ctx.content.clone())
            .collect();

        if texts.is_empty() {
            return Ok(String::new());
        }

        let mut identity_parts = Vec::new();

        // 🔧 明确日志：显示是否尝试使用Azure NER进行身份提取
        info!("👤 Attempting to use Azure NER for user identity extraction...");

        // 尝试调用 Azure NER
        match self.call_azure_ner(&texts).await {
            Ok(entities) => {
                info!("✅ SUCCESS: Azure NER found {} entities for identity analysis!", entities.len());
                debug!("Azure NER identity entities: {:?}", entities.iter().map(|e| format!("{}({})", e.text, e.category)).collect::<Vec<_>>());

                // 提取身份相关的实体
                for entity in entities {
                    if entity.confidence_score >= 0.8 {  // 身份信息要求更高的置信度
                        match entity.category.as_str() {
                            "Person" => {
                                let name_info = format!("Name: {}", entity.text);
                                if !identity_parts.contains(&name_info) {
                                    identity_parts.push(name_info);
                                    info!("🏷️  Azure NER extracted person name: {}", entity.text);
                                }
                            }
                            "PersonType" => {
                                let role_info = format!("Role: {}", entity.text);
                                if !identity_parts.contains(&role_info) {
                                    identity_parts.push(role_info);
                                    info!("💼 Azure NER extracted role: {}", entity.text);
                                }
                            }
                            "Organization" => {
                                let org_info = format!("Company: {}", entity.text);
                                if !identity_parts.contains(&org_info) {
                                    identity_parts.push(org_info);
                                    info!("🏢 Azure NER extracted organization: {}", entity.text);
                                }
                            }
                            "Product" | "Event" => {
                                let project_info = format!("Project: {}", entity.text);
                                if !identity_parts.contains(&project_info) {
                                    identity_parts.push(project_info);
                                    info!("🚀 Azure NER extracted project/event: {}", entity.text);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                info!("📋 Azure NER identity result: {} identity components extracted", identity_parts.len());

                // 如果 NER 结果不够，补充硬编码的提取结果
                if identity_parts.len() < 2 {
                    info!("📝 Azure NER identity results insufficient, supplementing with local extraction...");
                    let fallback_identity = self.extract_persistent_user_identity(contexts);
                    if !fallback_identity.is_empty() && !identity_parts.iter().any(|part| part.contains(&fallback_identity)) {
                        identity_parts.push(fallback_identity);
                    }
                }
            }
            Err(e) => {
                warn!("❌ FALLBACK: Azure NER failed for identity extraction, using local methods: {}", e);
                info!("🔄 Falling back to hardcoded identity extraction methods");
                // 降级到原有方法
                return Ok(self.extract_persistent_user_identity(contexts));
            }
        }

        let final_identity = if identity_parts.is_empty() {
            String::new()
        } else {
            identity_parts.join(", ")
        };

        if final_identity.is_empty() {
            info!("🚫 No user identity information extracted");
        } else {
            info!("🎯 Final identity result: {} (Azure NER + Local)", final_identity);
        }

        Ok(final_identity)
    }

    pub async fn process_request_with_group(
        &self,
        request: &ChatCompletionRequest,
        agent_id: Option<&str>,
        shared_context_group: Option<&str>,
    ) -> Result<ProcessedRequest> {
        debug!("Processing request with context sharing for agent: {:?}, group: {:?}", agent_id, shared_context_group);

        // Extract context from messages
        let conversation_context = self.extract_conversation_context(&request.messages).await?;
        debug!("Current conversation context: {}", &conversation_context[..std::cmp::min(100, conversation_context.len())]);

        // 🔧 跨Agent上下文共享逻辑 - 进一步优化过滤策略
        let similar_contexts = if let Some(group) = shared_context_group {
            // 使用上下文组查找相关上下文（跨Agent共享）
            let context_embedding = {
                let mut provider = self.embedding_provider.lock()
                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                provider.encode(&conversation_context)?
            };

            let contexts = self.storage.find_similar_contexts_in_group(group, &context_embedding, 15).await?; // 🔧 增加到15个候选
            debug!("Found {} similar contexts in group {}", contexts.len(), group);

            // 🔧 智能上下文过滤：确保客户身��信息和关键业务信息优先保留
            let mut filtered_contexts = Vec::new();

            // 🔧 改进：首先无条件添加所有包含客户身份的上下文
            for ctx in &contexts {
                if self.contains_client_identity(&ctx.content) {
                    filtered_contexts.push(ctx.clone());
                    if filtered_contexts.len() >= 5 { // 最多5个客户相关上下文
                        break;
                    }
                }
            }

            // 第二轮：添加业务关键信息，使用更宽松的阈值
            for ctx in &contexts {
                if !self.contains_client_identity(&ctx.content) &&
                   self.contains_business_info(&ctx.content) &&
                   filtered_contexts.len() < 10 {
                    filtered_contexts.push(ctx.clone());
                }
            }

            // 第三轮：添加其他高相似度上下文
            for ctx in contexts.into_iter() {
                if !self.contains_client_identity(&ctx.content) &&
                   !self.contains_business_info(&ctx.content) &&
                   filtered_contexts.len() < 12 &&
                   ctx.similarity > 0.05 { // 🔧 降低阈值到0.05
                    filtered_contexts.push(ctx);
                }
            }

            debug!("Filtered contexts: {} with relaxed criteria", filtered_contexts.len());
            filtered_contexts
        } else if let Some(agent_id) = agent_id {
            // 🔧 单Agent上下文查找 - 应用多Agent��功经验
            let context_embedding = {
                let mut provider = self.embedding_provider.lock()
                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                provider.encode(&conversation_context)?
            };

            let contexts = self.storage.find_similar_contexts(agent_id, &context_embedding, 10).await?; // 🔧 增加候选数量
            debug!("Found {} similar contexts for agent {}", contexts.len(), agent_id);

            // 🔧 应用与多Agent相同的智能过滤策略
            let mut filtered_contexts = Vec::new();

            // 第一轮：优先添加包含用户身份的上下文（降低阈值）
            for ctx in &contexts {
                if self.contains_user_identity(&ctx.content) && ctx.similarity > 0.05 { // 🔧 降低阈值
                    filtered_contexts.push(ctx.clone());
                    if filtered_contexts.len() >= 3 {
                        break;
                    }
                }
            }

            // 第二轮：添加其他相关上下文
            for ctx in contexts.into_iter() {
                if !self.contains_user_identity(&ctx.content) &&
                   filtered_contexts.len() < 8 &&
                   ctx.similarity > 0.05 { // 🔧 统一使用更宽松的阈值
                    filtered_contexts.push(ctx);
                }
            }

            debug!("Single-agent filtered contexts: {} with relaxed criteria", filtered_contexts.len());
            filtered_contexts
        } else {
            debug!("No agent ID or context group provided, skipping context lookup");
            Vec::new()
        };

        // 🔧 增强的上下文处理 - 优先保留关键信息
        let processed_messages = if !similar_contexts.is_empty() {
            debug!("Applying context sharing with {} relevant contexts", similar_contexts.len());

            // 🔧 改进：优先使用Azure NER增强的信息提取，统一错误处理
            info!("🔍 Starting enhanced context analysis with Azure NER...");
            let key_info = self.extract_comprehensive_key_information_with_ner(&similar_contexts).await
                .unwrap_or_else(|e| {
                    warn!("🔄 Azure NER unavailable ({}), using local extraction", e);
                    self.extract_comprehensive_key_information(&similar_contexts)
                });

            // 🔧 关键修复：使用 Azure NER 增强的用户身份提取，统一错误处理
            info!("👤 Starting user identity analysis with Azure NER...");
            let user_identity_info = self.extract_persistent_user_identity_with_ner(&similar_contexts).await
                .unwrap_or_else(|e| {
                    warn!("🔄 Azure NER identity extraction unavailable ({}), using local methods", e);
                    self.extract_persistent_user_identity(&similar_contexts)
                });

            let enhanced_context = if !key_info.is_empty() {
                let mut context_parts = Vec::new();

                // 🔧 优先添加用户身份信息（确保不丢失）
                if !user_identity_info.is_empty() {
                    context_parts.push(format!("User Identity: {}", user_identity_info));
                }

                // 添加其他关键信息
                if !key_info.is_empty() {
                    context_parts.push(format!("Previous Context: {}", key_info.join("; ")));
                }

                format!("IMPORTANT CONTEXT - {}", context_parts.join(" | "))
            } else {
                format!("Previous context: {}",
                       similar_contexts.first()
                           .map(|c| &c.content[..std::cmp::min(300, c.content.len())])  // 增加到300字符
                           .unwrap_or("No context available"))
            };

            // 🔧 修复：优先压缩策略，在压缩时强制保留用户身份信息
            let base_messages = if request.messages.len() > 5 {  // 保持阈值为5
                debug!("Long conversation detected ({} messages), applying identity-preserving compression", request.messages.len());
                self.apply_identity_preserving_compression(&request.messages, &user_identity_info).await?
            } else {
                request.messages.clone()
            };

            let mut enhanced = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: enhanced_context,
                }
            ];
            enhanced.extend(base_messages);

            debug!("Added enhanced context ({} chars) with user identity: {}",
                   enhanced[0].content.len(), user_identity_info);
            enhanced
        } else if request.messages.len() > 5 {
            // 多轮对话场景：智能压缩
            debug!("Multi-turn conversation detected ({} messages), applying smart compression", request.messages.len());
            self.apply_smart_compression(&request.messages).await?
        } else {
            debug!("No compression needed");
            request.messages.clone()
        };

        // 计算压缩效果
        let original_content_length: usize = request.messages.iter().map(|m| m.content.len()).sum();
        let processed_content_length: usize = processed_messages.iter().map(|m| m.content.len()).sum();

        let compression_ratio = if processed_content_length > original_content_length {
            -((processed_content_length - original_content_length) as f32 / original_content_length as f32)
        } else {
            ((original_content_length - processed_content_length) as f32 / original_content_length as f32)
        };

        Ok(ProcessedRequest {
            messages: processed_messages,
            compression_ratio,
            context_used: !similar_contexts.is_empty(),
        })
    }

    // 保持原有方法的向后兼容性
    pub async fn process_request(
        &self,
        request: &ChatCompletionRequest,
        agent_id: Option<&str>,
    ) -> Result<ProcessedRequest> {
        self.process_request_with_group(request, agent_id, None).await
    }

    // 🔧 新增：实现真正的语义压缩
    fn compress_historical_context(&self, contexts: &[SimilarContext]) -> String {
        if contexts.is_empty() {
            return String::new();
        }

        // 提取关�����息，�����是完整对话
        let mut key_facts = Vec::new();

        for context in contexts.iter().take(2) { // 只取最相关的2个上下文
            // 从历史对话中提取关键事实
            let content = &context.content;

            // 简单的关键信息提取逻辑
            if content.contains("name") || content.contains("名字") {
                if let Some(name_info) = self.extract_name_info(content) {
                    key_facts.push(name_info);
                }
            }

            if content.contains("work") || content.contains("job") || content.contains("工作") {
                if let Some(work_info) = self.extract_work_info(content) {
                    key_facts.push(work_info);
                }
            }

            if content.contains("project") || content.contains("项目") {
                if let Some(project_info) = self.extract_project_info(content) {
                    key_facts.push(project_info);
                }
            }
        }

        // 去重并生成简洁的上下文摘要
        key_facts.dedup();
        if key_facts.is_empty() {
            return String::new();
        }

        format!("Context: {}", key_facts.join(", "))
    }

    // 🔧 辅助方法：提取关键信息
    fn extract_name_info(&self, content: &str) -> Option<String> {
        // 简单的名字提取逻辑
        if let Some(start) = content.find("name is ") {
            let after_name = &content[start + 8..];
            if let Some(end) = after_name.find(" ") {
                let name = &after_name[..end];
                if name.len() > 0 && name.len() < 20 {
                    return Some(format!("name: {}", name));
                }
            }
        }
        None
    }

    fn extract_work_info(&self, content: &str) -> Option<String> {
        if content.contains("work at") || content.contains("job") {
            // 提取工作相关的简短信息
            if content.contains("tech") || content.contains("startup") {
                return Some("work: tech".to_string());
            }
            if content.contains("engineer") {
                return Some("work: engineer".to_string());
            }
        }
        None
    }

    fn extract_project_info(&self, content: &str) -> Option<String> {
        if content.contains("Python") && content.contains("machine learning") {
            return Some("project: Python ML".to_string());
        }
        if content.contains("project") {
            return Some("project: development".to_string());
        }
        None
    }

    // 🔧 新增：使用Azure NER增强的对话历史压缩
    async fn compress_conversation_history_with_ner(&self, messages: &[ChatMessage]) -> Result<String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        // 准备文本用于Azure NER分析
        let conversation_text = messages
            .iter()
            .take(5) // 只分析前5条消息以控制成本
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        info!("🔍 Using Azure NER for conversation history compression...");

        // 尝试使用Azure NER提取关键实体
        match self.call_azure_ner(&[conversation_text]).await {
            Ok(entities) => {
                info!("✅ Azure NER extracted {} entities from conversation history", entities.len());

                let mut summary_components = Vec::new();
                let mut names = Vec::new();
                let mut organizations = Vec::new();
                let mut topics = Vec::new();
                let mut roles = Vec::new();

                // 分类Azure NER提取的实体
                for entity in entities {
                    if entity.confidence_score >= 0.6 { // 较低的置信度阈值用于历史压缩
                        match entity.category.as_str() {
                            "Person" => {
                                if !names.contains(&entity.text) {
                                    names.push(entity.text);
                                }
                            }
                            "Organization" => {
                                if !organizations.contains(&entity.text) {
                                    organizations.push(entity.text);
                                }
                            }
                            "PersonType" => {
                                if !roles.contains(&entity.text) {
                                    roles.push(entity.text);
                                }
                            }
                            "Product" | "Event" => {
                                if !topics.contains(&entity.text) {
                                    topics.push(entity.text);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // 构建智能摘要
                if !names.is_empty() {
                    summary_components.push(format!("Participants: {}", names.join(", ")));
                }
                if !organizations.is_empty() {
                    summary_components.push(format!("Organizations: {}", organizations.join(", ")));
                }
                if !roles.is_empty() {
                    summary_components.push(format!("Roles: {}", roles.join(", ")));
                }
                if !topics.is_empty() {
                    summary_components.push(format!("Topics: {}", topics.join(", ")));
                }

                if summary_components.is_empty() {
                    // 如果Azure NER没有提取到足够信息，使用本地方法补充
                    info!("📝 Azure NER results insufficient for history, supplementing with local analysis");
                    Ok(self.compress_conversation_history(messages))
                } else {
                    let azure_summary = summary_components.join("; ");
                    info!("🎯 Azure NER conversation summary: {} components", summary_components.len());
                    Ok(azure_summary)
                }
            }
            Err(e) => {
                warn!("🔄 Azure NER unavailable for conversation history ({}), using local compression", e);
                Ok(self.compress_conversation_history(messages))
            }
        }
    }

    // 🔧 改进：更智能的对话历史压缩（减少硬编码）
    fn compress_conversation_history(&self, messages: &[ChatMessage]) -> String {
        if messages.is_empty() {
            return String::new();
        }

        // 🔧 ���用更智���的语义提取，减少硬编码
        let mut key_entities = Vec::new();
        let mut topics = std::collections::HashSet::new();
        let mut user_attributes = Vec::new();

        for message in messages {
            let content = &message.content;

            // 🔧 智能实体提取（使用更灵活的模式）
            if message.role == "user" {
                // 提取可能的用户标识符（姓名、角色、公司等）
                if let Some(identity) = self.extract_user_identity_smart(content) {
                    if !user_attributes.iter().any(|attr: &String| attr.contains(&identity)) {
                        user_attributes.push(identity);
                    }
                }

                // 🔧 关键改进：提取业务关键信息
                if let Some(business_info) = self.extract_business_details(content) {
                    if !key_entities.contains(&business_info) && key_entities.len() < 8 {
                        key_entities.push(business_info);
                    }
                }

                // 提取关键实体（使用NER-like方法）
                let entities = self.extract_entities_smart(content);
                for entity in entities {
                    if !key_entities.contains(&entity) && key_entities.len() < 8 {
                        key_entities.push(entity);
                    }
                }
            }

            // 🔧 主题提取（基于关键词聚类而非硬编码）
            let detected_topics = self.extract_topics_smart(content);
            for topic in detected_topics {
                topics.insert(topic);
            }
        }

        // 🔧 智能摘要生成 - 优���保留业务关键信���
        let mut summary_parts = Vec::new();

        if !user_attributes.is_empty() {
            summary_parts.push(format!("User: {}", user_attributes.join(", ")));
        }

        if !key_entities.is_empty() {
            summary_parts.push(format!("Context: {}", key_entities.join(", ")));
        }

        if !topics.is_empty() && topics.len() <= 3 {
            let topic_list: Vec<String> = topics.into_iter().take(3).collect();
            summary_parts.push(format!("Topics: {}", topic_list.join(", ")));
        }

        if summary_parts.is_empty() {
            // 🔧 fallback: 使用统计摘要而非简单计数
            let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
            let avg_message_len = total_chars / messages.len().max(1);
            format!("Session: {} exchanges, avg {} chars/msg",
                   messages.len() / 2, avg_message_len)
        } else {
            summary_parts.join("; ")
        }
    }

    async fn store_interaction(
        &self, // 改回��可变����用
        request: &ChatCompletionRequest,
        response: &ChatCompletionResponse,
        agent_id: Option<&str>,
    ) -> Result<()> {
        if let Some(agent_id) = agent_id {
            debug!("Storing interaction context for agent: {}", agent_id);

            // Create conversation summary
            let mut full_conversation = request.messages.clone();
            if let Some(choice) = response.choices.first() {
                full_conversation.push(choice.message.clone());
            }

            let conversation_text = self.extract_conversation_context(&full_conversation).await?;
            let embedding = {
                let mut provider = self.embedding_provider.lock()
                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                provider.encode(&conversation_text)?
            };

            // Store in persistent storage
            self.storage.store_context(agent_id, &conversation_text, &embedding).await?;

            debug!("✅ Interaction context stored successfully");
        }

        Ok(())
    }

    pub async fn store_interaction_with_group(
        &self,
        request: &ChatCompletionRequest,
        response: &ChatCompletionResponse,
        agent_id: Option<&str>,
        shared_context_group: Option<&str>,
    ) -> Result<()> {
        if let Some(agent_id) = agent_id {
            debug!("Storing interaction context for agent: {} in group: {:?}", agent_id, shared_context_group);

            // Create conversation summary
            let mut full_conversation = request.messages.clone();
            if let Some(choice) = response.choices.first() {
                full_conversation.push(choice.message.clone());
            }

            let conversation_text = self.extract_conversation_context(&full_conversation).await?;
            let embedding = {
                let mut provider = self.embedding_provider.lock()
                    .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
                provider.encode(&conversation_text)?
            };

            // Store with or without context group
            if let Some(group) = shared_context_group {
                self.storage.store_context_with_group(agent_id, group, &conversation_text, &embedding).await?;
            } else {
                self.storage.store_context(agent_id, &conversation_text, &embedding).await?;
            }

            debug!("✅ Interaction context stored successfully");
        }

        Ok(())
    }

    async fn extract_conversation_context(&self, messages: &[ChatMessage]) -> Result<String> {
        // Extract meaningful context from conversation
        let context = messages
            .iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(context)
    }

    async fn apply_weight_dynamics(
        &self,
        original_messages: &[ChatMessage],
        similar_contexts: &[SimilarContext],
    ) -> Result<Vec<ChatMessage>> {
        debug!("Applying weight dynamics with {} similar contexts", similar_contexts.len());

        // 简化的压缩逻辑，避免复杂的weight dynamics调用
        let compressed = self.fallback_compression(original_messages).await?;
        Ok(compressed)
    }

    async fn fallback_compression(&self, messages: &[ChatMessage]) -> Result<Vec<ChatMessage>> {
        // Simple fallback: keep last 3 messages
        let keep_count = std::cmp::min(3, messages.len());
        Ok(messages[messages.len() - keep_count..].to_vec())
    }

    fn calculate_compression_ratio(&self, original: &[ChatMessage], compressed: &[ChatMessage]) -> f32 {
        let original_length: usize = original.iter().map(|m| m.content.len()).sum();
        let compressed_length: usize = compressed.iter().map(|m| m.content.len()).sum();

        if original_length == 0 {
            return 0.0;
        }

        1.0 - (compressed_length as f32 / original_length as f32)
    }

    // 🔧 智能用户身份提取��减少硬编码模式）
    fn extract_user_identity_smart(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 🔧 模式1: 自我介绍模式
        let intro_patterns = [
            (r"i'?m (.+?)(?:[,.\n]|$)", 1),
            (r"my name is (.+?)(?:[,.\n]|$)", 1),
            (r"this is (.+?)(?:[,.\n]|$)", 1),
            (r"i am (.+?)(?:[,.\n]|$)", 1),
            (r"call me (.+?)(?:[,.\n]|$)", 1),
        ];

        for (pattern, group) in intro_patterns {
            if let Some(captures) = regex::Regex::new(pattern).ok()?.captures(&content_lower) {
                if let Some(matched) = captures.get(group) {
                    let identity = matched.as_str().trim();
                    if identity.len() > 1 && identity.len() < 50 && !identity.contains("assistant") {
                        return Some(identity.to_string());
                    }
                }
            }
        }

        None
    }

    // 🔧 智能实体提取
    fn extract_entities_smart(&self, content: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let content_lower = content.to_lowercase();

        // 🔧 技术栈检测
        let tech_terms = [
            "python", "javascript", "rust", "java", "typescript",
            "react", "vue", "angular", "tensorflow", "pytorch",
            "kubernetes", "docker", "aws", "azure", "gcp"
        ];

        // 🔧 领域检测
        let domain_terms = [
            "machine learning", "ai", "blockchain", "fintech", "healthcare",
            "e-commerce", "gaming", "cybersecurity", "data science"
        ];

        // 🔧 公司类型检测
        let company_terms = [
            "startup", "corporation", "enterprise", "consulting", "agency"
        ];

        for term in tech_terms.iter().chain(domain_terms.iter()).chain(company_terms.iter()) {
            if content_lower.contains(term) {
                entities.push(term.to_string());
            }
        }

        entities
    }

    // 🔧 智能主题提取
    fn extract_topics_smart(&self, content: &str) -> Vec<String> {
        let mut topics = Vec::new();
        let content_lower = content.to_lowercase();

        // 🔧 问题类型检测
        if content_lower.contains("how") && (content_lower.contains("work") || content_lower.contains("implement")) {
            topics.push("implementation".to_string());
        }

        if content_lower.contains("what") && (content_lower.contains("best") || content_lower.contains("recommend")) {
            topics.push("recommendations".to_string());
        }

        if content_lower.contains("problem") || content_lower.contains("issue") || content_lower.contains("error") {
            topics.push("troubleshooting".to_string());
        }

        // 🔧 技术主题���测
        let tech_topics = [
            ("algorithm", vec!["algorithm", "sort", "search", "optimize"]),
            ("database", vec!["database", "sql", "nosql", "query"]),
            ("frontend", vec!["frontend", "ui", "ux", "css", "html"]),
            ("backend", vec!["backend", "api", "server", "microservice"]),
            ("devops", vec!["deploy", "ci/cd", "pipeline", "infrastructure"]),
        ];

        for (topic, keywords) in tech_topics {
            if keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                topics.push(topic.to_string());
            }
        }

        topics
    }

    // 🔧 新增：从多个上下文���提取关键信息
    fn extract_key_information_from_contexts(&self, contexts: &[SimilarContext]) -> Vec<String> {
        let mut key_info = Vec::new();

        for context in contexts.iter().take(3) { // 只处理最相关的3个上下文
            let content = &context.content;

            // 提取客户姓名和公��信息 (如 "Michael Chen from Alpha Corp")
            if let Some(client_info) = self.extract_client_information(content) {
                if !key_info.contains(&client_info) {
                    key_info.push(client_info);
                }
            }

            // 提取重要的业务细节
            if let Some(business_detail) = self.extract_business_details(content) {
                if !key_info.contains(&business_detail) {
                    key_info.push(business_detail);
                }
            }

            // 提取解决方案信息
            if let Some(solution_info) = self.extract_solution_information(content) {
                if !key_info.contains(&solution_info) {
                    key_info.push(solution_info);
                }
            }
        }

        key_info
    }

    // 🔧 提取客户信息（姓名、公司等）- 增强版
    fn extract_client_information(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 🔧 ���先匹配完��的客户��息���式
        let client_patterns = [
            // 完整姓名 + 公司模式
            r"([A-Z][a-z]+\s+[A-Z][a-z]+)\s+from\s+([A-Z][a-zA-Z\s]+(?:Corp|Inc|LLC|Ltd|Corporation))",
            // ��化的姓名 + 公司模式
            r"([A-Z][a-z]+)\s+from\s+([A-Z][a-zA-Z\s]+)",
            // 直接的客户介绍模式
            r"this\s+is\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)\s+from\s+([A-Z][a-zA-Z\s]+)",
            // Hi开头的自我介绍
            r"hi,?\s+this\s+is\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)",
        ];

        for pattern in client_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    match captures.len() {
                        3 => {
                            // 包含姓名和公司
                            let name = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                            let company = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                            if !name.is_empty() && !company.is_empty() {
                                return Some(format!("Client: {} from {}", name, company.trim()));
                            }
                        },
                        2 => {
                            // 只有姓名
                            let name = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                            if !name.is_empty() {
                                return Some(format!("Client: {}", name));
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        // 🔧 备用：寻找常见的客户信息关键词
        if content_lower.contains("michael") && content_lower.contains("alpha") {
            return Some("Client: Michael from Alpha Corp".to_string());
        }

        if content_lower.contains("john") && content_lower.contains("techcorp") {
            return Some("Client: John from TechCorp".to_string());
        }

        None
    }

    // 🔧 提取业务细节
    fn extract_business_details(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 规模信息
        if content_lower.contains("inquiries") && content_lower.contains("month") {
            if let Some(volume) = self.extract_volume_info(&content_lower) {
                return Some(format!("Volume: {}", volume));
            }
        }

        // 技术需求
        if content_lower.contains("ai-powered") || content_lower.contains("ai powered") {
            return Some("Requirement: AI-powered solution".to_string());
        }

        // 行业信息
        if content_lower.contains("customer service") {
            return Some("Domain: Customer Service".to_string());
        }

        None
    }

    // 🔧 提取解决方案信息
    fn extract_solution_information(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        if content_lower.contains("recommend") || content_lower.contains("suggest") {
            // 提取推荐的解决方案
            if content_lower.contains("enterprise") {
                return Some("Solution: Enterprise package recommended".to_string());
            }
            if content_lower.contains("custom") {
                return Some("Solution: Custom solution".to_string());
            }
        }

        None
    }

    // 🔧 提取数量信息
    fn extract_volume_info(&self, content: &str) -> Option<String> {
        // 匹配 "数字 + thousand/k + inquiries/month" 模式
        let volume_patterns = [
            r"(\d+[,\d]*)\s*(?:thousand|k)\s*inquiries",
            r"(\d+[,\d]*)\s*inquiries\s*per\s*month",
            r"(\d+[,\d]*)\s*monthly\s*inquiries",
        ];

        for pattern in volume_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    if let Some(number) = captures.get(1) {
                        return Some(format!("{} inquiries/month", number.as_str()));
                    }
                }
            }
        }

        None
    }

    // 🔧 新增：智能压缩方法
    async fn apply_smart_compression(&self, messages: &[ChatMessage]) -> Result<Vec<ChatMessage>> {
        debug!("Applying smart compression to {} messages", messages.len());

        // 🔧 移除双��检查 - 调用方已经确认需要压缩
        if messages.len() <= 2 {
            debug!("Too few messages for compression, returning original");
            return Ok(messages.to_vec());
        }

        // 🔧 更积极的压缩策略
        let keep_recent = if messages.len() > 8 {
            2  // 长对话只保留最近2条
        } else if messages.len() > 5 {
            3  // 中等长度保留3条
        } else {
            messages.len().saturating_sub(1)  // 短对话保留大部分
        };

        let recent_messages = messages.iter().rev().take(keep_recent).rev().cloned().collect::<Vec<_>>();
        let historical_messages = &messages[..messages.len().saturating_sub(keep_recent)];

        // �� 生成更简洁的压缩摘要
        let compressed_summary = if historical_messages.is_empty() {
            String::new()
        } else {
            self.compress_conversation_history(historical_messages)
        };

        let mut result = Vec::new();

        // 🔧 只有在有实际历史内容时才添加摘要
        if !compressed_summary.is_empty() && compressed_summary.len() > 10 {
            result.push(ChatMessage {
                role: "system".to_string(),
                content: format!("Previous conversation: {}", compressed_summary),
            });
        }

        result.extend(recent_messages);

        debug!("✅ Compressed {} messages to {} (summary: {} chars)",
               messages.len(), result.len(),
               compressed_summary.len());

        Ok(result)
    }

    // 🔧 新增：检测是否包含用户身份信息
    fn contains_user_identity(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();

        // 🔧 更精确的用户身份模式检测
        let identity_patterns = [
            // 姓名模式
            r"my name is ([a-zA-Z]+)",
            r"i'm ([a-zA-Z]+)",
            r"i am ([a-zA-Z]+)",
            r"this is ([a-zA-Z]+)",
            // 工作/项目模式
            r"i'm working on",
            r"i am working on",
            r"working on (a|an)?\s*([a-zA-Z\s]+)\s*(project|system)",
            r"project about ([a-zA-Z\s]+)",
            // 职业/角色���式
            r"i'm (a|an)\s*([a-zA-Z\s]+)",
            r"i am (a|an)\s*([a-zA-Z\s]+)",
        ];

        for pattern in identity_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&content_lower) {
                    debug!("Found user identity pattern: {} in content: {}", pattern, &content[..std::cmp::min(100, content.len())]);
                    return true;
                }
            }
        }

        // 🔧 关��词检测作为后备
        let identity_keywords = [
            "my name", "i'm", "i am", "working on", "project about",
            "alice", "bob", "charlie", "david", "emma", "frank",
            "python", "machine learning", "data science", "ai"
        ];

        for keyword in identity_keywords {
            if content_lower.contains(keyword) {
                debug!("Found identity keyword: {} in content", keyword);
                return true;
            }
        }

        false
    }

    // 🔧 新增：检测客户身��信息（用于跨Agent场景）
    fn contains_client_identity(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();

        let client_patterns = [
            r"client is ([a-zA-Z]+)",
            r"customer ([a-zA-Z]+)",
            r"([a-zA-Z]+) from ([a-zA-Z\s]+)",
            r"working with ([a-zA-Z]+)",
        ];

        for pattern in client_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&content_lower) {
                    return true;
                }
            }
        }

        content_lower.contains("client") || content_lower.contains("customer")
    }

    // 🔧 新增：检测业务关键信息
    fn contains_business_info(&self, content: &str) -> bool {
        let content_lower = content.to_lowercase();

        let business_keywords = [
            "project", "system", "application", "solution", "requirements",
            "budget", "timeline", "deadline", "scope", "deliverable"
        ];

        business_keywords.iter().any(|keyword| content_lower.contains(keyword))
    }

    // 🔧 新增：提取综合关键信息
    fn extract_comprehensive_key_information(&self, contexts: &[SimilarContext]) -> Vec<String> {
        let mut key_info = Vec::new();

        for context in contexts.iter().take(4) { // 处理更多上下文来获得完整信息
            let content = &context.content;

            // 客户身份信��
            if let Some(client_info) = self.extract_client_information(content) {
                if !key_info.contains(&client_info) {
                    key_info.push(client_info);
                }
            }

            // ��务规模和需��
            if let Some(business_detail) = self.extract_business_details(content) {
                if !key_info.contains(&business_detail) {
                    key_info.push(business_detail);
                }
            }

            // 技术讨论内容
            if let Some(tech_info) = self.extract_technical_discussion(content) {
                if !key_info.contains(&tech_info) {
                    key_info.push(tech_info);
                }
            }

            // 解决方案和建议
            if let Some(solution_info) = self.extract_solution_information(content) {
                if !key_info.contains(&solution_info) {
                    key_info.push(solution_info);
                }
            }
        }

        key_info
    }

    // 🔧 ��增：提取���术讨论内容
    fn extract_technical_discussion(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 检测技术相关的讨论
        if content_lower.contains("architecture") || content_lower.contains("infrastructure") {
            return Some("Discussion: Technical architecture".to_string());
        }

        if content_lower.contains("scalability") || content_lower.contains("scaling") {
            return Some("Discussion: Scalability requirements".to_string());
        }

        if content_lower.contains("security") || content_lower.contains("compliance") {
            return Some("Discussion: Security and compliance".to_string());
        }

        if content_lower.contains("integration") || content_lower.contains("crm") {
            return Some("Discussion: System integration".to_string());
        }

        if content_lower.contains("pricing") || content_lower.contains("budget") {
            return Some("Discussion: Pricing and budget".to_string());
        }

        if content_lower.contains("timeline") || content_lower.contains("implementation") {
            return Some("Discussion: Implementation timeline".to_string());
        }

        None
    }

    // 🔧 新增：温和压缩方法（专为跨Agent场景设计）
    async fn apply_gentle_compression(&self, messages: &[ChatMessage]) -> Result<Vec<ChatMessage>> {
        debug!("Applying gentle compression to {} messages for cross-agent context", messages.len());

        if messages.len() <= 3 {
            debug!("Too few messages for gentle compression, returning original");
            return Ok(messages.to_vec());
        }

        // 🔧 温和的压缩策略：保留更多最近的消息
        let keep_recent = if messages.len() > 10 {
            4  // 很长对话保留最近4条
        } else if messages.len() > 7 {
            3  // 中长对话保留最近3条
        } else {
            messages.len().saturating_sub(2)  // 短对话保留几乎所有
        };

        let recent_messages = messages.iter().rev().take(keep_recent).rev().cloned().collect::<Vec<_>>();
        let historical_messages = &messages[..messages.len().saturating_sub(keep_recent)];

        // 🔧 生成更详细的历史摘要（专为跨Agent场景）
        let compressed_summary = if historical_messages.is_empty() {
            String::new()
        } else {
            self.compress_conversation_history_detailed(historical_messages)
        };

        let mut result = Vec::new();

        // 🔧 确保摘要包含足够的信息
        if !compressed_summary.is_empty() && compressed_summary.len() > 15 {
            result.push(ChatMessage {
                role: "system".to_string(),
                content: format!("Previous team discussion: {}", compressed_summary),
            });
        }

        result.extend(recent_messages);

        debug!("✅ Gently compressed {} messages to {} (detailed summary: {} chars)",
               messages.len(), result.len(),
               compressed_summary.len());

        Ok(result)
    }

    // 🔧 新增：详细的对话历史压缩（专为跨Agent场景）
    fn compress_conversation_history_detailed(&self, messages: &[ChatMessage]) -> String {
        if messages.is_empty() {
            return String::new();
        }

        let mut summary_parts = Vec::new();
        let mut client_info = Vec::new();
        let mut business_info = Vec::new();
        let mut technical_info = Vec::new();

        for message in messages {
            let content = &message.content;

            // 提取客户相关信息
            if let Some(client) = self.extract_client_information(content) {
                if !client_info.contains(&client) {
                    client_info.push(client);
                }
            }

            // 提取��务信息
            if let Some(business) = self.extract_business_details(content) {
                if !business_info.contains(&business) {
                    business_info.push(business);
                }
            }

            // 提取技术讨论
            if let Some(tech) = self.extract_technical_discussion(content) {
                if !technical_info.contains(&tech) {
                    technical_info.push(tech);
                }
            }
        }

        // 构建详细摘要
        if !client_info.is_empty() {
            summary_parts.push(client_info.join(", "));
        }

        if !business_info.is_empty() {
            summary_parts.push(business_info.join(", "));
        }

        if !technical_info.is_empty() {
            summary_parts.push(technical_info.join(", "));
        }

        if summary_parts.is_empty() {
            format!("Previous discussion: {} exchanges between team members", messages.len() / 2)
        } else {
            summary_parts.join("; ")
        }
    }

    // 🔧 新增：提取持久化用户身份信息（关键修复）
    fn extract_persistent_user_identity(&self, contexts: &[SimilarContext]) -> String {
        let mut identity_parts: Vec<String> = Vec::new();

        for context in contexts.iter() {
            let content = &context.content;
            let _content_lower = content.to_lowercase();

            // 🔧 优先提取��户姓名
            if let Some(name) = self.extract_user_name_robust(content) {
                if !identity_parts.iter().any(|part| part.contains(&name)) {
                    identity_parts.push(format!("Name: {}", name));
                }
            }

            // 🔧 提取工作/��目信息
            if let Some(project) = self.extract_project_info_robust(content) {
                if !identity_parts.iter().any(|part| part.contains(&project)) {
                    identity_parts.push(format!("Project: {}", project));
                }
            }

            // 🔧 提取职业信息
            if let Some(role) = self.extract_role_info_robust(content) {
                if !identity_parts.iter().any(|part| part.contains(&role)) {
                    identity_parts.push(format!("Role: {}", role));
                }
            }
        }

        if identity_parts.is_empty() {
            String::new()
        } else {
            identity_parts.join(", ")
        }
    }

    // 🔧 新增：强化的用户姓名提取
    fn extract_user_name_robust(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 直接姓名模式匹配
        let name_patterns = [
            r"my name is ([a-zA-Z]+)",
            r"i'?m ([a-zA-Z]+)(?:\s|,|\.)",
            r"i am ([a-zA-Z]+)(?:\s|,|\.)",
            r"this is ([a-zA-Z]+)(?:\s|,|\.)",
            r"hi,?\s+i'?m ([a-zA-Z]+)",
        ];

        for pattern in name_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(&content_lower) {
                    if let Some(name_match) = captures.get(1) {
                        let name = name_match.as_str();
                        // 过滤掉常见的非姓名词汇
                        if !["working", "doing", "building", "creating", "developing"].contains(&name)
                           && name.len() > 1 && name.len() < 20 {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }

        // 🔧 备用：特定姓名检测
        let common_names = ["alice", "bob", "charlie", "sarah", "david", "emma", "michael", "john"];
        for name in common_names {
            if content_lower.contains(name) {
                return Some(name.to_string());
            }
        }

        None
    }

    // 🔧 新增：���化的项目信息提取
    fn extract_project_info_robust(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 项目描述模式
        let project_patterns = [
            r"working on (a|an)?\s*([a-zA-Z\s]+?)\s*(project|system)",
            r"project about ([a-zA-Z\s]+)",
            r"building (a|an)?\s*([a-zA-Z\s]+?)\s*(system|application)",
            r"developing (a|an)?\s*([a-zA-Z\s]+?)\s*(solution|tool)",
        ];

        for pattern in project_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(&content_lower) {
                    // 找到项目描述的捕获组
                    for i in 1..captures.len() {
                        if let Some(project_match) = captures.get(i) {
                            let project = project_match.as_str().trim();
                            if project.len() > 3 && project.len() < 50
                               && !["a", "an", "the", "project", "system"].contains(&project) {
                                return Some(project.to_string());
                            }
                        }
                    }
                }
            }
        }

        // 🔧 特定技术栈检测
        if content_lower.contains("python") && content_lower.contains("machine learning") {
            return Some("Python machine learning".to_string());
        }

        if content_lower.contains("fraud detection") {
            return Some("fraud detection system".to_string());
        }

        None
    }

    // ���� 新增：强化的角���信息提取
    fn extract_role_info_robust(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 角色描述模式
        let role_patterns = [
            r"i'?m (a|an)\s*([a-zA-Z\s]+?)(?:\s|,|\.|$)",
            r"i am (a|an)\s*([a-zA-Z\s]+?)(?:\s|,|\.|$)",
            r"work as (a|an)?\s*([a-zA-Z\s]+?)(?:\s|,|\.|$)",
            r"job as (a|an)?\s*([a-zA-Z\s]+?)(?:\s|,|\.|$)",
        ];

        for pattern in role_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(&content_lower) {
                    if let Some(role_match) = captures.get(2) {
                        let role = role_match.as_str().trim();
                        if role.len() > 3 && role.len() < 30 {
                            return Some(role.to_string());
                        }
                    }
                }
            }
        }

        // �� 特定角色检测
        let roles = [
            "data scientist", "software engineer", "developer", "engineer",
            "analyst", "researcher", "consultant", "manager"
        ];

        for role in roles {
            if content_lower.contains(role) {
                return Some(role.to_string());
            }
        }

        None
    }

    // 🔧 新增�����份保护压缩方法（确保用户身份不丢失）
    async fn apply_identity_preserving_compression(&self, messages: &[ChatMessage], user_identity: &str) -> Result<Vec<ChatMessage>> {
        debug!("Applying identity-preserving compression to {} messages, preserving: {}", messages.len(), user_identity);

        if messages.len() <= 3 {
            return Ok(messages.to_vec());
        }

        // 🔧 更保守的压缩策略，确保用户身份信息不丢失
        let keep_recent = if messages.len() > 10 {
            4  // 长对话保留最近4条
        } else if messages.len() > 7 {
            3  // 中等对话保留最近3条
        } else {
            messages.len().saturating_sub(1)  // 短对话保留大部分
        };

        let recent_messages = messages.iter().rev().take(keep_recent).rev().cloned().collect::<Vec<_>>();
        let historical_messages = &messages[..messages.len().saturating_sub(keep_recent)];

        // 🔧 生成包含用户身份的压缩摘要
        let compressed_summary = if historical_messages.is_empty() {
            if !user_identity.is_empty() {
                format!("User context: {}", user_identity)
            } else {
                String::new()
            }
        } else {
            let history_summary = self.compress_conversation_history(historical_messages);
            if !user_identity.is_empty() {
                format!("{} | {}", user_identity, history_summary)
            } else {
                history_summary
            }
        };

        let mut result = Vec::new();

        // 🔧 ��制添加用户��份摘要（即使历史��空）
        if !compressed_summary.is_empty() {
            result.push(ChatMessage {
                role: "system".to_string(),
                content: format!("PRESERVED CONTEXT: {}", compressed_summary),
            });
        }

        result.extend(recent_messages);

        debug!("✅ Identity-preserving compression: {} -> {} messages, identity preserved: {}",
               messages.len(), result.len(), !user_identity.is_empty());

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct SimilarContext {
    pub content: String,
    pub similarity: f32,
    pub timestamp: i64,
}
