use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs;
use std::io::{Write, BufReader, BufRead};

// 简化的 .env 加载
fn load_dotenv() -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() { continue; }
            if let Some((key, value)) = line.split_once('=') {
                env::set_var(key.trim(), value.trim().trim_matches('"'));
            }
        }
    }
    Ok(())
}

/// 语义块数据结构
#[derive(Clone, Debug)]
struct SemanticChunk {
    pub id: String,
    pub title: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub created_at: u64,
    pub access_count: u64,
    pub semantic_tags: Vec<String>,
}

/// 持久化语义压缩系统
struct PersistentSemanticSystem {
    model: String,
    dimension: usize,
    db_path: String,
    chunks: HashMap<String, SemanticChunk>,
    embedding_cache: HashMap<String, Vec<f32>>,
    stats: SystemStats,
}

#[derive(Debug)]
struct SystemStats {
    total_chunks: usize,
    cache_hits: usize,
    api_calls: usize,
    total_queries: usize,
}

impl PersistentSemanticSystem {
    fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        load_dotenv()?;

        let model = env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-large".to_string());

        let dimension = match model.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 3072,
        };

        let mut system = Self {
            model,
            dimension,
            db_path: db_path.to_string(),
            chunks: HashMap::new(),
            embedding_cache: HashMap::new(),
            stats: SystemStats {
                total_chunks: 0,
                cache_hits: 0,
                api_calls: 0,
                total_queries: 0,
            },
        };

        // 尝试从磁盘加载现有数据
        system.load_from_disk()?;

        Ok(system)
    }

    /// 从磁盘加载现有数据 (简化的文本格式)
    fn load_from_disk(&mut self) -> Result<(), Box<dyn Error>> {
        let chunks_file = format!("{}/chunks.txt", self.db_path);

        if fs::metadata(&chunks_file).is_ok() {
            println!("📂 从磁盘加载现有语义库...");

            let file = fs::File::open(&chunks_file)?;
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

            let mut i = 0;
            while i < lines.len() {
                if lines[i].starts_with("CHUNK:") {
                    let id = lines[i].strip_prefix("CHUNK:").unwrap().to_string();
                    let title = lines.get(i+1).unwrap_or(&"".to_string()).clone();
                    let content = lines.get(i+2).unwrap_or(&"".to_string()).clone();

                    // 使用 get_embedding 而不是 generate_embedding，这样会利用缓存
                    let embedding = self.get_embedding(&content)?;

                    let chunk = SemanticChunk {
                        id: id.clone(),
                        title,
                        content,
                        embedding,
                        created_at: 0,
                        access_count: 0,
                        semantic_tags: vec![],
                    };

                    self.chunks.insert(id, chunk);
                    i += 4; // 跳过分隔符
                } else {
                    i += 1;
                }
            }

            self.stats.total_chunks = self.chunks.len();
            println!("✅ 成功加载 {} 个语义块", self.chunks.len());
        } else {
            println!("📝 首次运行，创建新的语义库");
            fs::create_dir_all(&self.db_path)?;
        }

        Ok(())
    }

    /// 保存到磁盘 (简化的文本格式)
    fn save_to_disk(&self) -> Result<(), Box<dyn Error>> {
        let chunks_file = format!("{}/chunks.txt", self.db_path);
        let mut file = fs::File::create(&chunks_file)?;

        for chunk in self.chunks.values() {
            writeln!(file, "CHUNK:{}", chunk.id)?;
            writeln!(file, "{}", chunk.title)?;
            writeln!(file, "{}", chunk.content)?;
            writeln!(file, "---")?;
        }

        // 保存统计信息
        let stats_file = format!("{}/stats.txt", self.db_path);
        let mut stats_file = fs::File::create(&stats_file)?;
        writeln!(stats_file, "total_chunks:{}", self.stats.total_chunks)?;
        writeln!(stats_file, "cache_hits:{}", self.stats.cache_hits)?;
        writeln!(stats_file, "api_calls:{}", self.stats.api_calls)?;
        writeln!(stats_file, "total_queries:{}", self.stats.total_queries)?;

        println!("💾 数据已保存到磁盘 ({} 个语义块)", self.chunks.len());
        Ok(())
    }

    /// 生成embedding
    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        // 高质量模拟embedding生成
        let mut embedding = vec![0.0; self.dimension];
        let bytes = text.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            let idx1 = (i * 7 + byte as usize) % self.dimension;
            let idx2 = (i * 13 + (byte as usize).pow(2)) % self.dimension;
            let idx3 = (i * 19 + (byte as usize).pow(3)) % self.dimension;

            embedding[idx1] += (byte as f32 / 255.0) * 0.8;
            embedding[idx2] += ((byte as f32 * 0.1).sin() + 1.0) * 0.3;
            embedding[idx3] += ((byte as f32 * 0.01).cos() + 1.0) * 0.2;
        }

        // 添加语义增强
        for i in 0..self.dimension {
            let pos_encoding = ((i as f32 / self.dimension as f32) * 2.0 * std::f32::consts::PI).sin() * 0.1;
            embedding[i] += pos_encoding;
        }

        // L2归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        Ok(embedding)
    }

    /// 获取或生成embedding
    fn get_embedding(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        // 检查内存缓存
        if let Some(cached) = self.embedding_cache.get(text) {
            self.stats.cache_hits += 1;
            println!("   💾 缓存命中: {:.50}...", text);
            return Ok(cached.clone());
        }

        // 生成新的embedding
        self.stats.api_calls += 1;
        println!("   🌐 生成embedding ({})...", self.model);

        let embedding = self.generate_embedding(text)?;
        self.embedding_cache.insert(text.to_string(), embedding.clone());
        Ok(embedding)
    }

    /// 添加知识块到持久化存储
    fn add_knowledge(&mut self, title: &str, content: &str, tags: Vec<String>) -> Result<(), Box<dyn Error>> {
        let chunk_id = format!("chunk_{:04}", self.chunks.len() + 1);

        // 检查是否已存在
        if self.chunks.values().any(|c| c.content == content) {
            println!("⚠️  内容已存在，跳过: {}", title);
            return Ok(());
        }

        let embedding = self.get_embedding(content)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let chunk = SemanticChunk {
            id: chunk_id.clone(),
            title: title.to_string(),
            content: content.to_string(),
            embedding,
            created_at: timestamp,
            access_count: 0,
            semantic_tags: tags,
        };

        self.chunks.insert(chunk_id.clone(), chunk);
        self.stats.total_chunks = self.chunks.len();

        println!("✅ 已添加知识: {} (ID: {})", title, chunk_id);

        // 自动保存到磁盘
        self.save_to_disk()?;
        Ok(())
    }

    /// 语义搜索
    fn semantic_search(&mut self, query: &str, top_k: usize) -> Result<Vec<(String, String, f32)>, Box<dyn Error>> {
        self.stats.total_queries += 1;
        println!("\n🔍 语义搜索: {}", query);

        let query_embedding = self.get_embedding(query)?;
        let mut similarities = Vec::new();

        for chunk in self.chunks.values() {
            let similarity = cosine_similarity(&query_embedding, &chunk.embedding);
            similarities.push((chunk.id.clone(), chunk.title.clone(), similarity));
        }

        // 按相似度排序
        similarities.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        similarities.truncate(top_k);

        println!("📊 搜索结果:");
        for (i, (_, title, similarity)) in similarities.iter().enumerate() {
            let confidence = if *similarity > 0.7 { "🟢 高" }
                           else if *similarity > 0.5 { "🟡 中" }
                           else { "🔴 低" };
            println!("   {}. {} - 相似度: {:.3} {}",
                     i + 1, title, similarity, confidence);
        }

        Ok(similarities)
    }

    /// 生成增强prompt
    fn generate_enhanced_prompt(&mut self, query: &str, context_results: &[(String, String, f32)]) -> Result<String, Box<dyn Error>> {
        let mut prompt = format!("查询: {}\n\n相关上下文:\n", query);

        for (chunk_id, title, similarity) in context_results {
            if *similarity > 0.3 {
                if let Some(chunk) = self.chunks.get_mut(chunk_id) {
                    chunk.access_count += 1; // 更新访问计数
                    prompt.push_str(&format!("\n## {} (相似度: {:.3})\n{}\n",
                                           title, similarity, chunk.content));
                }
            }
        }

        prompt.push_str("\n基于以上上下文，请提供准确的回答。");
        Ok(prompt)
    }

    /// 获取系统统计信息
    fn get_stats(&self) -> &SystemStats {
        &self.stats
    }

    /// 演示缓存效果
    fn demo_cache_effect(&mut self) -> Result<(), Box<dyn Error>> {
        println!("\n🧪 缓存效果演示:");

        let test_queries = vec![
            "边缘计算在IoT中的应用？",
            "量子计算的商业前景如何？",
            "边缘计算在IoT中的应用？", // 重复查询
            "量子计算的商业前景如何？", // 重复查询
        ];

        for (i, query) in test_queries.iter().enumerate() {
            println!("\n--- 测试查询 {} ---", i + 1);
            let _embedding = self.get_embedding(query)?;
        }

        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot_product / (norm_a * norm_b) }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 缓存效果验证演示");
    println!("=====================================");

    // 1. 初始化持久化系统
    let mut system = PersistentSemanticSystem::new("./cache_test_db")?;
    println!("✅ 系统初始化完成 - 模型: {}, 维度: {}", system.model, system.dimension);

    // 2. 添加少量测试数据
    println!("\n📚 添加测试数据...");

    let knowledge_entries = vec![
        ("边缘计算",
         "边缘计算将数据处理能力部署到网络边缘，减少延迟并提高响应速度。",
         vec!["边缘计算".to_string()]),

        ("量子计算",
         "量子计算正从实验室走向商业应用，具有巨大的商业潜力。",
         vec!["量子计算".to_string()]),
    ];

    for (title, content, tags) in knowledge_entries {
        system.add_knowledge(title, content, tags)?;
    }

    // 3. 缓存效果演示
    system.demo_cache_effect()?;

    // 4. 显示最终统计
    println!("\n📊 缓存效果分析:");
    let stats = system.get_stats();

    println!("   📚 总语义块数: {}", stats.total_chunks);
    println!("   💾 缓存命中: {} 次", stats.cache_hits);
    println!("   🌐 API调用: {} 次", stats.api_calls);

    let cache_hit_rate = if stats.api_calls + stats.cache_hits > 0 {
        (stats.cache_hits as f32 / (stats.api_calls + stats.cache_hits) as f32) * 100.0
    } else { 0.0 };
    println!("   📈 缓存命中率: {:.1}%", cache_hit_rate);

    // 5. 说明缓存逻辑
    println!("\n💡 缓存机制说明:");
    println!("   ✅ 相同文本内容 → 缓存命中");
    println!("   ❌ 不同文本内容 → 需要生成新embedding");
    println!("   🔄 这是正常的缓存行为，避免了重复计算相同内容");

    Ok(())
}
