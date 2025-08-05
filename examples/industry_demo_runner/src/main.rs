use std::collections::HashMap;
use std::error::Error;

/// 简化的 Embedding Provider trait
trait EmbeddingProvider {
    fn embed(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>>;
    fn embed_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn Error>>;
    fn dimension(&self) -> usize;
    fn model_info(&self) -> String;
}

/// Mock Embedding Provider for demonstration
struct MockEmbeddingProvider {
    dimension: usize,
    cache: HashMap<String, Vec<f32>>,
}

impl MockEmbeddingProvider {
    fn new(dimension: usize) -> Self {
        Self {
            dimension,
            cache: HashMap::new(),
        }
    }
}

impl EmbeddingProvider for MockEmbeddingProvider {
    fn embed(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        // 检查缓存
        if let Some(cached) = self.cache.get(text) {
            return Ok(cached.clone());
        }

        // 模拟基于文本内容的embedding生成
        let mut embedding = vec![0.0; self.dimension];
        let bytes = text.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            let index = i % self.dimension;
            embedding[index] += (byte as f32) / 255.0;
        }

        // 归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        // 缓存结果
        self.cache.insert(text.to_string(), embedding.clone());
        Ok(embedding)
    }

    fn embed_batch(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        texts.iter().map(|&text| self.embed(text)).collect()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_info(&self) -> String {
        format!("MockEmbedding-{}", self.dimension)
    }
}

/// 语义相似度计算
fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot_product / (norm_a * norm_b) }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🏭 行业标准Embedding库集成演示");
    println!("==========================================");

    // 1. 初始化模型
    println!("✅ 正在初始化Mock Embedding模型...");
    let mut provider = MockEmbeddingProvider::new(384);
    println!("✅ 模型初始化完成: {}, 维度: {}", provider.model_info(), provider.dimension());

    // 2. 准备测试数据
    println!("\n📦 处理知识片段...");
    let knowledge_base = vec![
        ("人工智能发展史",
         "人工智能的发展可以追溯到1950年代，艾伦·图灵提出了著名的图灵测试。1956年达特茅斯会议标志着AI领域的正式诞生。"),

        ("区块链技术革命",
         "区块链技术起源于2008年中本聪发布的比特币白皮书。作为一种去中心化的分布式账本技术，区块链通过密码学保证数据不可篡改。"),

        ("量子计算突破",
         "量子计算利用量子力学原理进行信息处理，具有解决某些问题的指数级加速能力。IBM、Google等公司在量子优势方面取得重要进展。"),

        ("云计算演进",
         "云计算将计算资源虚拟化并通过网络提供服务。从IaaS到PaaS再到SaaS，云计算模式不断演进，推动了数字化转型。"),

        ("5G网络部署",
         "5G网络提供超高速、低延迟的移动通信能力。毫米波技术、大规模MIMO、网络切片等关键技术支撑了5G的商用部署。")
    ];

    // 3. 生成embeddings
    println!("🔄 生成知识片段的embeddings...");
    let mut embeddings = Vec::new();

    for (title, content) in &knowledge_base {
        let embedding = provider.embed(content)?;
        embeddings.push((title.clone(), embedding));
        println!("   ✓ 已处理: {}", title);
    }

    // 4. 语义相似度演示
    println!("\n🔍 语义相似度分析:");
    println!("查询: '深度学习和神经网络的发展'");

    let query = "深度学习和神经网络的发展趋势如何";
    let query_embedding = provider.embed(query)?;

    let mut similarities = Vec::new();
    for (title, embedding) in &embeddings {
        let similarity = calculate_cosine_similarity(&query_embedding, embedding);
        similarities.push((title, similarity));
    }

    // 按相似度排序
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, (title, similarity)) in similarities.iter().enumerate() {
        println!("   {}. {} - 相似度: {:.3}", i + 1, title, similarity);
    }

    // 5. 批量处理演示
    println!("\n⚡ 批量处理性能测试:");
    let test_texts: Vec<&str> = knowledge_base.iter().map(|(_, content)| *content).collect();

    let start = std::time::Instant::now();
    let batch_embeddings = provider.embed_batch(&test_texts)?;
    let duration = start.elapsed();

    println!("   📊 批量处理 {} 个文本", test_texts.len());
    println!("   ⏱️  耗时: {:?}", duration);
    println!("   🚀 吞吐量: {:.1} 文本/秒", test_texts.len() as f64 / duration.as_secs_f64());

    // 6. 缓存效果演示
    println!("\n💾 缓存性能测试:");
    let cache_test_text = knowledge_base[0].1;

    // 第一次调用 (无缓存)
    let start = std::time::Instant::now();
    let _ = provider.embed(cache_test_text)?;
    let no_cache_time = start.elapsed();

    // 第二次调用 (有缓存)
    let start = std::time::Instant::now();
    let _ = provider.embed(cache_test_text)?;
    let cache_time = start.elapsed();

    let speedup = no_cache_time.as_nanos() as f64 / cache_time.as_nanos() as f64;

    println!("   🔄 首次计算: {:?}", no_cache_time);
    println!("   ⚡ 缓存命中: {:?}", cache_time);
    println!("   📈 加速比: {:.1}x", speedup);

    // 7. 模型推荐
    println!("\n💡 模型选择建议:");
    match provider.dimension() {
        384 => println!("   🎯 384维模型: 适合快速检索和实时应用"),
        768 => println!("   🎯 768维模型: 平衡精度和性能，适合通用场景"),
        1536 => println!("   🎯 1536维模型: 高精度语义理解，适合复杂任务"),
        _ => println!("   🎯 自定义维度: 根据具体需求调优"),
    }

    println!("\n✅ 演示完成!");
    println!("💡 提示: 在生产环境中可以使用 rust-bert、candle 等库加载真实的预训练模型");

    Ok(())
}
