// Context传递方式的工程实现
// 演示PC如何将压缩Context传递给LLM的不同方法

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio;

/// 🎯 方式一：普通API传递（最常见）
/// 将压缩Context作为prompt的一部分直接发送给LLM
pub struct StandardAPITransfer {
    pub llm_client: LLMClient,
}

impl StandardAPITransfer {
    /// 标准方式：将Context嵌入到prompt中
    pub async fn send_context_via_prompt(
        &self,
        compressed_context: &CompressedContext,
        user_query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("📤 Method 1: Standard API Transfer via Prompt");

        // 构建包含Context的完整prompt
        let full_prompt = self.build_contextualized_prompt(compressed_context, user_query);

        // 发送到LLM API
        let request = LLMRequest {
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant with access to user context.".to_string(),
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: full_prompt,
                }
            ],
            max_tokens: Some(500),
            temperature: Some(0.7),
        };

        println!("   📊 Prompt tokens: {} | Context tokens: {}",
                self.estimate_tokens(&full_prompt), compressed_context.compressed_tokens);

        let response = self.llm_client.chat_completion(request).await?;
        Ok(response.choices[0].message.content.clone())
    }

    /// 构建包含Context的prompt
    fn build_contextualized_prompt(&self, context: &CompressedContext, query: &str) -> String {
        format!(
            "Context Information:\n\
            - User Profile: {}\n\
            - Relevant History: {}\n\
            - Domain Knowledge: {}\n\
            - Personalization: {}\n\n\
            User Query: {}\n\n\
            Please provide a response considering the above context:",
            context.essential_user_info,
            context.relevant_history,
            context.task_knowledge,
            context.personalization_hints,
            query
        )
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }
}

/// 🎯 方式二：结构化API传递（OpenAI Functions/Tools）
/// 使用LLM的结构化输入能力，将Context作为工具调用参数
pub struct StructuredAPITransfer {
    pub llm_client: LLMClient,
}

impl StructuredAPITransfer {
    /// 结构化方式：使用Function Calling传递Context
    pub async fn send_context_via_functions(
        &self,
        compressed_context: &CompressedContext,
        user_query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("📤 Method 2: Structured API Transfer via Functions");

        let request = LLMRequest {
            messages: vec![
                LLMMessage {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                }
            ],
            functions: Some(vec![
                LLMFunction {
                    name: "respond_with_context".to_string(),
                    description: "Respond to user query using provided context".to_string(),
                    parameters: FunctionParameters {
                        context_info: compressed_context.clone(),
                        user_query: user_query.to_string(),
                    },
                }
            ]),
            function_call: Some(FunctionCall::Auto),
            max_tokens: Some(500),
            temperature: Some(0.7),
        };

        println!("   📊 Structured context transfer | Context size: {} tokens",
                compressed_context.compressed_tokens);

        let response = self.llm_client.chat_completion(request).await?;

        // 处理function call响应
        if let Some(function_call) = &response.choices[0].message.function_call {
            Ok(function_call.arguments.clone())
        } else {
            Ok(response.choices[0].message.content.clone())
        }
    }
}

/// 🎯 方式三：系统消息传递（推荐用于敏感Context）
/// 将Context放在system message中，与用户查询分离
pub struct SystemMessageTransfer {
    pub llm_client: LLMClient,
}

impl SystemMessageTransfer {
    /// 系统消息方式：Context作为系统级指令
    pub async fn send_context_via_system_message(
        &self,
        compressed_context: &CompressedContext,
        user_query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("📤 Method 3: System Message Transfer");

        // 构建系统消息，包含所有Context信息
        let system_message = self.build_system_context_message(compressed_context);

        let request = LLMRequest {
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: system_message,
                },
                LLMMessage {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                }
            ],
            max_tokens: Some(500),
            temperature: Some(0.7),
        };

        println!("   📊 System context: {} tokens | User query: {} tokens",
                compressed_context.compressed_tokens,
                self.estimate_tokens(user_query));

        let response = self.llm_client.chat_completion(request).await?;
        Ok(response.choices[0].message.content.clone())
    }

    /// 构建系统Context消息
    fn build_system_context_message(&self, context: &CompressedContext) -> String {
        format!(
            "You are an AI assistant with access to the following context about the current user:\n\n\
            USER PROFILE: {}\n\
            INTERACTION HISTORY: {}\n\
            RELEVANT KNOWLEDGE: {}\n\
            COMMUNICATION PREFERENCES: {}\n\n\
            Use this context to provide personalized and relevant responses. \
            Do not explicitly mention having access to this context unless relevant to the conversation.",
            context.essential_user_info,
            context.relevant_history,
            context.task_knowledge,
            context.personalization_hints
        )
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }
}

