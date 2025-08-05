use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

// 暂时注释掉rust_bert的使用，避免依赖冲突
// use rust_bert::pipelines::sentence_embeddings::{
//     SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
// };

/// 支持的embedding模型类型（基于行业标准）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// 使用rust-bert的预训练模型 (暂时禁用)
    // RustBert {
    //     model_type: RustBertModelType,
    //     device: DeviceType,
    // },
    /// OpenAI API
    OpenAI {
        model: String,
        api_key: String,
    },
    /// 本地Mock模型（用于测试）
    Mock { dimension: usize },
}

/// rust-bert支持的模型类型 (保留用于将来)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustBertModelType {
    /// all-MiniLM-L6-v2 (384维，快速)
    AllMiniLmL6V2,
    /// all-mpnet-base-v2 (768维，高质量)
    AllMpnetBaseV2,
    /// distilbert-base-nli-stsb-mean-tokens (768维)
    DistilBertBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Cpu,
    Cuda(usize), // GPU设备ID
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        EmbeddingModel::Mock { dimension: 384 }
    }
}

/// Embedding配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: EmbeddingModel,
    pub cache_size: usize,
    pub batch_size: usize,
    pub max_length: usize,
    pub normalize: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::default(),
            cache_size: 1000,
            batch_size: 32,
            max_length: 512,
            normalize: true,
        }
    }
}

/// 统一的embedding提供器（使用现成库）
pub struct EmbeddingProvider {
    config: EmbeddingConfig,
    cache: LruCache<String, Vec<f32>>,
    handler: Box<dyn EmbeddingHandler + Send + Sync>,
}

/// Embedding处理器trait
pub trait EmbeddingHandler {
    fn encode(&self, text: &str) -> Result<Vec<f32>>;
    fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn model_info(&self) -> String;
}

// 暂时注释掉rust-bert处理器，避免依赖冲突
// /// rust-bert处理器（使用现成库）
// pub struct RustBertHandler {
//     model: SentenceEmbeddingsModel,
//     dimension: usize,
//     model_info: String,
// }

/// OpenAI API处理器
#[cfg(feature = "openai-api")]
pub struct OpenAIHandler {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
}

#[cfg(feature = "openai-api")]
impl OpenAIHandler {
    pub fn new(model: String, api_key: String) -> Result<Self> {
        let dimension = match model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536, // 默认维度
        };

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            dimension,
        })
    }

    async fn call_openai_api(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        #[derive(Serialize)]
        struct OpenAIRequest<'a> {
            input: &'a [&'a str],
            model: &'a str,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            data: Vec<EmbeddingData>,
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        let request = OpenAIRequest {
            input: texts,
            model: &self.model,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("OpenAI API error: {}", response.status()));
        }

        let openai_response: OpenAIResponse = response.json().await?;
        Ok(openai_response.data.into_iter().map(|d| d.embedding).collect())
    }
}

#[cfg(feature = "openai-api")]
impl EmbeddingHandler for OpenAIHandler {
    fn encode(&self, text: &str) -> Result<Vec<f32>> {
        // 同步包装异步调用
        let rt = tokio::runtime::Runtime::new()?;
        let embeddings = rt.block_on(self.call_openai_api(&[text]))?;
        Ok(embeddings.into_iter().next().unwrap())
    }

    fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self.call_openai_api(texts))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_info(&self) -> String {
        format!("OpenAI {}", self.model)
    }
}

/// Mock处理器（用于测试）
pub struct MockHandler {
    dimension: usize,
}

impl MockHandler {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl EmbeddingHandler for MockHandler {
    fn encode(&self, text: &str) -> Result<Vec<f32>> {
        Ok(generate_enhanced_mock_embedding(text, self.dimension))
    }

    fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter()
            .map(|text| self.encode(text).unwrap())
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_info(&self) -> String {
        format!("Enhanced Mock ({}D)", self.dimension)
    }
}

impl EmbeddingProvider {
    /// 创建新的embedding提供器
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let handler: Box<dyn EmbeddingHandler + Send + Sync> = match &config.model {
            // 暂时注释掉rust-bert支持
            // EmbeddingModel::RustBert { model_type, device } => {
            //     Box::new(RustBertHandler::new(model_type.clone(), device.clone())?)
            // },
            #[cfg(feature = "openai-api")]
            EmbeddingModel::OpenAI { model, api_key } => {
                Box::new(OpenAIHandler::new(model.clone(), api_key.clone())?)
            },
            #[cfg(not(feature = "openai-api"))]
            EmbeddingModel::OpenAI { .. } => {
                return Err(anyhow!("OpenAI API support not enabled. Enable 'openai-api' feature."));
            },
            EmbeddingModel::Mock { dimension } => {
                Box::new(MockHandler::new(*dimension))
            },
        };

        let cache = LruCache::new(NonZeroUsize::new(config.cache_size).unwrap());

        Ok(Self {
            config,
            cache,
            handler,
        })
    }

