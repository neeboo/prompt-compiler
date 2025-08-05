//! Standard prompt generator

use crate::compiler::{PromptGenerator, PromptIR, ModelTarget};
use crate::error::Result;

/// Standard Prompt Generator
pub struct StandardGenerator;

impl StandardGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl PromptGenerator for StandardGenerator {
    fn generate(&self, ir: &PromptIR, target: &ModelTarget) -> Result<String> {
        let mut output = ir.compiled_content.clone();

        // Adjust output based on target model
        match target.model_name.as_str() {
            "gpt-4" | "gpt-3.5-turbo" => {
                output = self.optimize_for_gpt(&output, target)?;
            }
            "claude" => {
                output = self.optimize_for_claude(&output)?;
            }
            _ => {
                output = self.add_generic_enhancements(&output)?;
            }
        }
        
        Ok(output)
    }
}

impl StandardGenerator {
    /// Optimize for GPT models
    fn optimize_for_gpt(&self, content: &str, target: &ModelTarget) -> Result<String> {
        let mut optimized = content.to_string();

        // Add token budget hints
        if target.max_tokens < 500 {
            optimized.push_str("\n\nPlease keep the response concise and within a short length.");
        }
        
        // Adjust creativity hints based on temperature
        if target.temperature > 0.8 {
            optimized.push_str("\n\nPlease be creative and provide innovative solutions.");
        } else if target.temperature < 0.3 {
            optimized.push_str("\n\nPlease provide accurate and consistent answers.");
        }

        Ok(optimized)
    }

    /// Optimize for Claude models
    fn optimize_for_claude(&self, content: &str) -> Result<String> {
        let optimized = format!("Human: {}\n\nAssistant: I understand your requirements.", content);
        Ok(optimized)
    }

    /// Generic enhancements
    fn add_generic_enhancements(&self, content: &str) -> Result<String> {
        let enhanced = format!("{}\n\nPlease ensure the answer is accurate, helpful, and easy to understand.", content);
        Ok(enhanced)
    }
}
