//! Weight-based optimizer using ICL theory

use crate::compiler::{PromptOptimizer, PromptIR};
use crate::error::Result;
use prompt_compiler_weights::{ImplicitDynamics, DynamicsConfig, create_random_vector};
use nalgebra::DVector;

/// Prompt 意图分类
#[derive(Debug, Clone)]
enum PromptIntent {
    Coding,
    Explanation,
    Analysis,
    General,
}

/// 基于权重动态的优化器
/// 实现论文 "Learning without training: The implicit dynamics of in-context learning" 的理论
pub struct WeightOptimizer {
    dynamics: ImplicitDynamics,
    convergence_threshold: f32,
}

impl WeightOptimizer {
    pub fn new() -> Result<Self> {
        let config = DynamicsConfig::default();
        let dynamics = ImplicitDynamics::new(64, 64, config)?;

        Ok(Self {
            dynamics,
            convergence_threshold: 0.8,
        })
    }

    /// 基于权重动态分析优化 prompt 结构
    fn optimize_with_weight_analysis(&mut self, ir: &PromptIR) -> Result<PromptIR> {
        // 1. 将 prompt 转换为上下文向量序列
        let context_vectors = self.prompt_to_vectors(&ir.original_content)?;
        let query_vector = create_random_vector(64); // 模拟用户意图

        // 2. 计算权重更新序列
        let updates = self.dynamics.compute_sequential_updates(&context_vectors, &query_vector)?;

        // 3. 分析收敛性 - 使用简化的收敛计算
        let convergence = self.calculate_convergence(&updates);

        // 4. 基于收敛性和内容分析调整 prompt 结构
        let mut optimized_ir = ir.clone();
        
        // 使用智能优化替代硬编码模板
        optimized_ir.compiled_content = self.intelligent_optimize(&ir.original_content, convergence);

        if convergence < self.convergence_threshold {
            optimized_ir.compilation_metadata.insert(
                "weight_optimization".to_string(),
                format!("Enhanced structure due to low convergence: {:.3}", convergence)
            );
        } else {
            optimized_ir.compilation_metadata.insert(
                "weight_optimization".to_string(),
                format!("Refined with good convergence: {:.3}", convergence)
            );
        }

        // 5. 记录权重分析结果
        optimized_ir.compilation_metadata.insert(
            "convergence_rate".to_string(),
            convergence.to_string()
        );
        optimized_ir.compilation_metadata.insert(
            "weight_updates_count".to_string(),
            updates.len().to_string()
        );

        Ok(optimized_ir)
    }

    /// 将 prompt 转换为向量序列（使用 nalgebra DVector）
    fn prompt_to_vectors(&self, prompt: &str) -> Result<Vec<DVector<f32>>> {
        let words: Vec<&str> = prompt.split_whitespace().collect();
        let mut vectors = Vec::new();

        for word in words.iter().take(10) { // 限制上下文长度
            // 简化的词向量化：基于词长度和字符特征
            let mut vector = create_random_vector(64);

            // 基于词的特征调整向量
            let word_hash = word.chars().map(|c| c as u32).sum::<u32>() as f32;
            let length_factor = word.len() as f32 / 10.0;

            for i in 0..vector.len() {
                vector[i] = (vector[i] + word_hash / 1000.0 + length_factor).tanh();
            }

            vectors.push(vector);
        }

        if vectors.is_empty() {
            vectors.push(create_random_vector(64));
        }

        Ok(vectors)
    }

    /// 简化的收敛性计算
    fn calculate_convergence(&self, updates: &[prompt_compiler_weights::WeightUpdate]) -> f32 {
        if updates.is_empty() {
            return 0.0;
        }

        // 计算权重更新的方差作为收敛指标
        let norms: Vec<f32> = updates.iter().filter_map(|update| {
            // 安全地处理可能为 None 的权重更新
            let delta_w_norm = update.delta_w.norm();
            let delta_b_norm = update.delta_b.as_ref()?.norm();
            Some(delta_w_norm + delta_b_norm)
        }).collect();

        if norms.len() < 2 {
            return 0.5; // 默认中等收敛
        }

        let mean = norms.iter().sum::<f32>() / norms.len() as f32;
        let variance = norms.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / norms.len() as f32;

        // 低方差表示高收敛性
        let convergence: f32 = (1.0 - variance.sqrt()).max(0.0).min(1.0);
        convergence
    }

