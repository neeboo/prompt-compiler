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

    /// Execute iterative prompt optimization analysis
    pub fn optimize_iteratively(
        &mut self,
        original_prompt: &str,
        task: &str,
        max_steps: usize,
    ) -> Result<OptimizationHistory, WeightError> {
        let mut steps = Vec::new();
        let mut current_prompt = original_prompt.to_string();
        let mut all_updates = Vec::new();

        println!("🚀 Starting iterative Prompt optimization analysis");
        println!("Original task: {}", task);
        println!("Original Prompt: {}", original_prompt);
        println!("{}", "=".repeat(60));

        for step in 0..max_steps {
            // Analyze current prompt
            let analysis = self.analyzer.analyze_single_prompt(&current_prompt, task)?;

            // Generate improvement suggestions
            let suggestions = self.generate_improvement_suggestions(&current_prompt, &analysis);

            // Record this step
            let step_data = OptimizationStep {
                step_number: step + 1,
                prompt: current_prompt.clone(),
                analysis: analysis.clone(),
                improvement_suggestions: suggestions.clone(),
            };
            steps.push(step_data);

            // Print current step results
            println!("\n📊 Step {} analysis results:", step + 1);
            println!("Current Prompt: {}", current_prompt);
            println!("Effectiveness Score: {:.4}", analysis.effectiveness_score);
            println!("Update Magnitude: {:.4}", analysis.update_magnitude);
            println!("Is Stable: {}", if analysis.is_stable { "✅" } else { "❌" });

            // Collect weight updates for convergence analysis
            let context = self.encoder.encode_prompt(&current_prompt);
            let query = self.encoder.encode_task(task);
            let update = self.analyzer.dynamics.update_step(&context, &query)?;
            all_updates.push(update);

            // If already converged, end early
            if analysis.is_stable && step > 2 {
                println!("✅ Converged to a stable state, ending optimization early");
                break;
            }

            // Apply improvement suggestions to generate the next prompt
            if step < max_steps - 1 {
                current_prompt = self.apply_improvements(&current_prompt, &suggestions);
                println!("📝 Improvement suggestions: {:?}", suggestions);
                println!("🔄 Next Prompt: {}", current_prompt);
            }
        }

        // Calculate final convergence rate
        let convergence_metrics = self.analyzer.dynamics.predict_convergence(&all_updates);
        let final_convergence_rate = convergence_metrics.convergence_rate;

        // Calculate total improvement
        let initial_score = steps.first().map(|s| s.analysis.effectiveness_score).unwrap_or(0.0);
        let final_score = steps.last().map(|s| s.analysis.effectiveness_score).unwrap_or(0.0);
        let total_improvement = ((final_score - initial_score) / initial_score.max(0.001)) * 100.0;

        println!("\n🎯 Optimization Summary:");
        println!("Final Convergence Rate: {:.4}", final_convergence_rate);
        println!("Total Improvement: {:.1}%", total_improvement);
        println!("Gradient History: {:?}", convergence_metrics.gradient_norms);

        Ok(OptimizationHistory {
            original_prompt: original_prompt.to_string(),
            task: task.to_string(),
            steps,
            final_convergence_rate,
            total_improvement,
        })
    }

    /// Generate improvement suggestions
    fn generate_improvement_suggestions(&self, prompt: &str, analysis: &PromptAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Suggestions based on effectiveness score
        if analysis.effectiveness_score < 0.3 {
            suggestions.push("Add a clearer instruction structure".to_string());
            suggestions.push("Use guiding words like 'Please follow these steps'".to_string());
        }

        // Suggestions based on update magnitude
        if analysis.update_magnitude > 1.0 {
            suggestions.push("Simplify the prompt, avoid excessive complexity".to_string());
        }

        // Check for missing key elements in the prompt
        if !prompt.contains("please") && !prompt.contains("could you") {
            suggestions.push("Add polite language to enhance guidance".to_string());
        }

        if !prompt.contains("detailed") && !prompt.contains("specific") {
            suggestions.push("Add 'detailed' or 'specific' requirements for clarity".to_string());
        }

        if !prompt.contains("steps") && !prompt.contains("follow") {
            suggestions.push("Add step-by-step instructions to improve structure".to_string());
        }

        // If no other suggestions, provide general advice
        if suggestions.is_empty() {
            suggestions.push("The current prompt is already quite good".to_string());
        }

        suggestions
    }

    /// Apply improvement suggestions
    fn apply_improvements(&self, original: &str, suggestions: &[String]) -> String {
        let mut improved = original.to_string();

        for suggestion in suggestions {
            if suggestion.contains("follow these steps") && !improved.contains("steps") {
                improved = format!("Please follow these steps for {}: 1) Understand requirements 2) Analyze problem 3) Provide results", improved);
            } else if suggestion.contains("detailed") && !improved.contains("detailed") {
                improved = format!("Please provide a detailed {}", improved);
            } else if suggestion.contains("polite language") && !improved.contains("please") {
                improved = format!("Please {}", improved);
            } else if suggestion.contains("specific") && !improved.contains("specific") {
                improved = improved.replace("analysis", "specific analysis");
            }
        }

        improved
    }
}
