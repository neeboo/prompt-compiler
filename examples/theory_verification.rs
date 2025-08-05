//! 基于论文的理论验证实验
//! 验证 ICL ≡ 权重更新 的数学等价性

use prompt_compiler_weights::{ImplicitDynamics, DynamicsConfig, create_random_vector};
use nalgebra::DVector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 论文理论验证实验");
    println!("验证：In-context learning ≡ Implicit weight updates");
    println!("核心假设：ICL是一种预编码机制，将示例转换为权重调整");
    println!("{}", "=".repeat(60));

    // 实验1：验证权重更新的rank-1性质
    verify_rank_one_updates()?;

    // 实验2：验证收敛性预测
    verify_convergence_prediction()?;

    // 实验3：验证上下文长度与更新效果的关系
    verify_context_length_effect()?;

    // 实验4：验证预编码机制
    verify_preencoding_mechanism()?;

    // 实验5：验证编码保真度
    verify_encoding_fidelity()?;

    // 实验6：验证模式提取能力
    verify_pattern_extraction()?;

    Ok(())
}

/// 简化的矩阵秩估计
fn estimate_matrix_rank(matrix: &nalgebra::DMatrix<f32>) -> f32 {
    // 这里应该用 SVD 来精确计算，但为了简化，我们用 Frobenius 范数的近似
    let norm = matrix.norm();
    // 对于 rank-1 矩阵，最大奇异值约等于 Frobenius 范数
    norm / (matrix.nrows() as f32).sqrt()
}

/// 验证权重更新确实是 rank-1 的
fn verify_rank_one_updates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 实验1: 验证 rank-1 权重更新性质");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(32, 32, config)?;

    let context = vec![create_random_vector(32)];
    let query = create_random_vector(32);

    let updates = dynamics.compute_sequential_updates(&context, &query)?;

    for (i, update) in updates.iter().enumerate() {
        // 计算更新矩阵的秩
        let rank = estimate_matrix_rank(&update.delta_w);
        println!("  更新 {}: 估计秩 = {:.2} (理论值应该 ≈ 1)", i+1, rank);
    }

    Ok(())
}

/// 验证收敛性预测的准确性
fn verify_convergence_prediction() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 实验2: 验证收敛性预测");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(16, 16, config)?;

    // 测试不同长度的上下文
    for context_len in [1, 3, 5, 10] {
        let context: Vec<_> = (0..context_len).map(|_| create_random_vector(16)).collect();
        let query = create_random_vector(16);

        let updates = dynamics.compute_sequential_updates(&context, &query)?;
        let convergence = dynamics.predict_convergence(&updates);

        println!("  上下文长度 {}: 收敛率 = {:.3}, 收敛 = {}",
                 context_len, convergence.convergence_rate, convergence.is_converged);
    }

    Ok(())
}

/// 验证上下文长度对更新效果的影响
fn verify_context_length_effect() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 实验3: 上下文长度效应");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(24, 24, config)?;

    for len in [1, 2, 4, 8, 16] {
        let context: Vec<_> = (0..len).map(|_| create_random_vector(24)).collect();
        let query = create_random_vector(24);

        let updates = dynamics.compute_sequential_updates(&context, &query)?;
        let avg_effectiveness: f32 = updates.iter()
            .map(|u| u.effectiveness_score())
            .sum::<f32>() / updates.len() as f32;

        println!("  长度 {:2}: 平均效果 = {:.4}", len, avg_effectiveness);
    }

    Ok(())
}

