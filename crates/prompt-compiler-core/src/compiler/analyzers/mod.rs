//! Semantic analyzer for prompt content

use crate::compiler::{PromptAnalyzer, AnalysisResult};
use crate::error::Result;

/// 语义分析器 - 分析 prompt 的语义质量
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl PromptAnalyzer for SemanticAnalyzer {
    fn analyze(&self, prompt: &str) -> Result<AnalysisResult> {
        let intent_clarity = analyze_intent_clarity(prompt);
        let context_relevance = analyze_context_relevance(prompt);
        let conflicts = detect_constraint_conflicts(prompt);
        let optimizations = suggest_optimizations(prompt);

        Ok(AnalysisResult {
            intent_clarity,
            context_relevance,
            constraint_conflicts: conflicts,
            suggested_optimizations: optimizations,
        })
    }
}

/// 分析意图清晰度
fn analyze_intent_clarity(prompt: &str) -> f32 {
    let mut score: f32 = 0.0;

    // 检查动作词汇
    let action_words = ["write", "create", "generate", "analyze", "explain", "solve"];
    let has_action = action_words.iter().any(|&word| prompt.to_lowercase().contains(word));
    if has_action {
        score += 0.3;
    }

    // 检查具体性
    if prompt.len() > 20 {
        score += 0.2;
    }
    if prompt.contains("example") || prompt.contains("format") {
        score += 0.2;
    }

    // 检查结构
    if prompt.contains(":") || prompt.contains("-") {
        score += 0.3;
    }

    score.min(1.0)
}

/// 分析上下文相关性
fn analyze_context_relevance(prompt: &str) -> f32 {
    let mut score: f32 = 0.4; // 基础分

    // 上下文关键词
    let context_keywords = ["context", "background", "requirements", "constraints"];
    for keyword in context_keywords {
        if prompt.to_lowercase().contains(keyword) {
            score += 0.15;
        }
    }

    score.min(1.0)
}

/// 检测约束冲突
fn detect_constraint_conflicts(prompt: &str) -> Vec<String> {
    let mut conflicts = Vec::new();

    // 检查长度与详细度的冲突
    if prompt.contains("brief") && prompt.contains("detailed") {
        conflicts.push("长度要求冲突：同时要求简洁和详细".to_string());
    }

    // 检查风格冲突
    if prompt.contains("formal") && prompt.contains("casual") {
        conflicts.push("风格冲突：同时要求正式和随意".to_string());
    }

    conflicts
}

/// 建议优化方案
fn suggest_optimizations(prompt: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    // 长度检查
    if prompt.len() < 20 {
        suggestions.push("建议增加更多上下文信息".to_string());
    }

    // 结构检查
    if !prompt.contains("##") && !prompt.contains("-") {
        suggestions.push("建议添加结构化格式".to_string());
    }

    // 示例检查
    if !prompt.contains("example") && !prompt.contains("```") {
        suggestions.push("建议添加示例说明".to_string());
    }

    suggestions
}
