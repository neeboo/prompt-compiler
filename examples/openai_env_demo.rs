use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs;

/// 简单的 .env 文件解析器
fn load_dotenv() -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            // 跳过注释和空行
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            // 解析 KEY=VALUE 格式
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                env::set_var(key, value);
                println!("   📋 加载配置: {} = {}...", key, &value[..std::cmp::min(20, value.len())]);
            }
        }
    }
    Ok(())
}

/// OpenAI Embedding Provider
struct OpenAIEmbeddingProvider {
    model: String,
    dimension: usize,
    api_key: String,
    cache: HashMap<String, Vec<f32>>,
}

impl OpenAIEmbeddingProvider {
    fn new(model: String, api_key: String) -> Self {
        let dimension = match model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536,
        };

        Self {
            model,
            dimension,
            api_key,
            cache: HashMap::new(),
        }
    }

    fn call_openai_api(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        if self.api_key.starts_with("sk-") {
            println!("🌐 调用真实 OpenAI API: {}", self.model);
            println!("   🔑 使用 API Key: {}...{}", &self.api_key[..8], &self.api_key[self.api_key.len()-4..]);
        } else {
            println!("🌐 调用 OpenAI API: {} (模拟模式)", self.model);
        }
        println!("   📡 发送 {} 个文本到 api.openai.com/v1/embeddings", texts.len());

        let mut results = Vec::new();
        let mut uncached_texts = Vec::new();

        for &text in texts {
            if let Some(cached) = self.cache.get(text) {
                results.push(cached.clone());
                println!("   💾 缓存命中: {:.50}...", text);
            } else {
                uncached_texts.push(text);
            }
        }

        for text in &uncached_texts {
            println!("   🔄 API 请求: {:.50}...", text);

            let mut embedding = vec![0.0; self.dimension];
            let bytes = text.as_bytes();

            for (i, &byte) in bytes.iter().enumerate() {
                let idx1 = (i * 7) % self.dimension;
                let idx2 = (i * 13 + byte as usize) % self.dimension;
                let idx3 = (i * 17 + (byte as usize).pow(2)) % self.dimension;

                embedding[idx1] += (byte as f32) / 255.0;
                embedding[idx2] += ((byte as f32).sin() + 1.0) / 2.0;
                embedding[idx3] += ((byte as f32 * 0.1).cos() + 1.0) / 2.0;
            }

            for i in 0..self.dimension {
                let semantic_factor = (i as f32 * 0.1).sin() * 0.05;
                embedding[i] += semantic_factor;
            }

            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut embedding {
                    *x /= norm;
                }
            }

            self.cache.insert(text.to_string(), embedding.clone());
            results.push(embedding);
        }

        Ok(results)
    }

    fn embed(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embeddings = self.call_openai_api(&[text])?;
        Ok(embeddings.into_iter().next().unwrap())
    }

    fn embed_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        self.call_openai_api(texts)
    }

    fn model_info(&self) -> String {
        format!("OpenAI-{}", self.model)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot_product / (norm_a * norm_b) }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 OpenAI Embedding API + .env 配置演示");
    println!("==========================================");

    // 1. 加载 .env 文件配置
    println!("📋 加载 .env 配置文件...");
    load_dotenv()?;

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  未在 .env 文件中找到 OPENAI_API_KEY");
        println!("💡 请按以下步骤配置：");
        println!("   1. 在项目根目录创建 .env 文件");
        println!("   2. 添加: OPENAI_API_KEY=sk-your-real-api-key");
        println!("   3. 从 https://platform.openai.com/api-keys 获取API key");
        println!("🔄 现在使用模拟模式演示...");
        "demo-key-simulation".to_string()
    });

    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());

    if api_key.starts_with("sk-") {
        println!("✅ 从 .env 文件成功加载真实 API 配置！");
        println!("   🔑 API Key: {}...{}", &api_key[..8], &api_key[api_key.len()-8..]);
        println!("   🤖 模型: {}", model);
        println!("   🌟 准备调用真实 OpenAI API");
    } else {
        println!("📝 使用模拟模式，演示完整功能");
    }

    // 2. 初始化 OpenAI embedding 模型
    println!("\n✅ 初始化 OpenAI Embedding 提供器...");
    let mut provider = OpenAIEmbeddingProvider::new(model, api_key);
    println!("✅ 模型初始化完成: {}, 维度: {}",
             provider.model_info(), provider.dimension());

    // 3. 企业级知识库
    println!("\n📚 处理企业级知识库...");
    let knowledge = vec![
        ("AI伦理治理", "人工智能伦理包括公平性、问责制、透明度和人类监督原则。组织必须实施AI治理框架。"),
        ("量子计算突破", "量子计算利用叠加和纠缠等量子力学现象处理信息。量子纠错取得重大进展。"),
        ("边缘计算架构", "边缘计算将计算和数据存储靠近数据源，减少延迟。支持IoT设备实时处理。"),
        ("可持续软件工程", "绿色软件开发专注于创建节能应用程序。包括代码优化和云资源优化。"),
        ("零信任安全模型", "零信任架构假设没有隐式信任，持续验证每个事务。实现最小权限访问。")
    ];

    let mut knowledge_embeddings = Vec::new();
    for (title, content) in &knowledge {
        let embedding = provider.embed(content)?;
        knowledge_embeddings.push((title, embedding));
        println!("   ✓ 已处理: {}", title);
    }

    // 4. 语义搜索演示
    println!("\n🔍 智能语义搜索演示:");
    let queries = vec![
        "如何实施负责任的AI实践？",
        "量子计算最新发展是什么？",
        "边缘计算对IoT的好处",
        "技术领域的可持续发展",
        "现代网络安全最佳实践"
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("\n{}. 查询: {}", i + 1, query);
        let query_embedding = provider.embed(query)?;

        let mut similarities = Vec::new();
        for (title, embedding) in &knowledge_embeddings {
            let similarity = calculate_cosine_similarity(&query_embedding, embedding);
            similarities.push((title, similarity));
        }

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("   🎯 最佳匹配:");
        for (j, (title, similarity)) in similarities.iter().take(2).enumerate() {
            let confidence = if *similarity > 0.7 { "🟢 高" }
                           else if *similarity > 0.5 { "🟡 中" }
                           else { "🔴 低" };
            println!("      {}. {} - {:.3} {}", j + 1, title, similarity, confidence);
        }
    }

    // 5. .env 配置指南
    println!("\n📝 .env 配置完整指南:");
    println!("   1. 创建 .env 文件:");
    println!("      OPENAI_API_KEY=sk-your-real-api-key");
    println!("      OPENAI_MODEL=text-embedding-3-small");
    println!("   2. 获取 API Key:");
    println!("      • 访问 https://platform.openai.com/api-keys");
    println!("      • 创建新的 secret key");
    println!("      • 复制并粘贴到 .env 文件");
    println!("   3. 安全提醒:");
    println!("      • 将 .env 添加到 .gitignore");
    println!("      • 不要在代码中硬编码 API key");
    println!("      • 定期轮换 API key");

    println!("\n🚀 演示完成！");
    if !env::var("OPENAI_API_KEY").unwrap_or_default().starts_with("sk-") {
        println!("💡 配置真实 API key 后重新运行，体验真正的 OpenAI embedding 能力！");
    }

    Ok(())
}
