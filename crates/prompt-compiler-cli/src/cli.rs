//! CLI module - Command line interface

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json;
use prompt_compiler_core::compiler::analyzers::{ContextLearningAnalyzer, SemanticAnalyzer};
use prompt_compiler_core::compiler::generators::WeightAwareGenerator;
use prompt_compiler_core::compiler::optimizers::{
    PriorityBalanceOptimizer, TokenBudgetOptimizer, WeightUpdateOptimizer,
};
use prompt_compiler_core::{ModelTarget, PromptCompiler, PromptIR};
use prompt_compiler_core::{PromptAnalyzer, PromptGenerator, PromptOptimizer};
use prompt_compiler_weights::{
    create_random_vector, DynamicsConfig, ImplicitDynamics,
};

#[derive(Parser)]
#[command(name = "pc")]
#[command(about = "Prompt Compiler - AI prompt compiler based on in-context learning theory")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile prompt
    Compile {
        /// Input prompt text
        #[arg(short, long)]
        prompt: String,

        /// Target model
        #[arg(short, long, default_value = "gpt-4")]
        model: String,

        /// Token budget
        #[arg(short, long)]
        budget: Option<u32>,

        /// Priority (1-10)
        #[arg(short = 'p', long, default_value = "5")]
        priority: u8,

        /// Enable weight update analysis
        #[arg(long)]
        enable_weight_analysis: bool,

        /// Output format (json|text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Analyze prompt
    Analyze {
        /// Input prompt text
        #[arg(short, long)]
        prompt: String,

        /// Analyzer type
        #[arg(short, long, default_value = "semantic")]
        analyzer: String,
    },

    /// Optimize prompt
    Optimize {
        /// Input prompt text
        #[arg(short, long)]
        prompt: String,

        /// Optimizer type
        #[arg(short = 'O', long, default_value = "all")]
        optimizer: String,

        /// Token budget
        #[arg(short, long)]
        budget: Option<u32>,
    },

    /// Weight update demo
    WeightDemo {
        /// Number of context entries
        #[arg(short, long, default_value = "5")]
        context_count: usize,

        /// Show verbose information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show version information
    Version,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        match self.command {
            Commands::Compile {
                prompt,
                model,
                budget,
                priority,
                enable_weight_analysis,
                format,
            } => {
                Self::handle_compile(
                    prompt,
                    model,
                    budget,
                    priority,
                    enable_weight_analysis,
                    format,
                )
                .await
            }
            Commands::Analyze { prompt, analyzer } => Self::handle_analyze(prompt, analyzer).await,
            Commands::Optimize {
                prompt,
                optimizer,
                budget,
            } => Self::handle_optimize(prompt, optimizer, budget).await,
            Commands::WeightDemo {
                context_count,
                verbose,
            } => Self::handle_weight_demo(context_count, verbose).await,
            Commands::Version => Self::handle_version().await,
        }
    }

    async fn handle_compile(
        prompt: String,
        model: String,
        budget: Option<u32>,
        priority: u8,
        enable_weight_analysis: bool,
        format: String,
    ) -> Result<()> {
        println!("{}", "🚀 Starting prompt compilation...".cyan().bold());

        // Create compiler
        let compiler = PromptCompiler::new()
            .add_analyzer(Box::new(SemanticAnalyzer::new()))
            .add_analyzer(Box::new(ContextLearningAnalyzer::new()))
            .add_optimizer(Box::new(WeightUpdateOptimizer::new()))
            .add_optimizer(Box::new(TokenBudgetOptimizer::new()))
            .add_optimizer(Box::new(PriorityBalanceOptimizer::new()))
            .add_generator(Box::new(WeightAwareGenerator::new()));

        if enable_weight_analysis {
            println!("{}", "📊 Weight update analysis enabled".yellow());
        }

        // Compile
        let mut compiled = compiler.compile(&prompt)?;

        // Set user-specified parameters
        compiled.ir.token_budget = budget;
        compiled.ir.priority_level = priority;
        compiled.ir.target_models = vec![model.clone()];

        // Generate final prompt
        let target = ModelTarget {
            model_name: model.clone(),
            max_tokens: budget.unwrap_or(1000),
            temperature: 0.7,
            architecture_hints: std::collections::HashMap::new(),
        };

        let generator = WeightAwareGenerator::new();
        let final_prompt = generator.generate(&compiled.ir, &target)?;

        // Output results
        match format.as_str() {
            "json" => {
                let output = serde_json::json!({
                    "original_prompt": prompt,
                    "compiled_prompt": final_prompt,
                    "metadata": {
                        "target_model": model,
                        "token_budget": budget,
                        "priority": priority,
                    },
                    "compilation_hints": compiled.ir.compilation_hints,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            _ => {
                println!("\n{}", "📝 Compilation Result:".green().bold());
                println!("{}", "=".repeat(50).green());
                println!("{}", final_prompt);
                println!("{}", "=".repeat(50).green());

                println!("\n{}", "📊 Compilation Statistics:".blue().bold());
                println!("Target model: {}", model.cyan());
                if let Some(b) = budget {
                    println!("Token budget: {}", b.to_string().cyan());
                }
                println!("Priority: {}/10", priority.to_string().cyan());

                if !compiled.ir.compilation_hints.is_empty() {
                    println!("\n{}", "💡 Compilation Hints:".magenta().bold());
                    for hint in &compiled.ir.compilation_hints {
                        println!("• {}", hint.bright_black());
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_analyze(prompt: String, analyzer_type: String) -> Result<()> {
        println!("{}", "🔍 Starting prompt analysis...".cyan().bold());

        let result = match analyzer_type.as_str() {
            "semantic" => {
                let analyzer = SemanticAnalyzer::new();
                analyzer.analyze(&prompt)?
            }
            "context" => {
                let analyzer = ContextLearningAnalyzer::new();
                analyzer.analyze(&prompt)?
            }
            _ => {
                println!("{}", "❌ Unsupported analyzer type".red());
                return Ok(());
            }
        };

        println!("\n{}", "📊 Analysis Results:".green().bold());
        println!("Intent clarity: {:.2}", result.intent_clarity);
        println!("Context relevance: {:.2}", result.context_relevance);

        if !result.constraint_conflicts.is_empty() {
            println!("\n{}", "⚠️  Constraint conflicts:".yellow().bold());
            for conflict in result.constraint_conflicts {
                println!("• {}", conflict.red());
            }
        }

        if !result.suggested_optimizations.is_empty() {
            println!("\n{}", "💡 Optimization suggestions:".blue().bold());
            for suggestion in result.suggested_optimizations {
                println!("• {}", suggestion.bright_blue());
            }
        }

        Ok(())
    }

    async fn handle_optimize(
        prompt: String,
        optimizer_type: String,
        budget: Option<u32>,
    ) -> Result<()> {
        println!("{}", "⚡ Starting prompt optimization...".cyan().bold());

        // Create initial IR
        let mut ir = PromptIR::new(prompt.clone());
        ir.token_budget = budget;

        // Parse simple context
        let lines: Vec<&str> = prompt.lines().collect();
        for (i, line) in lines.iter().enumerate().skip(1) {
            ir.add_context(
                line.to_string(),
                1.0 / (i as f32 + 1.0),
                "user_input".to_string(),
            );
        }

        // Apply optimizers
        let optimized_ir = match optimizer_type.as_str() {
            "weight" => {
                let optimizer = WeightUpdateOptimizer::new();
                optimizer.optimize(&ir)?
            }
            "budget" => {
                let optimizer = TokenBudgetOptimizer::new();
                optimizer.optimize(&ir)?
            }
            "priority" => {
                let optimizer = PriorityBalanceOptimizer::new();
                optimizer.optimize(&ir)?
            }
            "all" => {
                let mut current_ir = ir;

                let weight_optimizer = WeightUpdateOptimizer::new();
                current_ir = weight_optimizer.optimize(&current_ir)?;

                let budget_optimizer = TokenBudgetOptimizer::new();
                current_ir = budget_optimizer.optimize(&current_ir)?;

                let priority_optimizer = PriorityBalanceOptimizer::new();
                priority_optimizer.optimize(&current_ir)?
            }
            _ => {
                println!("{}", "❌ Unsupported optimizer type".red());
                return Ok(());
            }
        };

        println!("\n{}", "✨ Optimization Results:".green().bold());
        println!("Original intent: {}", prompt.bright_black());
        println!("Optimized intent: {}", optimized_ir.intent.green());

        if !optimized_ir.compilation_hints.is_empty() {
            println!("\n{}", "🔧 Applied optimizations:".blue().bold());
            for hint in optimized_ir.compilation_hints {
                println!("• {}", hint.cyan());
            }
        }

        Ok(())
    }

    async fn handle_weight_demo(context_count: usize, verbose: bool) -> Result<()> {
        println!("{}", "🧠 Weight Update Dynamics Demo".magenta().bold());
        println!(
            "Based on: Learning without training: The implicit dynamics of in-context learning\n"
        );

        // Create random weights and context
        let config = DynamicsConfig::default();
        let mut dynamics = ImplicitDynamics::new(8, 8, config)?;

        let context_tokens: Vec<_> = (0..context_count)
            .map(|_| create_random_vector(8))
            .collect();
        let query = create_random_vector(8);

        println!("🔢 Computing sequential weight updates...");
        let updates = dynamics.compute_sequential_updates(&context_tokens, &query)?;

        println!("\n📊 Weight Update Sequence:");
        for (i, update) in updates.iter().enumerate() {
            println!(
                "Step {}: Update norm = {:.4}, Effectiveness = {:.4}",
                i + 1,
                update.delta_w.norm(),
                update.effectiveness_score()
            );

            if verbose {
                println!(
                    "   Context vector norm: {:.4}",
                    update.context_vector.norm()
                );
                println!("   Query vector norm: {:.4}", update.query_vector.norm());
            }
        }

        // Analyze convergence
        let convergence = dynamics.predict_convergence(&updates);

        println!("\n🎯 Convergence Analysis:");
        println!("Convergence rate: {:.4}", convergence.convergence_rate);
        println!(
            "Converged: {}",
            if convergence.is_converged {
                "Yes".green()
            } else {
                "No".yellow()
            }
        );

        if verbose && !convergence.gradient_norms.is_empty() {
            println!("\n📈 Gradient norm sequence:");
            for (i, norm) in convergence.gradient_norms.iter().enumerate() {
                println!("  Step {}-{}: {:.6}", i + 1, i + 2, norm);
            }
        }

        println!(
            "\n{}",
            "💡 This demonstrates the implicit weight update mechanism described in the paper"
                .blue()
                .italic()
        );

        Ok(())
    }

    async fn handle_version() -> Result<()> {
        println!(
            "{} {}",
            "Prompt Compiler".cyan().bold(),
            env!("CARGO_PKG_VERSION").green()
        );
        println!(
            "Based on: Learning without training: The implicit dynamics of in-context learning"
        );
        println!("Authors: Benoit Dherin, Michael Munn, Hanna Mazzawi, et al.");
        println!("\n🧠 {}", "Theoretical Foundation:".blue().bold());
        println!("• In-context learning ≡ Implicit weight updates");
        println!("• T_W(C,x) = T_{{W+ΔW(C)}}(x)");
        println!("• ΔW(C) is rank-1 matrix update");

        Ok(())
    }
}
