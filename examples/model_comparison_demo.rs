use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs;

/// 简单的 .env 文件解析器
fn load_dotenv() -> Result<(), Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                env::set_var(key, value);
            }
        }
    }
    Ok(())
}

/// OpenAI 模型对比演示
struct ModelComparison {
    models: Vec<ModelInfo>,
}

struct ModelInfo {
    name: String,
    dimension: usize,
    cost_per_1k_tokens: f32,
    use_cases: Vec<String>,
    performance_score: f32,
}

impl ModelComparison {
    fn new() -> Self {
        Self {
            models: vec![
                ModelInfo {
                    name: "text-embedding-3-small".to_string(),
                    dimension: 1536,
                    cost_per_1k_tokens: 0.00002, // $0.00002 per 1K tokens
                    use_cases: vec![
                        "大规模文档检索".to_string(),
                        "实时语义搜索".to_string(),
                        "聊天机器人知识库".to_string(),
                        "内容推荐系统".to_string(),
                    ],
                    performance_score: 85.0,
                },
                ModelInfo {
                    name: "text-embedding-3-large".to_string(),
                    dimension: 3072,
                    cost_per_1k_tokens: 0.00013, // $0.00013 per 1K tokens
                    use_cases: vec![
                        "法律文档分析".to_string(),
                        "学术论文检索".to_string(),
                        "医疗知识库".to_string(),
                        "高精度语义匹配".to_string(),
                    ],
                    performance_score: 95.0,
                },
                ModelInfo {
                    name: "text-embedding-ada-002".to_string(),
                    dimension: 1536,
                    cost_per_1k_tokens: 0.0001, // $0.0001 per 1K tokens (Legacy)
                    use_cases: vec![
                        "传统应用迁移".to_string(),
                        "预算有限项目".to_string(),
                    ],
                    performance_score: 75.0,
                },
            ],
        }
    }

    fn compare_models(&self) {
        println!("📊 OpenAI Embedding 模型深度对比");
        println!("{}", "=".repeat(60));

        for (i, model) in self.models.iter().enumerate() {
            println!("\n{}. 🤖 {}", i + 1, model.name);
            println!("   📏 向量维度: {} 维", model.dimension);
            println!("   💰 成本: ${:.5} / 1K tokens", model.cost_per_1k_tokens);
            println!("   🎯 性能评分: {}/100", model.performance_score);
            println!("   🔧 适用场景:");
            for use_case in &model.use_cases {
                println!("      • {}", use_case);
            }

            // 成本效益分析
            let cost_efficiency = model.performance_score / (model.cost_per_1k_tokens * 100000.0);
            println!("   📈 性价比指数: {:.1}", cost_efficiency);
        }
    }

    fn recommend_model(&self, use_case: &str) -> &ModelInfo {
        match use_case {
            "high_precision" => &self.models[1], // text-embedding-3-large
            "cost_effective" => &self.models[0], // text-embedding-3-small
            "legacy" => &self.models[2],         // text-embedding-ada-002
            _ => &self.models[0], // 默认推荐 small
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 OpenAI Embedding 模型对比演示");
    println!("=====================================");

    // 1. 加载当前配置
    load_dotenv()?;
    let current_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());

    println!("📋 当前配置模型: {}", current_model);

    // 2. 模型对比分析
    let comparison = ModelComparison::new();
    comparison.compare_models();

    // 3. 场景化推荐
    println!("\n💡 场景化模型推荐:");
    println!("{}", "=".repeat(40));

    let scenarios = vec![
        ("高精度语义分析", "high_precision"),
        ("成本效益优先", "cost_effective"),
        ("传统系统兼容", "legacy"),
    ];

    for (scenario, use_case) in scenarios {
        let recommended = comparison.recommend_model(use_case);
        println!("\n🎯 场景: {}", scenario);
        println!("   推荐模型: {}", recommended.name);
        println!("   理由: {} 维度, 性能 {}/100, 成本 ${:.5}/1K",
                 recommended.dimension, recommended.performance_score, recommended.cost_per_1k_tokens);
    }

    // 4. 成本计算器
    println!("\n💰 成本估算 (处理100万tokens):");
    println!("{}", "=".repeat(40));

    let tokens_1m = 1_000_000.0;
    for model in &comparison.models {
        let cost_1m = (tokens_1m / 1000.0) * model.cost_per_1k_tokens;
        println!("   • {}: ${:.2}", model.name, cost_1m);
    }

    // 5. 当前模型详细信息
    if let Some(current) = comparison.models.iter().find(|m| m.name == current_model) {
        println!("\n🔍 当前使用模型详情:");
        println!("{}", "=".repeat(30));
        println!("   📱 模型: {}", current.name);
        println!("   📏 维度: {} 维 ({})",
                 current.dimension,
                 if current.dimension >= 3000 { "高精度" } else { "标准精度" });
        println!("   💰 成本: ${:.5}/1K tokens", current.cost_per_1k_tokens);
        println!("   🎯 性能: {}/100", current.performance_score);

        // 存储需求估算
        let storage_per_vector_mb = (current.dimension * 4) as f64 / 1_000_000.0; // 4 bytes per float32
        println!("   💾 存储需求: {:.3} MB/百万向量", storage_per_vector_mb * 1_000_000.0);
    }

    // 6. 切换建议
    println!("\n🔄 模型切换指南:");
    println!("{}", "=".repeat(25));
    println!("   修改 .env 文件中的 OPENAI_MODEL 配置:");
    println!("   • 高精度: OPENAI_MODEL=text-embedding-3-large");
    println!("   • 标准版: OPENAI_MODEL=text-embedding-3-small");
    println!("   • 传统版: OPENAI_MODEL=text-embedding-ada-002");

    println!("\n✨ 当前正在使用: {} 🎉", current_model);
    if current_model == "text-embedding-3-large" {
        println!("🌟 恭喜！你正在使用最高精度模型，适合复杂语义任务！");
    }

    Ok(())
}
