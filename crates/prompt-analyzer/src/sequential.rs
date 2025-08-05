use crate::analyzer::{PromptAnalyzer, PromptAnalysis};
use crate::encoder::SimpleTextEncoder;
use prompt_compiler_weights::*;
use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStep {
    pub step_number: usize,
    pub prompt: String,
    pub analysis: PromptAnalysis,
    pub improvement_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationHistory {
    pub original_prompt: String,
    pub task: String,
    pub steps: Vec<OptimizationStep>,
    pub final_convergence_rate: f32,
    pub total_improvement: f32,
}

pub struct SequentialPromptAnalyzer {
    analyzer: PromptAnalyzer,
    encoder: SimpleTextEncoder,
}

impl SequentialPromptAnalyzer {
    pub fn new() -> Result<Self, WeightError> {
        Ok(Self {
            analyzer: PromptAnalyzer::new()?,
            encoder: SimpleTextEncoder::new(),
        })
    }

    /// 执行迭代式 prompt 优化分析
    pub fn optimize_iteratively(
        &mut self,
        original_prompt: &str,
        task: &str,
        max_steps: usize,
    ) -> Result<OptimizationHistory, WeightError> {
        let mut steps = Vec::new();
        let mut current_prompt = original_prompt.to_string();
        let mut all_updates = Vec::new();

        println!("🚀 开始迭代式 Prompt 优化分析");
        println!("原始任务: {}", task);
        println!("原始 Prompt: {}", original_prompt);
        println!("{}", "=".repeat(60));

        for step in 0..max_steps {
            // 分析当前 prompt
            let analysis = self.analyzer.analyze_single_prompt(&current_prompt, task)?;

            // 生成改进建议
            let suggestions = self.generate_improvement_suggestions(&current_prompt, &analysis);

            // 记录这一步
            let step_data = OptimizationStep {
                step_number: step + 1,
                prompt: current_prompt.clone(),
                analysis: analysis.clone(),
                improvement_suggestions: suggestions.clone(),
            };
            steps.push(step_data);

            // 打印当前步骤结果
            println!("\n📊 第 {} 步分析结果:", step + 1);
            println!("当前 Prompt: {}", current_prompt);
            println!("效果得分: {:.4}", analysis.effectiveness_score);
            println!("更新幅度: {:.4}", analysis.update_magnitude);
            println!("是否稳定: {}", if analysis.is_stable { "✅" } else { "❌" });

            // 收集权重更新用于收敛分析
            let context = self.encoder.encode_prompt(&current_prompt);
            let query = self.encoder.encode_task(task);
            let update = self.analyzer.dynamics.update_step(&context, &query)?;
            all_updates.push(update);

            // 如果已经收敛，提前结束
            if analysis.is_stable && step > 2 {
                println!("✅ 已达到稳定状态，提前结束优化");
                break;
            }

            // 应用改进建议生成下一个 prompt
            if step < max_steps - 1 {
                current_prompt = self.apply_improvements(&current_prompt, &suggestions);
                println!("📝 改进建议: {:?}", suggestions);
                println!("🔄 下一步 Prompt: {}", current_prompt);
            }
        }

        // 计算最终收敛率
        let convergence_metrics = self.analyzer.dynamics.predict_convergence(&all_updates);
        let final_convergence_rate = convergence_metrics.convergence_rate;

        // 计算总体改进
        let initial_score = steps.first().map(|s| s.analysis.effectiveness_score).unwrap_or(0.0);
        let final_score = steps.last().map(|s| s.analysis.effectiveness_score).unwrap_or(0.0);
        let total_improvement = ((final_score - initial_score) / initial_score.max(0.001)) * 100.0;

        println!("\n🎯 优化总结:");
        println!("最终收敛率: {:.4}", final_convergence_rate);
        println!("总体改进: {:.1}%", total_improvement);
        println!("梯度历史: {:?}", convergence_metrics.gradient_norms);

        Ok(OptimizationHistory {
            original_prompt: original_prompt.to_string(),
            task: task.to_string(),
            steps,
            final_convergence_rate,
            total_improvement,
        })
    }

    /// 生成改进建议
    fn generate_improvement_suggestions(&self, prompt: &str, analysis: &PromptAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 基于效果得分给出建议
        if analysis.effectiveness_score < 0.3 {
            suggestions.push("添加更明确的指令结构".to_string());
            suggestions.push("使用\"请按照以下步骤\"等引导词".to_string());
        }

        // 基于更新幅度给出建议
        if analysis.update_magnitude > 1.0 {
            suggestions.push("简化prompt，避免过于复杂".to_string());
        }

        // 检查prompt中缺失的关键元素
        if !prompt.contains("请") && !prompt.contains("麻烦") {
            suggestions.push("添加礼貌用语增强引导性".to_string());
        }

        if !prompt.contains("详细") && !prompt.contains("具体") {
            suggestions.push("添加\"详细\"或\"具体\"要求明确性".to_string());
        }

        if !prompt.contains("步骤") && !prompt.contains("按照") {
            suggestions.push("添加步骤化指令提高结构性".to_string());
        }

        // 如果没有其他建议，给出通用建议
        if suggestions.is_empty() {
            suggestions.push("当前prompt已经比较优秀".to_string());
        }

        suggestions
    }

    /// 应用改进建议
    fn apply_improvements(&self, original: &str, suggestions: &[String]) -> String {
        let mut improved = original.to_string();

        for suggestion in suggestions {
            if suggestion.contains("请按照以下步骤") && !improved.contains("步骤") {
                improved = format!("请按照以下步骤{}：1) 理解要求 2) 分析问题 3) 给出结果", improved);
            } else if suggestion.contains("详细") && !improved.contains("详细") {
                improved = format!("请详细{}", improved);
            } else if suggestion.contains("礼貌用语") && !improved.contains("请") {
                improved = format!("请{}", improved);
            } else if suggestion.contains("具体") && !improved.contains("具体") {
                improved = improved.replace("分析", "具体分析");
            }
        }

        improved
    }
}