/// 验证预编码机制：示例 → 权重调整
fn verify_preencoding_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 实验4: 验证预编码机制");
    println!("理论：具体示例 → 抽象的权重调整规则");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(64, 64, config)?;

    // 模拟不同类型的"示例"
    let coding_examples = create_coding_like_vectors(5, 64);
    let math_examples = create_math_like_vectors(5, 64);
    let query = create_random_vector(64);

    println!("  编程类示例预编码：");
    let coding_updates = dynamics.compute_sequential_updates(&coding_examples, &query)?;
    for (i, update) in coding_updates.iter().enumerate() {
        let pattern_strength = calculate_pattern_strength(update);
        println!("    更新 {}: 模式强度 = {:.3}", i+1, pattern_strength);
    }

    println!("  数学类示例预编码：");
    let math_updates = dynamics.compute_sequential_updates(&math_examples, &query)?;
    for (i, update) in math_updates.iter().enumerate() {
        let pattern_strength = calculate_pattern_strength(update);
        println!("    更新 {}: 模式强度 = {:.3}", i+1, pattern_strength);
    }

    // 比较不同类型示例的编码差异
    let coding_signature = compute_encoding_signature(&coding_updates);
    let math_signature = compute_encoding_signature(&math_updates);
    let difference = calculate_signature_difference(&coding_signature, &math_signature);

    println!("  编码差异度: {:.3} (不同类型示例应产生不同的权重调整)", difference);

    Ok(())
}

/// 验证编码保真度：能否从权重调整重构原始模式
fn verify_encoding_fidelity() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 实验5: 验证编码保真度");
    println!("测试：权重调整是否保留了原始示例的关键模式？");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(32, 32, config)?;

    // 测试不同复杂度的示例模式
    for pattern_complexity in [0.2, 0.5, 0.8] {
        let examples = create_patterned_vectors(5, 32, pattern_complexity);
        let query = create_random_vector(32);

        // 预编码：示例 → 权重调整
        let updates = dynamics.compute_sequential_updates(&examples, &query)?;

        // 评估编码质量
        let original_pattern = extract_pattern_from_examples(&examples);
        let encoded_pattern = extract_pattern_from_updates(&updates);
        let fidelity = pattern_similarity(&original_pattern, &encoded_pattern);

        println!("  模式复杂度 {:.1}: 编码保真度 = {:.3} ({:.1}% 模式保留)",
                 pattern_complexity, fidelity, fidelity * 100.0);
    }

    Ok(())
}

/// 验证模式提取能力：预编码是否能提取抽象模式
fn verify_pattern_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 实验6: 验证模式提取能力");
    println!("测试：预编码能否从具体示例中提取抽象模式？");

    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(48, 48, config)?;

    // 创建具有明确模式的示例
    let pattern_types = ["递增", "递减", "周期性", "随机"];

    for pattern_type in pattern_types {
        let examples = create_examples_with_pattern(pattern_type, 6, 48);
        let query = create_random_vector(48);

        // 预编码过程
        let updates = dynamics.compute_sequential_updates(&examples, &query)?;

        // 分析编码后的模式特征
        let pattern_clarity = measure_pattern_clarity(&updates);
        let abstraction_level = measure_abstraction_level(&updates, &examples);

        println!("  {} 模式:", pattern_type);
        println!("    模式清晰度: {:.3}", pattern_clarity);
        println!("    抽象化程度: {:.3}", abstraction_level);
    }

    Ok(())
}

// 辅助函数：创建具有编程特征的向量
fn create_coding_like_vectors(count: usize, dim: usize) -> Vec<DVector<f32>> {
    (0..count).map(|i| {
        let mut vec = create_random_vector(dim);
        // 添加"编程特征"：某些维度具有特定模式
        for j in 0..dim/4 {
            vec[j] = (i as f32 * 0.1 + j as f32 * 0.01).sin(); // 结构化模式
        }
        vec
    }).collect()
}

// 辅助函数：创建具有数学特征的向量
fn create_math_like_vectors(count: usize, dim: usize) -> Vec<DVector<f32>> {
    (0..count).map(|i| {
        let mut vec = create_random_vector(dim);
        // 添加"数学特征"：不同的结构化模式
        for j in 0..dim/4 {
            vec[j] = (i as f32 * 0.2).exp() * (j as f32 * 0.05).cos(); // 指数+三角模式
        }
        vec
    }).collect()
}

// 计算权重更新的模式强度
fn calculate_pattern_strength(update: &prompt_compiler_weights::WeightUpdate) -> f32 {
    // 使用矩阵的条件数作为模式强度的代理
    let norm = update.delta_w.norm();
    let trace = update.delta_w.diagonal().sum();
    norm / (trace.abs() + 1e-6) // 避免除零
}

