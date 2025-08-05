use crate::analyzer::PromptAnalyzer;
use crate::encoder::SimpleTextEncoder;
use prompt_compiler_weights::*;
use serde::{Deserialize, Serialize};

/// 高级分析器配置
#[derive(Debug, Clone)]
pub struct AdvancedAnalyzerConfig {
    /// 学习率
    pub learning_rate: f32,
    /// 正则化强度
    pub regularization_strength: f32,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值
    pub convergence_threshold: f32,
    /// 是否启用自适应学习率
    pub adaptive_learning_rate: bool,
}

impl Default for AdvancedAnalyzerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,  // 降低学习率
            regularization_strength: 0.05,  // 增强正则化
            max_iterations: 50,
            convergence_threshold: 0.01,
            adaptive_learning_rate: true,
        }
    }
}

/// 详细的收敛分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedConvergenceAnalysis {
    /// 每步的梯度范数
    pub gradient_norms: Vec<f32>,
    /// 每步的效果得分
    pub effectiveness_scores: Vec<f32>,
    /// 最终收敛率
    pub final_convergence_rate: f32,
    /// 是否收敛
    pub converged: bool,
    /// 收敛所需步数
    pub convergence_steps: Option<usize>,
    /// 收敛类型
    pub convergence_type: ConvergenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceType {
    /// 快速收敛
    Rapid,
    /// 平稳收敛
    Steady,
    /// 缓慢收敛
    Slow,
    /// 震荡但收敛
    Oscillating,
    /// 发散
    Diverging,
    /// 平稳但未收敛
    Stable,
}

/// 增强版prompt分析器
pub struct EnhancedPromptAnalyzer {
    base_analyzer: PromptAnalyzer,
    config: AdvancedAnalyzerConfig,
}

impl EnhancedPromptAnalyzer {
    /// 创建增强版分析器
    pub fn new(config: AdvancedAnalyzerConfig) -> Result<Self, WeightError> {
        // 创建基础分析器但使用新的配置
        let encoder = SimpleTextEncoder::new();

        let dynamics_config = DynamicsConfig {
            learning_rate: config.learning_rate,
            use_skip_connections: true,
            regularization_strength: config.regularization_strength,
        };

        let dynamics = ImplicitDynamics::new(
            encoder.prompt_feature_dim(),
            encoder.task_feature_dim(),
            dynamics_config
        )?;

        let base_analyzer = PromptAnalyzer {
            encoder,
            dynamics,
        };

        Ok(Self {
            base_analyzer,
            config,
        })
    }

    /// 深度收敛分析
    pub fn deep_convergence_analysis(
        &mut self,
        prompt: &str,
        task: &str,
    ) -> Result<DetailedConvergenceAnalysis, WeightError> {
        let context = self.base_analyzer.encoder.encode_prompt(prompt);
        let query = self.base_analyzer.encoder.encode_task(task);

        let mut gradient_norms = Vec::new();
        let mut effectiveness_scores = Vec::new();
        let mut updates = Vec::new();

        let mut current_learning_rate = self.config.learning_rate;

        println!("🔬 开始深度收敛分析");
        println!("Prompt: {}", prompt);
        println!("Task: {}", task);
        println!("初始学习率: {:.4}", current_learning_rate);
        println!("{}", "=".repeat(60));

        for step in 0..self.config.max_iterations {
            // 更新动态配置（如果启用自适应学习率）
            if self.config.adaptive_learning_rate && step > 0 {
                let recent_variance = self.calculate_gradient_variance(&gradient_norms, 3);
                if recent_variance > 0.1 {
                    current_learning_rate *= 0.9; // 减少学习率
                    self.update_learning_rate(current_learning_rate)?;
                }
            }

            // 执行权重更新
            let update = self.base_analyzer.dynamics.update_step(&context, &query)?;
            let gradient_norm = update.delta_w.norm();
            let effectiveness = update.effectiveness_score();

            gradient_norms.push(gradient_norm);
            effectiveness_scores.push(effectiveness);
            updates.push(update);

            println!("步骤 {}: 梯度范数 {:.6}, 效果得分 {:.6}, 学习率 {:.4}",
                step + 1, gradient_norm, effectiveness, current_learning_rate);

            // 检查收敛
            if gradient_norm < self.config.convergence_threshold {
                println!("✅ 在第 {} 步达到收敛！", step + 1);

                return Ok(DetailedConvergenceAnalysis {
                    gradient_norms: gradient_norms.clone(),  // 添加 clone()
                    effectiveness_scores,
                    final_convergence_rate: self.calculate_convergence_rate(&gradient_norms),
                    converged: true,
                    convergence_steps: Some(step + 1),
                    convergence_type: self.classify_convergence_type(&gradient_norms),
                });
            }

            // 检查发散
            if gradient_norm > 10.0 {
                println!("❌ 检测到发散，停止分析");
                break;
            }
        }

        println!("⏰ 达到最大迭代次数，分析结束");

        Ok(DetailedConvergenceAnalysis {
            gradient_norms: gradient_norms.clone(),
            effectiveness_scores,
            final_convergence_rate: self.calculate_convergence_rate(&gradient_norms),
            converged: false,
            convergence_steps: None,
            convergence_type: self.classify_convergence_type(&gradient_norms),
        })
    }

