use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

mod llm_client;
mod cache;
mod context_engine;
mod storage;

use llm_client::LLMClient;
use cache::ResponseCache;
use context_engine::ContextEngine;
use storage::NodeStorage;

/// OpenAI compatible chat completion request
#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub stream: bool,
    // PC extensions
    #[serde(default)]
    pub context_sharing: bool,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub shared_context_group: Option<String>, // 新增：支持跨Agent上下文组
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI compatible response
#[derive(Debug, Serialize, Clone)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    // PC extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pc_context_used: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pc_compression_ratio: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Node application state
#[derive(Clone)]
pub struct AppState {
    pub llm_client: Arc<LLMClient>,
    pub cache: Arc<ResponseCache>,
    pub context_engine: Arc<ContextEngine>,
    pub storage: Arc<NodeStorage>,
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "prompt-compiler-node",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// OpenAI compatible chat completions endpoint
async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    info!("Received chat completion request for model: {}", request.model);

    // 🔧 添加输入验证
    if let Err(status) = validate_request(&request) {
        return Err(status);
    }

    // Extract PC-specific headers
    let agent_id = headers
        .get("x-pc-agent-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| request.agent_id.clone());

    let context_sharing = headers
        .get("x-pc-context-share")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(request.context_sharing);

    // 🔧 新增：提取shared_context_group
    let shared_context_group = headers
        .get("x-pc-context-group")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| request.shared_context_group.clone());

    // Generate cache key
    let cache_key = generate_cache_key(&request, &agent_id, &shared_context_group);

    // Check cache first
    if let Some(cached_response) = state.cache.get(&cache_key).await {
        info!("Cache hit for request");
        return Ok(Json(cached_response));
    }

    // Process with context engine if enabled
    let processed_request = if context_sharing {
        match state.context_engine.process_request_with_group(&request, agent_id.as_deref(), shared_context_group.as_deref()).await {
            Ok(processed) => {
                info!("Context processing completed with compression ratio: {:.2}%",
                     processed.compression_ratio * 100.0);
                processed
            }
            Err(e) => {
                warn!("Context processing failed: {}, falling back to direct LLM call", e);
                ProcessedRequest {
                    messages: request.messages.clone(),
                    compression_ratio: 0.0,
                    context_used: false,
                }
            }
        }
    } else {
        ProcessedRequest {
            messages: request.messages.clone(),
            compression_ratio: 0.0,
            context_used: false,
        }
    };

    // Call LLM
    match state.llm_client.complete(&request, &processed_request.messages).await {
        Ok(llm_response) => {
            let response = ChatCompletionResponse {
                id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                model: request.model.clone(),
                choices: vec![Choice {
                    index: 0,
                    message: ChatMessage {
                        role: "assistant".to_string(),
                        content: llm_response.content,
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: llm_response.usage.prompt_tokens,
                    completion_tokens: llm_response.usage.completion_tokens,
                    total_tokens: llm_response.usage.total_tokens,
                },
                pc_context_used: if context_sharing { Some(processed_request.context_used) } else { None },
                pc_compression_ratio: if context_sharing { Some(processed_request.compression_ratio) } else { None },
            };

            // Cache the response
            state.cache.insert(cache_key, response.clone()).await;

            // Store context if sharing is enabled
            if context_sharing {
                if let Err(e) = state.context_engine.store_interaction_with_group(&request, &response, agent_id.as_deref(), shared_context_group.as_deref()).await {
                    warn!("Failed to store interaction context: {}", e);
                }
            }

            Ok(Json(response))
        }
        Err(e) => {
            error!("LLM call failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// PC native context-aware completion endpoint
async fn pc_context_completion(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    info!("Received PC native context completion request");

    // Force context sharing for PC native endpoint
    let mut pc_request = request;
    pc_request.context_sharing = true;

    chat_completions(State(state), HeaderMap::new(), Json(pc_request)).await
}

/// List available models (OpenAI compatible)
async fn list_models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "gpt-4",
                "object": "model",
                "created": 1677610602,
                "owned_by": "openai"
            },
            {
                "id": "gpt-3.5-turbo",
                "object": "model",
                "created": 1677610602,
                "owned_by": "openai"
            },
            {
                "id": "pc-enhanced",
                "object": "model",
                "created": 1677610602,
                "owned_by": "prompt-compiler"
            }
        ]
    }))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // OpenAI compatible endpoints
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        // PC native endpoints
        .route("/v1/pc/context-completion", post(pc_context_completion))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    // Initialize components
    info!("Initializing Prompt Compiler Node...");

    let storage = Arc::new(NodeStorage::new("./pc_node_data").await?);
    let cache = Arc::new(ResponseCache::new(1000).await);
    let llm_client = Arc::new(LLMClient::new().await?);
    let context_engine = Arc::new(ContextEngine::new(storage.clone()).await?);

    let state = AppState {
        llm_client,
        cache,
        context_engine,
        storage,
    };

    let app = create_router(state);
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("🚀 Prompt Compiler Node running on http://{}", addr);
    info!("📖 OpenAI Compatible API: http://{}/v1/chat/completions", addr);
    info!("🧠 PC Enhanced API: http://{}/v1/pc/context-completion", addr);
    info!("❤️  Health Check: http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// Helper functions
fn default_temperature() -> f32 { 0.7 }
fn default_max_tokens() -> u32 { 1000 }

fn generate_cache_key(request: &ChatCompletionRequest, agent_id: &Option<String>, shared_context_group: &Option<String>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    request.messages.hash(&mut hasher);
    request.model.hash(&mut hasher);
    agent_id.hash(&mut hasher);
    shared_context_group.hash(&mut hasher);
    format!("chat_completion_{:x}", hasher.finish())
}

#[derive(Debug)]
pub struct ProcessedRequest {
    pub messages: Vec<ChatMessage>,
    pub compression_ratio: f32,
    pub context_used: bool,
}

// 输入验证函数
fn validate_request(request: &ChatCompletionRequest) -> Result<(), StatusCode> {
    // 检查模型是否为空
    if request.model.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 🔧 检查模型是否为支持的模型
    let supported_models = [
        "gpt-4", "gpt-3.5-turbo", "gpt-4-turbo",
        "pc-enhanced", "text-davinci-003"
    ];

    if !supported_models.contains(&request.model.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 检查消息是否为空
    if request.messages.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 🔧 检查消息内容是否有效
    for message in &request.messages {
        if message.role.is_empty() || message.content.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        // 检查角色是否有效
        let valid_roles = ["user", "assistant", "system"];
        if !valid_roles.contains(&message.role.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 检查温度是否在合理范围内
    if request.temperature < 0.0 || request.temperature > 2.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 检查最大令牌数是否合理
    if request.max_tokens == 0 || request.max_tokens > 8192 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}
