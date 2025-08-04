//! Optimizer module - Prompt optimization based on weight update theory

use crate::compiler::{PromptOptimizer, PromptIR, ContextEntry};
use crate::error::Result;

/// Weight update efficiency optimizer
/// Optimizes prompts for better weight update effects based on paper theory
pub struct WeightUpdateOptimizer {
    pub name: String,
    pub min_context_diversity: f32,
    pub max_context_length: usize,
}

impl WeightUpdateOptimizer {
    pub fn new() -> Self {
        Self {
            name: "weight_update_optimizer".to_string(),
            min_context_diversity: 0.3,
            max_context_length: 10,
        }
    }

    /// Optimize context sequence for improved weight update effects
    fn optimize_context_sequence(&self, context: &mut Vec<ContextEntry>) {
        // Sort by importance
        context.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        
        // Limit context length
        if context.len() > self.max_context_length {
            context.truncate(self.max_context_length);
        }
        
        // Adjust importance weights to align with convergence theory from paper
        for (i, entry) in context.iter_mut().enumerate() {
            // Decreasing weights, aligning with gradient descent convergence patterns
            entry.importance *= 1.0 / (i + 1) as f32;
        }
    }

    /// Enhance context diversity
    fn enhance_context_diversity(&self, context: &mut Vec<ContextEntry>) {
        // Check context diversity
        let diversity = self.calculate_diversity(context);
        
        if diversity < self.min_context_diversity {
            // Add diversity enhancement to context
            for entry in context.iter_mut() {
                if entry.content.len() < 50 {
                    entry.content = format!("{} (diversity enhancement)", entry.content);
                }
            }
        }
    }

    fn calculate_diversity(&self, context: &[ContextEntry]) -> f32 {
        if context.len() < 2 {
            return 0.0;
        }

        let mut all_words = std::collections::HashSet::new();
        let mut total_words = 0;

        for entry in context {
            let words: Vec<&str> = entry.content.split_whitespace().collect();
            total_words += words.len();
            all_words.extend(words);
        }

        if total_words == 0 {
            return 0.0;
        }

        all_words.len() as f32 / total_words as f32
    }
}

impl PromptOptimizer for WeightUpdateOptimizer {
    fn optimize(&self, ir: &PromptIR) -> Result<PromptIR> {
        let mut optimized_ir = ir.clone();
        
        // Optimize context sequence
        self.optimize_context_sequence(&mut optimized_ir.context);
        
        // Enhance context diversity
        self.enhance_context_diversity(&mut optimized_ir.context);
        
        // Add optimization hint
        optimized_ir.compilation_hints.push(
            "Weight update optimization applied".to_string()
        );
        
        Ok(optimized_ir)
    }
}

/// Token budget optimizer
pub struct TokenBudgetOptimizer {
    pub name: String,
    pub target_efficiency: f32,
}

impl TokenBudgetOptimizer {
    pub fn new() -> Self {
        Self {
            name: "token_budget_optimizer".to_string(),
            target_efficiency: 0.8,
        }
    }

    /// Estimate token count for text
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Simplified token estimation - actual implementation would use tokenizer
        (text.len() as f32 / 4.0) as u32
    }

    /// Optimize text to fit token budget
    fn optimize_text_for_budget(&self, text: &str, budget: u32) -> String {
        let estimated_tokens = self.estimate_tokens(text);
        
        if estimated_tokens <= budget {
            return text.to_string();
        }
        
        // Simple truncation strategy
        let ratio = budget as f32 / estimated_tokens as f32;
        let target_length = (text.len() as f32 * ratio) as usize;
        
        if target_length < text.len() {
            format!("{}...", &text[..target_length.saturating_sub(3)])
        } else {
            text.to_string()
        }
    }
}

impl PromptOptimizer for TokenBudgetOptimizer {
    fn optimize(&self, ir: &PromptIR) -> Result<PromptIR> {
        let mut optimized_ir = ir.clone();
        
        if let Some(budget) = ir.token_budget {
            // Estimate current token usage
            let current_tokens = self.estimate_tokens(&ir.intent) +
                ir.context.iter()
                    .map(|entry| self.estimate_tokens(&entry.content))
                    .sum::<u32>();
            
            if current_tokens > budget {
                // Optimize intent text
                optimized_ir.intent = self.optimize_text_for_budget(
                    &ir.intent,
                    (budget as f32 * 0.3) as u32
                );
                
                // Optimize context entries
                let context_budget = (budget as f32 * 0.7) as u32;
                let per_context_budget = if ir.context.is_empty() {
                    0
                } else {
                    context_budget / ir.context.len() as u32
                };
                
                for entry in &mut optimized_ir.context {
                    entry.content = self.optimize_text_for_budget(
                        &entry.content,
                        per_context_budget
                    );
                }
                
                optimized_ir.compilation_hints.push(
                    format!("Token usage optimized: {} -> {}", current_tokens, budget)
                );
            }
        }
        
        Ok(optimized_ir)
    }
}

/// Priority balance optimizer
pub struct PriorityBalanceOptimizer {
    pub name: String,
}

impl PriorityBalanceOptimizer {
    pub fn new() -> Self {
        Self {
            name: "priority_balance_optimizer".to_string(),
        }
    }

    /// Adjust context importance based on priority
    fn balance_context_priorities(&self, ir: &mut PromptIR) {
        let priority_factor = ir.priority_level as f32 / 10.0;
        
        for entry in &mut ir.context {
            // High priority prompts give context higher importance
            entry.importance *= priority_factor;
        }
        
        // Adjust token budget based on priority
        if let Some(budget) = ir.token_budget {
            if ir.priority_level >= 8 {
                // High priority allows more tokens
                ir.token_budget = Some((budget as f32 * 1.2) as u32);
            } else if ir.priority_level <= 3 {
                // Low priority restricts token usage
                ir.token_budget = Some((budget as f32 * 0.8) as u32);
            }
        }
    }
}

impl PromptOptimizer for PriorityBalanceOptimizer {
    fn optimize(&self, ir: &PromptIR) -> Result<PromptIR> {
        let mut optimized_ir = ir.clone();
        
        self.balance_context_priorities(&mut optimized_ir);
        
        optimized_ir.compilation_hints.push(
            format!("Resource allocation adjusted for priority({})", ir.priority_level)
        );
        
        Ok(optimized_ir)
    }
}
