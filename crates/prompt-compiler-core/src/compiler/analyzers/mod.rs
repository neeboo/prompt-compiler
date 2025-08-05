//! Semantic analyzer for prompt content

use crate::compiler::{PromptAnalyzer, AnalysisResult};
use crate::error::Result;

/// Semantic Analyzer - Analyzes prompt semantic quality
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

/// Analyze intent clarity
fn analyze_intent_clarity(prompt: &str) -> f32 {
    let mut score: f32 = 0.0;

    // Check action words
    let action_words = ["write", "create", "generate", "analyze", "explain", "solve"];
    let has_action = action_words.iter().any(|&word| prompt.to_lowercase().contains(word));
    if has_action {
        score += 0.3;
    }

    // Check specificity
    if prompt.len() > 20 {
        score += 0.2;
    }
    if prompt.contains("example") || prompt.contains("format") {
        score += 0.2;
    }

    // Check structure
    if prompt.contains(":") || prompt.contains("-") {
        score += 0.3;
    }

    score.min(1.0)
}

/// Analyze context relevance
fn analyze_context_relevance(prompt: &str) -> f32 {
    let mut score: f32 = 0.4; // Base score

    // Context keywords
    let context_keywords = ["context", "background", "requirements", "constraints"];
    for keyword in context_keywords {
        if prompt.to_lowercase().contains(keyword) {
            score += 0.15;
        }
    }

    score.min(1.0)
}

/// Detect constraint conflicts
fn detect_constraint_conflicts(prompt: &str) -> Vec<String> {
    let mut conflicts = Vec::new();

    // Check length and detail conflicts
    if prompt.contains("brief") && prompt.contains("detailed") {
        conflicts.push("Length requirement conflict: requires both brevity and detail".to_string());
    }

    // Check style conflicts
    if prompt.contains("formal") && prompt.contains("casual") {
        conflicts.push("Style conflict: requires both formal and casual elements".to_string());
    }

    conflicts
}

/// Suggest optimizations
fn suggest_optimizations(prompt: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Length check
    if prompt.len() < 20 {
        suggestions.push("Consider adding more contextual information".to_string());
    }

    // Structure check
    if !prompt.contains("##") && !prompt.contains("-") {
        suggestions.push("Consider adding a structured format".to_string());
    }

    // Example check
    if !prompt.contains("example") && !prompt.contains("```") {
        suggestions.push("Consider adding an example for clarification".to_string());
    }

    suggestions
}
