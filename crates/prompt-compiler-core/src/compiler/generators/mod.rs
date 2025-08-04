//! Generator module - Convert IR to target model prompts

use crate::compiler::{PromptGenerator, PromptIR, ModelTarget};
use crate::error::Result;
use std::collections::HashMap;

/// Standard prompt generator
pub struct StandardPromptGenerator {
    pub name: String,
    pub template_registry: HashMap<String, PromptTemplate>,
}

/// Prompt template
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub name: String,
    pub structure: Vec<TemplateSection>,
    pub model_specific_hints: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum TemplateSection {
    Intent,
    Persona,
    Context,
    Constraints,
    Examples,
    Instructions,
}

impl StandardPromptGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            name: "standard_prompt_generator".to_string(),
            template_registry: HashMap::new(),
        };
        
        generator.register_default_templates();
        generator
    }

    fn register_default_templates(&mut self) {
        // GPT-4 template
        let gpt4_template = PromptTemplate {
            name: "gpt4_optimized".to_string(),
            structure: vec![
                TemplateSection::Persona,
                TemplateSection::Intent,
                TemplateSection::Context,
                TemplateSection::Constraints,
                TemplateSection::Instructions,
            ],
            model_specific_hints: {
                let mut hints = HashMap::new();
                hints.insert("temperature".to_string(), "Use moderate temperature setting".to_string());
                hints.insert("structure".to_string(), "Clear sectioned structure".to_string());
                hints
            },
        };
        
        self.template_registry.insert("gpt-4".to_string(), gpt4_template);
        
        // Claude template
        let claude_template = PromptTemplate {
            name: "claude_optimized".to_string(),
            structure: vec![
                TemplateSection::Intent,
                TemplateSection::Context,
                TemplateSection::Examples,
                TemplateSection::Constraints,
                TemplateSection::Persona,
            ],
            model_specific_hints: {
                let mut hints = HashMap::new();
                hints.insert("reasoning".to_string(), "Encourage step-by-step reasoning".to_string());
                hints.insert("examples".to_string(), "Provide specific examples".to_string());
                hints
            },
        };
        
        self.template_registry.insert("claude".to_string(), claude_template);
    }

    fn select_template(&self, target: &ModelTarget) -> &PromptTemplate {
        self.template_registry
            .get(&target.model_name)
            .or_else(|| self.template_registry.get("gpt-4"))
            .unwrap()
    }

    fn render_section(&self, section: &TemplateSection, ir: &PromptIR) -> String {
        match section {
            TemplateSection::Intent => {
                format!("## Task Objective\n{}\n", ir.intent)
            },
            TemplateSection::Persona => {
                if let Some(persona) = &ir.persona {
                    format!("## Role Setting\n{}\n", persona)
                } else {
                    String::new()
                }
            },
            TemplateSection::Context => {
                if ir.context.is_empty() {
                    return String::new();
                }
                
                let mut context_text = String::from("## Context Information\n");
                for (i, entry) in ir.context.iter().enumerate() {
                    context_text.push_str(&format!(
                        "{}. {} (importance: {:.2})\n",
                        i + 1, entry.content, entry.importance
                    ));
                }
                context_text.push('\n');
                context_text
            },
            TemplateSection::Constraints => {
                if ir.constraints.is_empty() {
                    return String::new();
                }
                
                let mut constraints_text = String::from("## Constraints\n");
                for constraint in &ir.constraints {
                    constraints_text.push_str(&format!("- {}\n", constraint));
                }
                constraints_text.push('\n');
                constraints_text
            },
            TemplateSection::Examples => {
                // Extract examples from context
                let examples: Vec<_> = ir.context.iter()
                    .filter(|entry| entry.content.contains("example") || entry.content.contains("for instance"))
                    .collect();
                
                if examples.is_empty() {
                    return String::new();
                }
                
                let mut examples_text = String::from("## Example References\n");
                for example in examples {
                    examples_text.push_str(&format!("- {}\n", example.content));
                }
                examples_text.push('\n');
                examples_text
            },
            TemplateSection::Instructions => {
                let mut instructions = String::from("## Execution Instructions\n");
                
                // Add token budget hint
                if let Some(budget) = ir.token_budget {
                    instructions.push_str(&format!("- Please keep response within {} tokens\n", budget));
                }
                
                // Add priority hint
                instructions.push_str(&format!("- Task priority: {}/10\n", ir.priority_level));
                
                // Add compilation hints
                for hint in &ir.compilation_hints {
                    instructions.push_str(&format!("- {}\n", hint));
                }
                
                instructions.push('\n');
                instructions
            },
        }
    }
}