    /// 编码单个文本
    pub fn encode(&mut self, text: &str) -> Result<Vec<f32>> {
        // 检查缓存
        if let Some(cached) = self.cache.get(text) {
            return Ok(cached.clone());
        }

        // 截断过长的文本
        let truncated_text = if text.len() > self.config.max_length {
            &text[..self.config.max_length]
        } else {
            text
        };

        // 生成embedding
        let mut embedding = self.handler.encode(truncated_text)?;

        // 归一化（如果启用）
        if self.config.normalize {
            l2_normalize(&mut embedding);
        }

        // 缓存结果
        self.cache.put(text.to_string(), embedding.clone());

        Ok(embedding)
    }

    /// 批量编码文本
    pub fn encode_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();

        // 检查缓存
        for (i, text) in texts.iter().enumerate() {
            if let Some(cached) = self.cache.get(*text) {
                results.push(Some(cached.clone()));
            } else {
                results.push(None);
                uncached_texts.push(*text);
                uncached_indices.push(i);
            }
        }

        // 批量处理未缓存的文本
        if !uncached_texts.is_empty() {
            let mut embeddings = self.handler.encode_batch(&uncached_texts)?;

            // 归一化（如果启用）
            if self.config.normalize {
                for embedding in &mut embeddings {
                    l2_normalize(embedding);
                }
            }

            for (embedding, &original_index) in embeddings.iter().zip(&uncached_indices) {
                results[original_index] = Some(embedding.clone());
                // 缓存结果
                self.cache.put(texts[original_index].to_string(), embedding.clone());
            }
        }

        // 转换为最终结果
        results.into_iter()
            .map(|opt| opt.ok_or_else(|| anyhow!("Missing embedding result")))
            .collect()
    }

    /// 获取embedding维度
    pub fn dimension(&self) -> usize {
        self.handler.dimension()
    }

    /// 获取模型信息
    pub fn model_info(&self) -> String {
        self.handler.model_info()
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.cap().get())
    }
}

/// 生成增强的mock embedding（改进版）
fn generate_enhanced_mock_embedding(text: &str, dimension: usize) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(dimension);

    // 基于文本内容的多重特征
    let text_len_factor = (text.len() as f32).sqrt() * 0.01;
    let char_diversity = calculate_text_entropy(text);
    let word_count = text.split_whitespace().count() as f32 * 0.001;
    let text_hash = simple_hash(text) * 0.001;

    for i in 0..dimension {
        let i_float = i as f32;

        // 多层次特征组合
        let base_feature = (i_float * text_len_factor * 2.0).sin() * 0.2;
        let diversity_feature = (i_float * char_diversity * 3.0).cos() * 0.15;
        let word_feature = (i_float * word_count * 4.0).sin() * 0.1;
        let hash_feature = (i_float * text_hash * 5.0).cos() * 0.25;

        // 位置权重
        let position_weight = 1.0 - (i_float / dimension as f32) * 0.2;

        let combined_value = (base_feature + diversity_feature + word_feature + hash_feature)
            * position_weight;

        embedding.push(combined_value.tanh());
    }

    // L2归一化
    l2_normalize(&mut embedding);

    embedding
}

fn calculate_text_entropy(text: &str) -> f32 {
    let mut char_counts = HashMap::new();
    let total_chars = text.chars().count() as f32;

    for c in text.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let entropy: f32 = char_counts
        .values()
        .map(|&count| {
            let p = count as f32 / total_chars;
            -p * p.ln()
        })
        .sum();

    entropy * 0.1
}

fn simple_hash(text: &str) -> f32 {
    text.chars()
        .enumerate()
        .map(|(i, c)| (c as u32 as f32) * (i + 1) as f32)
        .sum::<f32>()
}

fn l2_normalize(embedding: &mut Vec<f32>) {
    let norm: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in embedding.iter_mut() {
            *val /= norm;
        }
    }
}

// 暂时注释掉rust-bert相关的便利函数
// /// 便利函数：创建rust-bert提供器
// pub fn create_rust_bert_provider(model_type: RustBertModelType) -> Result<EmbeddingProvider> {
//     let config = EmbeddingConfig {
//         model: EmbeddingModel::RustBert {
//             model_type,
//             device: DeviceType::Cpu,
//         },
//         ..Default::default()
//     };
//     EmbeddingProvider::new(config)
// }

/// 便利函数：创建OpenAI提供器
#[cfg(feature = "openai-api")]
pub fn create_openai_provider(api_key: String) -> Result<EmbeddingProvider> {
    let config = EmbeddingConfig {
        model: EmbeddingModel::OpenAI {
            model: "text-embedding-3-small".to_string(),
            api_key,
        },
        ..Default::default()
    };
    EmbeddingProvider::new(config)
}

/// 便利函数：创建Mock提供器
pub fn create_mock_provider(dimension: usize) -> Result<EmbeddingProvider> {
    let config = EmbeddingConfig {
        model: EmbeddingModel::Mock { dimension },
        ..Default::default()
    };
    EmbeddingProvider::new(config)
}