/// 🎯 方式四：流式传递（适用于大型Context）
/// 将大型Context分块流式传递给LLM
pub struct StreamingTransfer {
    pub llm_client: LLMClient,
}

impl StreamingTransfer {
    /// 流式方式：分块传递大型Context
    pub async fn send_context_via_streaming(
        &self,
        compressed_context: &CompressedContext,
        user_query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("📤 Method 4: Streaming Transfer");

        // 将Context分成多个chunks
        let context_chunks = self.split_context_into_chunks(compressed_context, 200); // 每块200 tokens

        let mut conversation_messages = vec![
            LLMMessage {
                role: "system".to_string(),
                content: "You will receive context information in chunks. Acknowledge each chunk and prepare to answer based on all provided context.".to_string(),
            }
        ];

        // 逐步发送Context chunks
        for (i, chunk) in context_chunks.iter().enumerate() {
            conversation_messages.push(LLMMessage {
                role: "user".to_string(),
                content: format!("Context chunk {}: {}", i + 1, chunk),
            });

            // 模拟LLM确认接收
            conversation_messages.push(LLMMessage {
                role: "assistant".to_string(),
                content: format!("Understood context chunk {}. Ready for next chunk or query.", i + 1),
            });

            println!("   📦 Sent chunk {}/{} ({} tokens)",
                    i + 1, context_chunks.len(), self.estimate_tokens(chunk));
        }

        // 发送最终查询
        conversation_messages.push(LLMMessage {
            role: "user".to_string(),
            content: format!("Now please answer this query using all the context provided: {}", user_query),
        });

        let request = LLMRequest {
            messages: conversation_messages,
            max_tokens: Some(500),
            temperature: Some(0.7),
        };

        let response = self.llm_client.chat_completion(request).await?;
        Ok(response.choices[0].message.content.clone())
    }

    /// 将Context分块
    fn split_context_into_chunks(&self, context: &CompressedContext, chunk_size: u32) -> Vec<String> {
        let full_context = format!(
            "User: {} | History: {} | Knowledge: {} | Preferences: {}",
            context.essential_user_info,
            context.relevant_history,
            context.task_knowledge,
            context.personalization_hints
        );

        // 简化的分块逻辑
        let chunk_char_size = (chunk_size * 4) as usize; // 大约4字符=1token
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < full_context.len() {
            let end = std::cmp::min(start + chunk_char_size, full_context.len());
            chunks.push(full_context[start..end].to_string());
            start = end;
        }

        chunks
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }
}

/// 🎯 方式五：缓存辅助传递（适用于重复Context）
/// 利用LLM的上下文缓存能力，减少重复传输
pub struct CacheAssistedTransfer {
    pub llm_client: LLMClient,
    pub context_cache: HashMap<String, String>, // context_id -> cached_reference
}