impl PromptGenerator for StandardPromptGenerator {
    fn generate(&self, ir: &PromptIR, target: &ModelTarget) -> Result<String> {
        let template = self.select_template(target);
        let mut generated_prompt = String::new();
        
        // Add model-specific prefix
        if let Some(prefix) = template.model_specific_hints.get("prefix") {
            generated_prompt.push_str(prefix);
            generated_prompt.push_str("\n\n");
        }
        
        // Generate sections based on template structure
        for section in &template.structure {
            let section_content = self.render_section(section, ir);
            if !section_content.trim().is_empty() {
                generated_prompt.push_str(&section_content);
            }
        }
        
        // Add model-specific suffix
        if let Some(suffix) = template.model_specific_hints.get("suffix") {
            generated_prompt.push_str(suffix);
        }
        
        // Final cleanup and formatting
        generated_prompt = generated_prompt.trim().to_string();
        
        Ok(generated_prompt)
    }
}

/// Weight update aware generator
/// Generates optimized prompts based on computed weight update information
pub struct WeightAwareGenerator {
    pub name: String,
    pub base_generator: StandardPromptGenerator,
}

impl WeightAwareGenerator {
    pub fn new() -> Self {
        Self {
            name: "weight_aware_generator".to_string(),
            base_generator: StandardPromptGenerator::new(),
        }
    }

    /// Analyze weight updates and generate corresponding hints
    fn generate_weight_update_hints(&self, ir: &PromptIR) -> Vec<String> {
        let mut hints = Vec::new();
        
        // Hints based on context length
        match ir.context.len() {
            0..=2 => hints.push("Minimal context, recommend direct clear instructions".to_string()),
            3..=5 => hints.push("Moderate context length, favorable for weight update convergence".to_string()),
            6..=10 => hints.push("Rich context, may require multi-step weight updates".to_string()),
            _ => hints.push("Excessive context, may affect convergence speed".to_string()),
        }
        
        // Hints based on importance distribution
        let importance_variance = calculate_importance_variance(&ir.context);
        if importance_variance > 0.5 {
            hints.push("Large context importance differences, suitable for gradual processing".to_string());
        } else {
            hints.push("Relatively uniform context importance, can be processed in parallel".to_string());
        }
        
        hints
    }
}

impl PromptGenerator for WeightAwareGenerator {
    fn generate(&self, ir: &PromptIR, target: &ModelTarget) -> Result<String> {
        // Generate weight update related hints
        let weight_hints = self.generate_weight_update_hints(ir);
        
        // Create enhanced IR
        let mut enhanced_ir = ir.clone();
        enhanced_ir.compilation_hints.extend(weight_hints);
        
        // Use base generator to generate prompt
        let mut generated_prompt = self.base_generator.generate(&enhanced_ir, target)?;
        
        // Add weight update related metadata comments
        generated_prompt.push_str("\n<!-- Weight Update Information -->\n");
        generated_prompt.push_str(&format!("<!-- Context entries: {} -->\n", ir.context.len()));
        if let Some(budget) = ir.token_budget {
            generated_prompt.push_str(&format!("<!-- Token budget: {} -->\n", budget));
        }
        
        Ok(generated_prompt)
    }
}

/// Calculate importance variance
fn calculate_importance_variance(context: &[crate::ir::ContextEntry]) -> f32 {
    if context.len() < 2 {
        return 0.0;
    }
    
    let mean: f32 = context.iter().map(|entry| entry.importance).sum::<f32>() / context.len() as f32;
    let variance: f32 = context.iter()
        .map(|entry| (entry.importance - mean).powi(2))
        .sum::<f32>() / context.len() as f32;
    
    variance
}
