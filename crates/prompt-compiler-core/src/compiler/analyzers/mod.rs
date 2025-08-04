//! Analyzer module - Prompt analysis based on in-context learning theory

use crate::compiler::{AnalysisResult, PromptAnalyzer};
use crate::error::Result;

/// Basic semantic analyzer
pub struct SemanticAnalyzer {
    pub name: String,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            name: "semantic_analyzer".to_string(),
        }
    }
}

impl PromptAnalyzer for SemanticAnalyzer {
    fn analyze(&self, prompt: &str) -> Result<AnalysisResult> {
        // Basic semantic analysis
        let intent_clarity = analyze_intent_clarity(prompt);
        let context_relevance = analyze_context_relevance(prompt);
        let constraint_conflicts = detect_constraint_conflicts(prompt);
        let suggested_optimizations = suggest_optimizations(prompt);

        Ok(AnalysisResult {
            intent_clarity,
            context_relevance,
            constraint_conflicts,
            suggested_optimizations,
        })
    }
}

/// Context learning efficiency analyzer
/// Analyzes prompt's in-context learning potential based on paper theory
pub struct ContextLearningAnalyzer {
    pub name: String,
}

impl ContextLearningAnalyzer {
    pub fn new() -> Self {
        Self {
            name: "context_learning_analyzer".to_string(),
        }
    }

    /// Analyze weight update potential of context
    fn analyze_weight_update_potential(&self, prompt: &str) -> f32 {
        let lines: Vec<&str> = prompt.lines().collect();

        // Analyze context diversity and structure
        let diversity_score = analyze_context_diversity(&lines);
        let structure_score = analyze_context_structure(&lines);
        let coherence_score = analyze_context_coherence(&lines);

        (diversity_score + structure_score + coherence_score) / 3.0
    }
}

impl PromptAnalyzer for ContextLearningAnalyzer {
    fn analyze(&self, prompt: &str) -> Result<AnalysisResult> {
        let weight_update_potential = self.analyze_weight_update_potential(prompt);

        let mut optimizations = Vec::new();
        if weight_update_potential < 0.5 {
            optimizations.push("Consider adding more diverse context examples".to_string());
            optimizations.push("Optimize context structure".to_string());
        }

        Ok(AnalysisResult {
            intent_clarity: weight_update_potential,
            context_relevance: weight_update_potential,
            constraint_conflicts: Vec::new(),
            suggested_optimizations: optimizations,
        })
    }
}

// Analysis function implementations

fn analyze_intent_clarity(prompt: &str) -> f32 {
    let has_clear_objective =
        prompt.contains("please") || prompt.contains("help") || prompt.contains("need");
    let has_specific_request =
        prompt.len() > 20 && (prompt.contains("specific") || prompt.contains("detailed"));

    match (has_clear_objective, has_specific_request) {
        (true, true) => 0.9,
        (true, false) => 0.7,
        (false, true) => 0.6,
        (false, false) => 0.3,
    }
}

fn analyze_context_relevance(prompt: &str) -> f32 {
    let lines: Vec<&str> = prompt
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    if lines.len() < 2 {
        return 0.2;
    }

    // Simple relevance analysis - check keyword overlap and topic consistency
    let first_line = lines[0].to_lowercase();
    let keyword_overlap = lines
        .iter()
        .skip(1)
        .map(|line| {
            let line_lower = line.to_lowercase();
            let words: Vec<&str> = first_line.split_whitespace().collect();
            let overlap_count = words
                .iter()
                .filter(|word| line_lower.contains(*word))
                .count();
            overlap_count as f32 / words.len().max(1) as f32
        })
        .sum::<f32>()
        / (lines.len() - 1) as f32;

    keyword_overlap.min(1.0)
}

fn detect_constraint_conflicts(_prompt: &str) -> Vec<String> {
    // Return empty for now - can extend to detect logical conflicts
    Vec::new()
}

fn suggest_optimizations(prompt: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    if prompt.len() < 50 {
        suggestions.push("Consider providing more detailed context information".to_string());
    }

    if !prompt.contains("example") && !prompt.contains("for instance") {
        suggestions.push(
            "Adding specific examples can improve in-context learning effectiveness".to_string(),
        );
    }

    if prompt.lines().count() < 3 {
        suggestions.push("Structured multi-line prompts typically work better".to_string());
    }

    suggestions
}

fn analyze_context_diversity(lines: &[&str]) -> f32 {
    if lines.len() < 2 {
        return 0.0;
    }

    // Calculate vocabulary diversity
    let mut all_words: Vec<String> = Vec::new();
    let mut unique_words = std::collections::HashSet::new();

    for line in lines {
        let words: Vec<String> = line
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        all_words.extend(words.clone());
        unique_words.extend(words);
    }

    if all_words.is_empty() {
        return 0.0;
    }

    unique_words.len() as f32 / all_words.len() as f32
}

fn analyze_context_structure(lines: &[&str]) -> f32 {
    if lines.is_empty() {
        return 0.0;
    }

    // Check structural patterns
    let has_numbering = lines.iter().any(|line| {
        line.trim_start()
            .chars()
            .next()
            .map_or(false, |c| c.is_numeric())
    });

    let has_bullets = lines
        .iter()
        .any(|line| line.trim_start().starts_with('-') || line.trim_start().starts_with('*'));

    let consistent_length = {
        let lengths: Vec<usize> = lines.iter().map(|line| line.len()).collect();
        if lengths.len() < 2 {
            return 0.5;
        }
        let avg_len = lengths.iter().sum::<usize>() as f32 / lengths.len() as f32;
        let variance = lengths
            .iter()
            .map(|&len| (len as f32 - avg_len).powi(2))
            .sum::<f32>()
            / lengths.len() as f32;
        1.0 - (variance.sqrt() / avg_len).min(1.0)
    };

    let structure_score = match (has_numbering, has_bullets) {
        (true, _) => 0.8,
        (false, true) => 0.6,
        (false, false) => 0.3,
    };

    (structure_score + consistent_length) / 2.0
}

fn analyze_context_coherence(lines: &[&str]) -> f32 {
    if lines.len() < 2 {
        return 1.0;
    }

    // Simple coherence analysis - check vocabulary overlap between adjacent lines
    let coherence_scores: Vec<f32> = lines
        .windows(2)
        .map(|window| {
            let words1: std::collections::HashSet<&str> =
                window[0].to_lowercase().split_whitespace().collect();
            let words2: std::collections::HashSet<&str> =
                window[1].to_lowercase().split_whitespace().collect();

            if words1.is_empty() || words2.is_empty() {
                return 0.0;
            }

            let intersection = words1.intersection(&words2).count();
            let union = words1.union(&words2).count();

            intersection as f32 / union as f32
        })
        .collect();

    if coherence_scores.is_empty() {
        return 1.0;
    }

    coherence_scores.iter().sum::<f32>() / coherence_scores.len() as f32
}
