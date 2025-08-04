//! Web interface and API server for prompt compiler

use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
pub struct CompileRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub budget: Option<u32>,
    pub priority: Option<u8>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub compiled_prompt: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

pub async fn create_app() -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/compile", post(compile_prompt))
        .route("/analyze", post(analyze_prompt))
        .layer(CorsLayer::permissive())
}

async fn health_check() -> &'static str {
    "Prompt Compiler API is running"
}

async fn compile_prompt(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<ApiError>)> {
    // TODO: Implement actual compilation logic
    let response = CompileResponse {
        compiled_prompt: format!("Compiled: {}", payload.prompt),
        metadata: HashMap::new(),
    };
    
    Ok(Json(response))
}

async fn analyze_prompt(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<HashMap<String, f32>>, (StatusCode, Json<ApiError>)> {
    // TODO: Implement actual analysis logic
    let mut analysis = HashMap::new();
    analysis.insert("intent_clarity".to_string(), 0.8);
    analysis.insert("context_relevance".to_string(), 0.7);
    
    Ok(Json(analysis))
}
