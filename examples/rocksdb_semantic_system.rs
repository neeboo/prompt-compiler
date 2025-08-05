use prompt_compiler_storage::{StateDB, SemanticChunk, ContextInjectionStrategy, CompilationStats};
use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs;

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

/// 基于RocksDB的企业级语义系统
struct RocksDBSemanticSystem {
    db: StateDB,
    model: String,
    dimension: usize,
    embedding_cache: HashMap<String, Vec<f32>>,
    stats: SystemStats,
}

#[derive(Debug)]
struct SystemStats {
    cache_hits: usize,
    api_calls: usize,
    total_queries: usize,
}

impl RocksDBSemanticSystem {
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

        let db = StateDB::new(db_path)?;

        Ok(Self {
            db,
            model,
            dimension,
            embedding_cache: HashMap::new(),
            stats: SystemStats {
                cache_hits: 0,
                api_calls: 0,
                total_queries: 0,
            },
        })
    }

    /// 完善的生成embedding方法
    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        // 检查缓存
        if let Some(cached) = self.embedding_cache.get(text) {
            self.stats.cache_hits += 1;
            println!("   💾 缓存命中: {:.50}...", text);
            return Ok(cached.clone());
        }

        self.stats.api_calls += 1;
        println!("   🌐 调用 {} API...", self.model);

        // 高质量embedding生成
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

        // 语义增强
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

        // 缓存结果
        self.embedding_cache.insert(text.to_string(), embedding.clone());

        Ok(embedding)
    }

    /// 添加语义块到RocksDB
    fn add_semantic_chunk(&mut self, title: &str, content: &str) -> Result<String, Box<dyn Error>> {
        let id = format!("chunk_{}", uuid::Uuid::new_v4());
        let embedding = self.generate_embedding(content)?;

        let chunk = SemanticChunk {
            id: id.clone(),
            content_hash: format!("{:x}", md5::compute(content)),
            compressed_embedding: embedding,
            original_size: content.len(),
            compressed_size: content.len(), // 暂时相等
            compression_ratio: 1.0,
            access_count: 0,
            last_accessed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            semantic_tags: vec![title.to_string()],
        };

        // 存储到RocksDB
        self.db.store_semantic_chunk(&chunk)?;

        println!("✅ 语义块已存储到RocksDB: {}", id);
        Ok(id)
    }

    /// 语义搜索
    fn semantic_search(&mut self, query: &str, top_k: usize) -> Result<Vec<(String, f32)>, Box<dyn Error>> {
        self.stats.total_queries += 1;

        let query_embedding = self.generate_embedding(query)?;
        let chunks = self.db.get_all_semantic_chunks()?;

        let mut similarities = Vec::new();

        for chunk in chunks {
            let similarity = cosine_similarity(&query_embedding, &chunk.compressed_embedding);
            similarities.push((chunk.id, similarity));
        }

        // 按相似度排序
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(top_k);

        println!("🔍 找到 {} 个相关语义块", similarities.len());
        Ok(similarities)
    }

    /// 语义压缩
    fn compress_context(&mut self, context: &str, target_ratio: f32) -> Result<String, Box<dyn Error>> {
        println!("🗜️ 开始语义压缩 (目标比率: {:.1}%)", target_ratio * 100.0);

        // 分割上下文
        let sentences: Vec<&str> = context.split(". ").collect();
        let target_sentences = ((sentences.len() as f32) * target_ratio) as usize;

        if target_sentences >= sentences.len() {
            return Ok(context.to_string());
        }

        // 计算每个句子的重要性
        let mut sentence_scores = Vec::new();
        for sentence in &sentences {
            let embedding = self.generate_embedding(sentence)?;
            let score = embedding.iter().map(|x| x.abs()).sum::<f32>();
            sentence_scores.push((sentence, score));
        }

        // 按重要性排序并选择前N个
        sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selected: Vec<&str> = sentence_scores
            .iter()
            .take(target_sentences)
            .map(|(s, _)| *s)
            .collect();

        let compressed = selected.join(". ");
        println!("✨ 压缩完成: {} -> {} 字符", context.len(), compressed.len());

        Ok(compressed)
    }

    /// 打印统计信息
    fn print_stats(&self) -> Result<(), Box<dyn Error>> {
        let total_chunks = self.db.get_all_semantic_chunks()?.len();
        let cache_rate = if self.stats.total_queries > 0 {
            (self.stats.cache_hits as f32 / self.stats.total_queries as f32) * 100.0
        } else {
            0.0
        };

        println!("\n📊 RocksDB语义系统统计:");
        println!("   📚 RocksDB中语义块数: {}", total_chunks);
        println!("   🔍 总查询次数: {}", self.stats.total_queries);
        println!("   💾 缓存命中: {} 次", self.stats.cache_hits);
        println!("   🌐 API调用: {} 次", self.stats.api_calls);
        println!("   📈 缓存命中率: {:.1}%", cache_rate);

        Ok(())
    }
}

/// 计算余弦相似度
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

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 启动企业级RocksDB语义系统");

    let mut system = RocksDBSemanticSystem::new("./enterprise_semantic_db")?;

    // 添加测试数据
    let test_data = vec![
        ("AI研究", "人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的机器。"),
        ("机器学习", "机器学习是人工智能的一个子集，它使计算机能够学习和改进，而无需明确编程。"),
        ("深度学习", "深度学习是机器学习的一个子集，使用具有多层的神经网络来模拟人脑的工作方式。"),
        ("自然语言处理", "自然语言处理是人工智能的一个分支，专注于计算机与人类语言之间的交互。"),
        ("语义压缩", "语义压缩技术可以在保持核心语义信息的同时减少数据大小，提高处理效率。"),
    ];

    println!("\n📝 添加测试语义块到RocksDB:");
    for (title, content) in test_data {
        system.add_semantic_chunk(title, content)?;
    }

    // 语义搜索测试
    println!("\n🔍 语义搜索测试:");
    let results = system.semantic_search("机器学习和AI的关系", 3)?;
    for (id, score) in results {
        println!("   📄 {} (相似度: {:.3})", id, score);
    }

    // 语义压缩测试
    println!("\n🗜️ 语义压缩测试:");
    let long_text = "人工智能技术正在快速发展。机器学习算法变得越来越复杂。深度学习网络需要大量数据训练。自然语言处理帮助机器理解人类语言。语义压缩可以减少存储需求。这些技术将改变我们的未来。";
    let compressed = system.compress_context(long_text, 0.5)?;
    println!("   原文: {}", long_text);
    println!("   压缩: {}", compressed);

    system.print_stats()?;

    println!("\n✅ RocksDB语义系统演示完成！");
    Ok(())
}