impl CacheAssistedTransfer {
    /// 缓存辅助方式：使用上下文缓存
    pub async fn send_context_via_cache(
        &mut self,
        compressed_context: &CompressedContext,
        user_query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("📤 Method 5: Cache-Assisted Transfer");

        let context_id = &compressed_context.context_id;

        // 检查是否已缓存
        if let Some(cached_ref) = self.context_cache.get(context_id) {
            println!("   💾 Using cached context reference: {}", cached_ref);

            // 使用缓存引用
            let request = LLMRequest {
                messages: vec![
                    LLMMessage {
                        role: "system".to_string(),
                        content: format!("Use previously cached context: {}", cached_ref),
                    },
                    LLMMessage {
                        role: "user".to_string(),
                        content: user_query.to_string(),
                    }
                ],
                max_tokens: Some(500),
                temperature: Some(0.7),
            };

            let response = self.llm_client.chat_completion(request).await?;
            return Ok(response.choices[0].message.content.clone());
        }

        // 首次传输，建立缓存
        println!("   🆕 First time context, establishing cache...");

        let cache_establishment_request = LLMRequest {
            messages: vec![
                LLMMessage {
                    role: "system".to_string(),
                    content: format!(
                        "Please cache the following context with ID '{}' for future reference:\n\n\
                        User Profile: {}\n\
                        History: {}\n\
                        Knowledge: {}\n\
                        Preferences: {}\n\n\
                        Respond with 'Context cached successfully' when ready.",
                        context_id,
                        compressed_context.essential_user_info,
                        compressed_context.relevant_history,
                        compressed_context.task_knowledge,
                        compressed_context.personalization_hints
                    ),
                }
            ],
            max_tokens: Some(100),
            temperature: Some(0.1),
        };

        let cache_response = self.llm_client.chat_completion(cache_establishment_request).await?;

        // 存储缓存引用
        self.context_cache.insert(context_id.clone(), format!("cached_context_{}", context_id));

        println!("   ✅ Context cached: {}", cache_response.choices[0].message.content);

        // 现在使用缓存处理实际查询
        self.send_context_via_cache(compressed_context, user_query).await
    }
}

/// 🎯 完整的Context传递管理器
/// 根据不同场景选择最优的传递方式
pub struct ContextTransferManager {
    standard_transfer: StandardAPITransfer,
    structured_transfer: StructuredAPITransfer,
    system_transfer: SystemMessageTransfer,
    streaming_transfer: StreamingTransfer,
    cache_transfer: CacheAssistedTransfer,
}

impl ContextTransferManager {
    pub fn new(llm_client: LLMClient) -> Self {
        Self {
            standard_transfer: StandardAPITransfer { llm_client: llm_client.clone() },
            structured_transfer: StructuredAPITransfer { llm_client: llm_client.clone() },
            system_transfer: SystemMessageTransfer { llm_client: llm_client.clone() },
            streaming_transfer: StreamingTransfer { llm_client: llm_client.clone() },
            cache_transfer: CacheAssistedTransfer {
                llm_client: llm_client.clone(),
                context_cache: HashMap::new(),
            },
        }
    }

    /// 智能选择最优传递方式
    pub async fn send_context_intelligently(
        &mut self,
        compressed_context: &CompressedContext,
        user_query: &str,
        preferences: &TransferPreferences,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        println!("🤖 Intelligent Context Transfer Selection");
        println!("   📊 Context size: {} tokens", compressed_context.compressed_tokens);
        println!("   🎯 Transfer preferences: {:?}", preferences);

        match self.select_optimal_method(compressed_context, preferences) {
            TransferMethod::Standard => {
                println!("   ✅ Selected: Standard API Transfer");
                self.standard_transfer.send_context_via_prompt(compressed_context, user_query).await
            },
            TransferMethod::Structured => {
                println!("   ✅ Selected: Structured API Transfer");
                self.structured_transfer.send_context_via_functions(compressed_context, user_query).await
            },
            TransferMethod::SystemMessage => {
                println!("   ✅ Selected: System Message Transfer");
                self.system_transfer.send_context_via_system_message(compressed_context, user_query).await
            },
            TransferMethod::Streaming => {
                println!("   ✅ Selected: Streaming Transfer");
                self.streaming_transfer.send_context_via_streaming(compressed_context, user_query).await
            },
            TransferMethod::CacheAssisted => {
                println!("   ✅ Selected: Cache-Assisted Transfer");
                self.cache_transfer.send_context_via_cache(compressed_context, user_query).await
            },
        }
    }

    /// 选择最优传递方法
    fn select_optimal_method(
        &self,
        context: &CompressedContext,
        preferences: &TransferPreferences,
    ) -> TransferMethod {

        // 基于Context大小和偏好选择方法
        match (context.compressed_tokens, preferences.priority) {
            // 小Context，标准传递
            (0..=300, _) => TransferMethod::Standard,

            // 中等Context，根据偏好选择
            (301..=600, TransferPriority::Privacy) => TransferMethod::SystemMessage,
            (301..=600, TransferPriority::Structure) => TransferMethod::Structured,
            (301..=600, TransferPriority::Performance) => TransferMethod::CacheAssisted,

            // 大Context，流式传递
            (601.., _) => TransferMethod::Streaming,
        }
    }
}