    /// 计算梯度方差
    fn calculate_gradient_variance(&self, gradients: &[f32], window: usize) -> f32 {
        if gradients.len() < window {
            return 0.0;
        }

        let recent: Vec<f32> = gradients.iter().rev().take(window).cloned().collect();
        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / recent.len() as f32;

        variance
    }

    /// 更新学习率
    fn update_learning_rate(&mut self, new_rate: f32) -> Result<(), WeightError> {
        // 这里需要重新创建dynamics以更新学习率
        // 这是当前实现的限制，在实际应用中应该允许动态更新
        let config = DynamicsConfig {
            learning_rate: new_rate,
            use_skip_connections: true,
            regularization_strength: self.config.regularization_strength,
        };

        self.base_analyzer.dynamics = ImplicitDynamics::new(
            self.base_analyzer.encoder.prompt_feature_dim(),
            self.base_analyzer.encoder.task_feature_dim(),
            config
        )?;

        Ok(())
    }

    /// 计算收敛率
    fn calculate_convergence_rate(&self, gradient_norms: &[f32]) -> f32 {
        if gradient_norms.len() < 4 {
            return 0.0;
        }

        let early_avg = gradient_norms.iter().take(3).sum::<f32>() / 3.0;
        let late_avg = gradient_norms.iter().rev().take(3).sum::<f32>() / 3.0;

        if early_avg > 0.0 {
            (early_avg - late_avg) / early_avg
        } else {
            0.0
        }
    }

    /// 分类收敛类型
    fn classify_convergence_type(&self, gradient_norms: &[f32]) -> ConvergenceType {
        if gradient_norms.len() < 3 {
            return ConvergenceType::Stable;
        }

        let first = gradient_norms[0];
        let last = *gradient_norms.last().unwrap();
        let variance = self.calculate_gradient_variance(gradient_norms, gradient_norms.len());

        // 发散
        if last > first * 1.5 {
            return ConvergenceType::Diverging;
        }

        // 收敛
        if last < first * 0.1 {
            if variance < 0.01 {
                return ConvergenceType::Rapid;
            } else {
                return ConvergenceType::Oscillating;
            }
        }

        // 平稳收敛
        if last < first * 0.5 && variance < 0.05 {
            return ConvergenceType::Steady;
        }

        // 缓慢收敛
        if last < first * 0.8 {
            return ConvergenceType::Slow;
        }

        ConvergenceType::Stable
    }

    /// 获取基础分析器的引用
    pub fn base_analyzer(&mut self) -> &mut PromptAnalyzer {
        &mut self.base_analyzer
    }
}
