//! Weight-based optimizer using ICL theory

use crate::compiler::{PromptOptimizer, PromptIR};
use crate::error::Result;
use prompt_compiler_weights::{ImplicitDynamics, DynamicsConfig, create_random_vector};
use nalgebra::DVector;

/// Prompt intent classification
#[derive(Debug, Clone)]
enum PromptIntent {
    Coding,
    Explanation,
    Analysis,
    General,
}

/// Weight dynamics-based optimizer
/// Implements theory from "Learning without training: The implicit dynamics of in-context learning"
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

    /// Optimize prompt structure based on weight dynamics analysis
    fn optimize_with_weight_analysis(&mut self, ir: &PromptIR) -> Result<PromptIR> {
        // 1. Convert prompt to context vector sequence
        let context_vectors = self.prompt_to_vectors(&ir.original_content)?;
        let query_vector = create_random_vector(64); // Simulate user intent

        // 2. Calculate weight update sequence
        let updates = self.dynamics.compute_sequential_updates(&context_vectors, &query_vector)?;

        // 3. Analyze convergence - using simplified convergence calculation
        let convergence = self.calculate_convergence(&updates);

        // 4. Adjust prompt structure based on convergence and content analysis
        let mut optimized_ir = ir.clone();
        
        // Use intelligent optimization instead of hardcoded templates
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

        // 5. Record weight analysis results
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

    /// Convert prompt to vector sequence (using nalgebra DVector)
    fn prompt_to_vectors(&self, prompt: &str) -> Result<Vec<DVector<f32>>> {
        let words: Vec<&str> = prompt.split_whitespace().collect();
        let mut vectors = Vec::new();

        for word in words.iter().take(10) { // Limit context length
            // Simplified word vectorization: based on word length and character features
            let mut vector = create_random_vector(64);

            // Adjust vector based on word features
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

    /// Simplified convergence calculation
    fn calculate_convergence(&self, updates: &[prompt_compiler_weights::WeightUpdate]) -> f32 {
        if updates.is_empty() {
            return 0.0;
        }

        // Calculate the variance of weight updates as a convergence metric
        let norms: Vec<f32> = updates.iter().filter_map(|update| {
            // Safely handle possible None weight updates
            let delta_w_norm = update.delta_w.norm();
            let delta_b_norm = update.delta_b.as_ref()?.norm();
            Some(delta_w_norm + delta_b_norm)
        }).collect();

        if norms.len() < 2 {
            return 0.5; // Default to medium convergence
        }

        let mean = norms.iter().sum::<f32>() / norms.len() as f32;
        let variance = norms.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / norms.len() as f32;

        // Low variance indicates high convergence
        let convergence: f32 = (1.0 - variance.sqrt()).max(0.0).min(1.0);
        convergence
    }

    /// Enhance prompt structure (for low convergence situations)
    fn enhance_prompt_structure(&self, original: &str) -> String {
        format!(r#"## Task Description
{}

## Execution Requirements
- Please provide clear and structured answers
- Include specific implementation steps
- Provide example code if necessary

## Output Format
Please organize your answer according to the following format:

1. **Overview**: Briefly explain the solution
2. **Detailed Implementation**: Specific implementation details
3. **Examples**: Relevant usage examples
4. **Notes**: Important points or limitations

## Quality Standards
- Accuracy: Ensure information is correct
- Completeness: Cover all important aspects
- Clarity: Expression should be concise and clear"#, original.trim())
    }

    /// Intelligent optimization based on content analysis (replaces hardcoded templates)
    fn intelligent_optimize(&self, original: &str, convergence: f32) -> String {
        // Analyze the actual intent and type of the prompt
        let intent = self.analyze_prompt_intent(original);
        let length = original.len();

        match (intent, length, convergence) {
            // Code-related requests
            (PromptIntent::Coding, len, _) if len < 20 => {
                self.enhance_coding_prompt(original)
            }
            // Explanation requests
            (PromptIntent::Explanation, len, _) if len < 30 => {
                self.enhance_explanation_prompt(original)
            }
            // Analysis requests
            (PromptIntent::Analysis, len, _) if len < 25 => {
                self.enhance_analysis_prompt(original)
            }
            // For prompts with sufficient length and structure, only perform light optimization
            (_, len, conv) if len > 50 && conv > 0.6 => {
                self.light_optimize(original)
            }
            // Default: Add basic structure
            _ => {
                self.add_basic_structure(original)
            }
        }
    }

    /// Analyze prompt intent
    fn analyze_prompt_intent(&self, prompt: &str) -> PromptIntent {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("code") || prompt_lower.contains("program") ||
           prompt_lower.contains("algorithm") || prompt_lower.contains("implement") {
            PromptIntent::Coding
        } else if prompt_lower.contains("explain") || prompt_lower.contains("describe") ||
                  prompt_lower.contains("what is") || prompt_lower.contains("how") {
            PromptIntent::Explanation
        } else if prompt_lower.contains("analyze") || prompt_lower.contains("compare") ||
                  prompt_lower.contains("evaluate") || prompt_lower.contains("assess") {
            PromptIntent::Analysis
        } else {
            PromptIntent::General
        }
    }

    /// Optimization for programming requests
    fn enhance_coding_prompt(&self, original: &str) -> String {
        format!("Please help {}, requirements:\n1. Provide complete runnable code\n2. Include necessary comments and explanations\n3. If possible, give usage examples", original.trim())
    }

    /// Optimization for explanation requests
    fn enhance_explanation_prompt(&self, original: &str) -> String {
        format!("Please explain {} in detail, including:\n1. Core concept definitions\n2. Working principle explanations\n3. Practical application scenarios", original.trim())
    }

    /// Optimization for analysis requests
    fn enhance_analysis_prompt(&self, original: &str) -> String {
        format!("Please analyze {} in depth from the following angles:\n1. Key element identification\n2. Pros and cons comparison\n3. Conclusions and recommendations", original.trim())
    }

    /// Light optimization
    fn light_optimize(&self, original: &str) -> String {
        if !original.contains("please") && !original.contains("help") {
            format!("Please {}", original.trim())
        } else {
            format!("{}, please ensure the answer is accurate and detailed.", original.trim())
        }
    }

    /// Add basic structure
    fn add_basic_structure(&self, original: &str) -> String {
        format!("Task: {}\nRequirements: Please provide clear and useful answers.", original.trim())
    }

    /// Generate context information for injection space
    pub fn create_injection_context(&self, ir: &PromptIR, convergence: f32) -> InjectionContext {
        let intent = self.analyze_prompt_intent(&ir.original_content);
        let semantic_vectors = self.prompt_to_vectors(&ir.original_content).unwrap_or_default();
        
        InjectionContext {
            original_query: ir.original_content.clone(),
            optimized_prompt: ir.compiled_content.clone(),
            
            // Semantic space information
            semantic_analysis: SemanticAnalysis {
                intent_classification: intent.clone(),
                complexity_score: ir.original_content.len() as f32 / 100.0,
                context_vectors: semantic_vectors,
            },
            
            // Weight dynamics analysis
            weight_dynamics: WeightDynamicsInfo {
                convergence_rate: convergence,
                optimization_strategy: self.get_optimization_strategy(&intent, convergence),
                confidence_score: self.calculate_confidence(convergence),
            },
            
            // Reasoning guidance information
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
        // Calculate our confidence in the optimization effect based on convergence
        if convergence > 0.8 { 0.9 }
        else if convergence > 0.6 { 0.7 }
        else { 0.5 }
    }
    
    fn extract_focus_areas(&self, intent: &PromptIntent) -> Vec<String> {
        match intent {
            PromptIntent::Coding => vec![
                "Code Completeness".to_string(),
                "Best Practices".to_string(),
                "Readability".to_string(),
                "Example Demonstration".to_string()
            ],
            PromptIntent::Explanation => vec![
                "Concept Clarity".to_string(),
                "Logical Structure".to_string(),
                "Practical Application".to_string()
            ],
            PromptIntent::Analysis => vec![
                "Multi-angle Analysis".to_string(),
                "Critical Thinking".to_string(),
                "Conclusion Derivation".to_string()
            ],
            PromptIntent::General => vec![
                "Accuracy".to_string(),
                "Completeness".to_string()
            ]
        }
    }
    
    fn suggest_response_structure(&self, intent: &PromptIntent) -> ResponseStructure {
        match intent {
            PromptIntent::Coding => ResponseStructure {
                sections: vec![
                    "Code Implementation".to_string(),
                    "Key Comments".to_string(),
                    "Usage Examples".to_string(),
                    "Important Notes".to_string()
                ],
                preferred_format: "code_with_explanation".to_string()
            },
            PromptIntent::Explanation => ResponseStructure {
                sections: vec![
                    "Core Concepts".to_string(),
                    "Working Principles".to_string(),
                    "Application Scenarios".to_string()
                ],
                preferred_format: "structured_explanation".to_string()
            },
            _ => ResponseStructure {
                sections: vec!["Main Content".to_string()],
                preferred_format: "natural".to_string()
            }
        }
    }
    
    fn define_quality_criteria(&self, intent: &PromptIntent) -> Vec<QualityCriterion> {
        match intent {
            PromptIntent::Coding => vec![
                QualityCriterion { name: "Runnability".to_string(), weight: 0.4 },
                QualityCriterion { name: "Code Quality".to_string(), weight: 0.3 },
                QualityCriterion { name: "Documentation Completeness".to_string(), weight: 0.3 },
            ],
            PromptIntent::Explanation => vec![
                QualityCriterion { name: "Concept Accuracy".to_string(), weight: 0.5 },
                QualityCriterion { name: "Logical Clarity".to_string(), weight: 0.3 },
                QualityCriterion { name: "Practicality".to_string(), weight: 0.2 },
            ],
            _ => vec![
                QualityCriterion { name: "Accuracy".to_string(), weight: 0.6 },
                QualityCriterion { name: "Completeness".to_string(), weight: 0.4 },
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

/// Context information for injection space
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