/// 传递方法枚举
#[derive(Debug, Clone)]
pub enum TransferMethod {
    Standard,        // 标准prompt嵌入
    Structured,      // 结构化Function调用
    SystemMessage,   // 系统消息传递
    Streaming,       // 流式分块传递
    CacheAssisted,   // 缓存辅助传递
}

/// 传递偏好设置
#[derive(Debug, Clone)]
pub struct TransferPreferences {
    pub priority: TransferPriority,
    pub max_context_tokens: u32,
    pub cache_enabled: bool,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone)]
pub enum TransferPriority {
    Performance,     // 优先考虑性能
    Privacy,         // 优先考虑隐私
    Structure,       // 优先考虑结构化
    CostOptimization, // 优先考虑成本
}

#[derive(Debug, Clone)]
pub enum PrivacyLevel {
    Low,    // 允许在prompt中包含Context
    Medium, // 使用系统消息隔离
    High,   // 使用缓存和引用
}

// 支撑结构定义
#[derive(Debug, Clone)]
pub struct CompressedContext {
    pub context_id: String,
    pub essential_user_info: String,
    pub relevant_history: String,
    pub task_knowledge: String,
    pub personalization_hints: String,
    pub compressed_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct LLMClient;

impl LLMClient {
    pub async fn chat_completion(&self, request: LLMRequest) -> Result<LLMResponse, Box<dyn std::error::Error + Send + Sync>> {
        // 模拟API调用
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(LLMResponse {
            choices: vec![
                LLMChoice {
                    message: LLMMessage {
                        role: "assistant".to_string(),
                        content: "Based on the provided context, here's my response...".to_string(),
                        function_call: None,
                    }
                }
            ]
        })
    }
}

#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub messages: Vec<LLMMessage>,
    pub functions: Option<Vec<LLMFunction>>,
    pub function_call: Option<FunctionCall>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
    pub function_call: Option<FunctionCallResponse>,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub choices: Vec<LLMChoice>,
}

#[derive(Debug, Clone)]
pub struct LLMChoice {
    pub message: LLMMessage,
}

#[derive(Debug, Clone)]
pub struct LLMFunction {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Clone)]
pub struct FunctionParameters {
    pub context_info: CompressedContext,
    pub user_query: String,
}

#[derive(Debug, Clone)]
pub enum FunctionCall {
    Auto,
}

#[derive(Debug, Clone)]
pub struct FunctionCallResponse {
    pub arguments: String,
}

/// 🚀 Demo主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Context Transfer Methods Demonstration");
    println!("{}", "=".repeat(60));

    // 创建模拟的压缩Context
    let compressed_context = CompressedContext {
        context_id: "ctx_user_zhang_001".to_string(),
        essential_user_info: "User: 张先生 (本科), technical level: high, satisfaction: 8.5".to_string(),
        relevant_history: "Recent: 1. login issue → cache clear resolved; 2. password reset → guided".to_string(),
        task_knowledge: "Similar cases: 3 login resolutions via cache clearing".to_string(),
        personalization_hints: "Adjust tone for concise communication".to_string(),
        compressed_tokens: 280,
    };

    let user_query = "I'm having trouble logging in again, similar to last week's issue.";
    let llm_client = LLMClient;

    // 创建传递管理器
    let mut transfer_manager = ContextTransferManager::new(llm_client);

    // 设置传递偏好
    let preferences = TransferPreferences {
        priority: TransferPriority::Performance,
        max_context_tokens: 500,
        cache_enabled: true,
        privacy_level: PrivacyLevel::Medium,
    };

    // 演示智能传递
    let response = transfer_manager.send_context_intelligently(
        &compressed_context,
        user_query,
        &preferences,
    ).await?;

    println!("\n📋 Final Response: {}", response);

    println!("\n🎯 Key Insights:");
    println!("   📤 5 different context transfer methods available");
    println!("   🤖 Intelligent selection based on context size and preferences");
    println!("   💾 Cache-assisted transfer for repeated contexts");
    println!("   🔒 Privacy-aware transfer options");
    println!("   ⚡ Performance optimization through method selection");

    println!("\n✨ Context transfer methods demonstrated successfully!");
    Ok(())
}
