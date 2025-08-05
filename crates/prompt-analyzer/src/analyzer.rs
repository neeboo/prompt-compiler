use prompt_compiler_weights::*;
use crate::encoder::SimpleTextEncoder;
use serde::{Serialize, Deserialize};  // 添加 serde 导入

#[derive(Debug, Clone, Serialize, Deserialize)]  // 添加 Clone, Serialize, Deserialize
pub struct PromptComparison {
    pub prompt_a_score: f32,
    pub prompt_b_score: f32,
    pub convergence_diff: f32,
    pub effectiveness_ratio: f32,
    pub winner: String,
    pub confidence: f32,
}

pub struct PromptAnalyzer {
    pub encoder: SimpleTextEncoder,  // 改为 pub
    pub dynamics: ImplicitDynamics,
}

impl PromptAnalyzer {
    pub fn new() -> Result<Self, WeightError> {
        let encoder = SimpleTextEncoder::new();

        let config = DynamicsConfig {
            learning_rate: 0.5,
            use_skip_connections: true,
            regularization_strength: 0.01,
        };

        // 使用合适的维度：prompt特征16维 -> task特征8维
        let dynamics = ImplicitDynamics::new(
            encoder.prompt_feature_dim(),
            encoder.task_feature_dim(),
            config
        )?;

        Ok(Self {
            encoder,
            dynamics,
        })
    }

    /// 比较两个prompt的效果
    pub fn compare_prompts(
        &mut self,
        prompt_a: &str,
        prompt_b: &str,
        task: &str
    ) -> Result<PromptComparison, WeightError> {
        // 编码输入
        let context_a = self.encoder.encode_prompt(prompt_a);
        let context_b = self.encoder.encode_prompt(prompt_b);
        let query = self.encoder.encode_task(task);

        // 获取初始状态快照
        let initial_weights = self.dynamics.weights().clone();

        // 测试prompt A
        let update_a = self.dynamics.update_step(&context_a, &query)?;
        let score_a = update_a.effectiveness_score();

        // 重置到初始状态
        self.reset_to_weights(&initial_weights);

        // 测试prompt B
        let update_b = self.dynamics.update_step(&context_b, &query)?;
        let score_b = update_b.effectiveness_score();

        // 计算收敛性差异
        let convergence_a = self.dynamics.predict_convergence(&[update_a]);
        let convergence_b = self.dynamics.predict_convergence(&[update_b]);

        let convergence_diff = convergence_a.convergence_rate - convergence_b.convergence_rate;

        // 计算效果比值和置信度
        let effectiveness_ratio = if score_b > 0.0 { score_a / score_b } else { score_a * 10.0 };
        let score_diff = (score_a - score_b).abs();
        let confidence = (score_diff * 10.0).min(1.0); // 简单的置信度计算

        let winner = if score_a > score_b { "A" } else { "B" }.to_string();

        Ok(PromptComparison {
            prompt_a_score: score_a,
            prompt_b_score: score_b,
            convergence_diff,
            effectiveness_ratio,
            winner,
            confidence,
        })
    }

    /// 分析单个prompt的特征
    pub fn analyze_single_prompt(&mut self, prompt: &str, task: &str) -> Result<PromptAnalysis, WeightError> {
        let context = self.encoder.encode_prompt(prompt);
        let query = self.encoder.encode_task(task);

        let update = self.dynamics.update_step(&context, &query)?;
        let convergence = self.dynamics.predict_convergence(&[update.clone()]);

        Ok(PromptAnalysis {
            effectiveness_score: update.effectiveness_score(),
            convergence_rate: convergence.convergence_rate,
            update_magnitude: update.delta_w.norm(),
            is_stable: convergence.is_converged,
        })
    }

    /// 重置权重到指定状态（用于对比测试）
    fn reset_to_weights(&mut self, weights: &nalgebra::DMatrix<f32>) {
        // 这里需要扩展ImplicitDynamics来支持重置
        // 暂时创建新实例
        let config = self.dynamics.config().clone();
        self.dynamics = ImplicitDynamics::new(
            weights.ncols(),
            weights.nrows(),
            config
        ).unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]  // 添加 Clone, Serialize, Deserialize
pub struct PromptAnalysis {
    pub effectiveness_score: f32,
    pub convergence_rate: f32,
    pub update_magnitude: f32,
    pub is_stable: bool,
}