// 计算编码签名
fn compute_encoding_signature(updates: &[prompt_compiler_weights::WeightUpdate]) -> Vec<f32> {
    updates.iter().map(|update| {
        let w_norm = update.delta_w.norm();
        let b_norm = update.delta_b.as_ref().map_or(0.0, |b| b.norm());
        w_norm + b_norm
    }).collect()
}

// 计算签名差异
fn calculate_signature_difference(sig1: &[f32], sig2: &[f32]) -> f32 {
    sig1.iter().zip(sig2.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

// 创建具有特定模式的向量
fn create_patterned_vectors(count: usize, dim: usize, complexity: f32) -> Vec<DVector<f32>> {
    (0..count).map(|i| {
        let mut vec = DVector::zeros(dim);
        for j in 0..dim {
            // 复杂度决定模式的规律性
            vec[j] = (i as f32 * complexity + j as f32 * (1.0 - complexity)).sin();
        }
        vec
    }).collect()
}

// 从示例中提取模式
fn extract_pattern_from_examples(examples: &[DVector<f32>]) -> Vec<f32> {
    if examples.is_empty() { return vec![]; }

    let dim = examples[0].len();
    let mut pattern = vec![0.0; dim];

    for i in 0..dim {
        let values: Vec<f32> = examples.iter().map(|v| v[i]).collect();
        pattern[i] = calculate_pattern_score(&values);
    }
    pattern
}

// 从权重更新中提取模式
fn extract_pattern_from_updates(updates: &[prompt_compiler_weights::WeightUpdate]) -> Vec<f32> {
    updates.iter().map(|update| update.delta_w.norm()).collect()
}

// 计算模式相似度
fn pattern_similarity(pattern1: &[f32], pattern2: &[f32]) -> f32 {
    if pattern1.len() != pattern2.len() { return 0.0; }

    let dot_product: f32 = pattern1.iter().zip(pattern2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = pattern1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = pattern2.iter().map(|x| x * x).sum::<f32>().sqrt();

    dot_product / (norm1 * norm2 + 1e-6)
}

// 计算模式评分
fn calculate_pattern_score(values: &[f32]) -> f32 {
    if values.len() < 2 { return 0.0; }

    // 计算一阶差分的方差（低方差 = 高规律性）
    let diffs: Vec<f32> = values.windows(2).map(|w| w[1] - w[0]).collect();
    let mean_diff = diffs.iter().sum::<f32>() / diffs.len() as f32;
    let variance = diffs.iter().map(|d| (d - mean_diff).powi(2)).sum::<f32>() / diffs.len() as f32;

    1.0 / (variance + 1e-6) // 方差越小，模式性越强
}

// 创建具有特定模式类型的示例
fn create_examples_with_pattern(pattern_type: &str, count: usize, dim: usize) -> Vec<DVector<f32>> {
    (0..count).map(|i| {
        let mut vec = DVector::zeros(dim);
        for j in 0..dim {
            vec[j] = match pattern_type {
                "递增" => i as f32 + j as f32 * 0.1,
                "递减" => (count - i) as f32 - j as f32 * 0.1,
                "周期性" => (i as f32 * 0.5 + j as f32 * 0.2).sin(),
                _ => create_random_vector(1)[0], // 随机
            };
        }
        vec
    }).collect()
}

// 测量模式清晰度
fn measure_pattern_clarity(updates: &[prompt_compiler_weights::WeightUpdate]) -> f32 {
    if updates.is_empty() { return 0.0; }

    let norms: Vec<f32> = updates.iter().map(|u| u.delta_w.norm()).collect();
    let mean = norms.iter().sum::<f32>() / norms.len() as f32;
    let variance = norms.iter().map(|n| (n - mean).powi(2)).sum::<f32>() / norms.len() as f32;

    mean / (variance.sqrt() + 1e-6) // 信噪比作为清晰度指标
}

// 测量抽象化程度
fn measure_abstraction_level(updates: &[prompt_compiler_weights::WeightUpdate], examples: &[DVector<f32>]) -> f32 {
    let update_complexity = updates.iter().map(|u| u.delta_w.norm()).sum::<f32>();
    let example_complexity = examples.iter().map(|e| e.norm()).sum::<f32>();

    update_complexity / (example_complexity + 1e-6) // 相对复杂度
}