    /// 增强 prompt 结构（用于低收敛情况）
    fn enhance_prompt_structure(&self, original: &str) -> String {
        format!(r#"## 任务描述
{}

## 执行要求
- 请提供清晰、结构化的回答
- 包含具体的实现步骤
- 如有必要，提供示例代码

## 输出格式
请按照以下格式组织回答：

1. **概述**: 简要说明解决方案
2. **详细实现**: 具体的实现内容
3. **示例**: 相关的使用示例
4. **注意事项**: 重要的注意点或限制

## 质量标准
- 准确性：确保信息正确无误
- 完整性：涵盖所有重要方面
- 清晰性：表达简洁明了"#, original.trim())
    }

    /// 基于内容分析的智能优化（替代硬编码模板）
    fn intelligent_optimize(&self, original: &str, convergence: f32) -> String {
        // 分析 prompt 的实际意图和类型
        let intent = self.analyze_prompt_intent(original);
        let length = original.len();

        match (intent, length, convergence) {
            // 代码相关请求
            (PromptIntent::Coding, len, _) if len < 20 => {
                self.enhance_coding_prompt(original)
            }
            // 解释说明请求
            (PromptIntent::Explanation, len, _) if len < 30 => {
                self.enhance_explanation_prompt(original)
            }
            // 分析讨论请求
            (PromptIntent::Analysis, len, _) if len < 25 => {
                self.enhance_analysis_prompt(original)
            }
            // 对于已经有一定长度和结构的 prompt，只做轻度优化
            (_, len, conv) if len > 50 && conv > 0.6 => {
                self.light_optimize(original)
            }
            // 默认：添加基本结构
            _ => {
                self.add_basic_structure(original)
            }
        }
    }

    /// 分析 prompt 意图
    fn analyze_prompt_intent(&self, prompt: &str) -> PromptIntent {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("代码") || prompt_lower.contains("code") ||
           prompt_lower.contains("程序") || prompt_lower.contains("algorithm") {
            PromptIntent::Coding
        } else if prompt_lower.contains("解释") || prompt_lower.contains("explain") ||
                  prompt_lower.contains("什么是") || prompt_lower.contains("how") {
            PromptIntent::Explanation
        } else if prompt_lower.contains("分析") || prompt_lower.contains("analyze") ||
                  prompt_lower.contains("比较") || prompt_lower.contains("evaluate") {
            PromptIntent::Analysis
        } else {
            PromptIntent::General
        }
    }

    /// 针对编程请求的优化
    fn enhance_coding_prompt(&self, original: &str) -> String {
        format!("请帮助{}，要求：\n1. 提供完整可运行的代码\n2. 包含必要的注释说明\n3. 如果可能，给出使用示例", original.trim())
    }

    /// 针对解释请求的优化
    fn enhance_explanation_prompt(&self, original: &str) -> String {
        format!("请详细{}，包括：\n1. 核心概念定义\n2. 工作原理说明\n3. 实际应用场景", original.trim())
    }

    /// 针对分析请求的优化
    fn enhance_analysis_prompt(&self, original: &str) -> String {
        format!("请深入{}，从以下角度：\n1. 关键要素识别\n2. 优缺点对比\n3. 结论和建议", original.trim())
    }

    /// 轻度优化
    fn light_optimize(&self, original: &str) -> String {
        if !original.contains("请") && !original.contains("help") {
            format!("请{}", original.trim())
        } else {
            format!("{}，请确保回答准确详细。", original.trim())
        }
    }

    /// 添加基本结构
    fn add_basic_structure(&self, original: &str) -> String {
        format!("任务：{}\n要求：请提供清晰、有用的回答。", original.trim())
    }

    /// 生成注入空间的上下文信息
    pub fn create_injection_context(&self, ir: &PromptIR, convergence: f32) -> InjectionContext {
        let intent = self.analyze_prompt_intent(&ir.original_content);
        let semantic_vectors = self.prompt_to_vectors(&ir.original_content).unwrap_or_default();
        
        InjectionContext {
            original_query: ir.original_content.clone(),
            optimized_prompt: ir.compiled_content.clone(),
            
            // 语义空间信息
            semantic_analysis: SemanticAnalysis {
                intent_classification: intent.clone(),
                complexity_score: ir.original_content.len() as f32 / 100.0,
                context_vectors: semantic_vectors,
            },
            
            // 权重动态分析
            weight_dynamics: WeightDynamicsInfo {
                convergence_rate: convergence,
                optimization_strategy: self.get_optimization_strategy(&intent, convergence),
                confidence_score: self.calculate_confidence(convergence),
            },
            
            // 推理指导信息
            reasoning_guidance: ReasoningGuidance {
                focus_areas: self.extract_focus_areas(&intent),
                response_structure: self.suggest_response_structure(&intent),
                quality_criteria: self.define_quality_criteria(&intent),
            }
        }
    }
    
    fn get_optimization_strategy(&self, intent: &PromptIntent, convergence: f32) -> String {
        match (intent, convergence) {
            (PromptIntent::Coding, conv) if conv < 0.6 => "deep_structure_enhancement".to_string(),
            (PromptIntent::Coding, _) => "coding_best_practices".to_string(),
            (PromptIntent::Explanation, conv) if conv < 0.7 => "conceptual_framework".to_string(),
            _ => "general_optimization".to_string()
        }
    }
    
    fn calculate_confidence(&self, convergence: f32) -> f32 {
        // 基于收敛性计算我们对优化效果的信心
        if convergence > 0.8 { 0.9 }
        else if convergence > 0.6 { 0.7 }
        else { 0.5 }
    }
    
    fn extract_focus_areas(&self, intent: &PromptIntent) -> Vec<String> {
        match intent {
            PromptIntent::Coding => vec![
                "代码完整性".to_string(),
                "最佳实践".to_string(),
                "可读性".to_string(),
                "示例演示".to_string()
            ],
            PromptIntent::Explanation => vec![
                "概念清晰度".to_string(),
                "逻辑结构".to_string(),
                "实际应用".to_string()
            ],
            PromptIntent::Analysis => vec![
                "多角度分析".to_string(),
                "批判性思维".to_string(),
                "结论导出".to_string()
            ],
            PromptIntent::General => vec![
                "准确性".to_string(),
                "完整性".to_string()
            ]
        }
    }
    
    fn suggest_response_structure(&self, intent: &PromptIntent) -> ResponseStructure {
        match intent {
            PromptIntent::Coding => ResponseStructure {
                sections: vec![
                    "代码实现".to_string(),
                    "关键注释".to_string(),
                    "使用示例".to_string(),
                    "注意事项".to_string()
                ],
                preferred_format: "code_with_explanation".to_string()
            },
            PromptIntent::Explanation => ResponseStructure {
                sections: vec![
                    "核心概念".to_string(),
                    "工作原理".to_string(),
                    "应用场景".to_string()
                ],
                preferred_format: "structured_explanation".to_string()
            },
            _ => ResponseStructure {
                sections: vec!["主要内容".to_string()],
                preferred_format: "natural".to_string()
            }
        }
    }
    
    fn define_quality_criteria(&self, intent: &PromptIntent) -> Vec<QualityCriterion> {
        match intent {
            PromptIntent::Coding => vec![
                QualityCriterion { name: "可运行性".to_string(), weight: 0.4 },
                QualityCriterion { name: "代码质量".to_string(), weight: 0.3 },
                QualityCriterion { name: "文档完整性".to_string(), weight: 0.3 },
            ],
            PromptIntent::Explanation => vec![
                QualityCriterion { name: "概念准确性".to_string(), weight: 0.5 },
                QualityCriterion { name: "逻辑清晰度".to_string(), weight: 0.3 },
                QualityCriterion { name: "实用性".to_string(), weight: 0.2 },
            ],
            _ => vec![
                QualityCriterion { name: "准确性".to_string(), weight: 0.6 },
                QualityCriterion { name: "完整性".to_string(), weight: 0.4 },
            ]
        }
    }
}

impl PromptOptimizer for WeightOptimizer {
    fn optimize(&self, ir: &PromptIR) -> Result<PromptIR> {
        let mut optimizer = WeightOptimizer::new()?;
        optimizer.optimize_with_weight_analysis(ir)
    }
}

/// 注入空间的上下文信息
#[derive(Debug, Clone)]
pub struct InjectionContext {
    pub original_query: String,
    pub optimized_prompt: String,
    pub semantic_analysis: SemanticAnalysis,
    pub weight_dynamics: WeightDynamicsInfo,
    pub reasoning_guidance: ReasoningGuidance,
}

#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    pub intent_classification: PromptIntent,
    pub complexity_score: f32,
    pub context_vectors: Vec<DVector<f32>>,
}

#[derive(Debug, Clone)]
pub struct WeightDynamicsInfo {
    pub convergence_rate: f32,
    pub optimization_strategy: String,
    pub confidence_score: f32,
}

#[derive(Debug, Clone)]
pub struct ReasoningGuidance {
    pub focus_areas: Vec<String>,
    pub response_structure: ResponseStructure,
    pub quality_criteria: Vec<QualityCriterion>,
}

#[derive(Debug, Clone)]
pub struct ResponseStructure {
    pub sections: Vec<String>,
    pub preferred_format: String,
}

#[derive(Debug, Clone)]
pub struct QualityCriterion {
    pub name: String,
    pub weight: f32,
}
