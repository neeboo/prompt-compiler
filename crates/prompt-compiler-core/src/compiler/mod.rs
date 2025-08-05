//! Compiler module - Core compilation logic and traits

pub mod analyzers;
pub mod generators;
pub mod optimizers;

use crate::error::Result;
pub use crate::ir::*;
use std::collections::HashMap;

/// Main prompt compiler structure
pub struct PromptCompiler {
    analyzers: Vec<Box<dyn PromptAnalyzer + Send + Sync>>,
    optimizers: Vec<Box<dyn PromptOptimizer + Send + Sync>>,
    generators: Vec<Box<dyn PromptGenerator + Send + Sync>>,
}

/// Prompt analyzer trait
pub trait PromptAnalyzer {
    fn analyze(&self, prompt: &str) -> Result<AnalysisResult>;
}

/// Prompt optimizer trait  
pub trait PromptOptimizer {
    fn optimize(&self, ir: &PromptIR) -> Result<PromptIR>;
}

/// Prompt generator trait
pub trait PromptGenerator {
    fn generate(&self, ir: &PromptIR, target: &ModelTarget) -> Result<String>;
}

/// Analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisResult {
    pub intent_clarity: f32,
    pub context_relevance: f32,
    pub constraint_conflicts: Vec<String>,
    pub suggested_optimizations: Vec<String>,
}

/// Model target configuration
#[derive(Debug, Clone)]
pub struct ModelTarget {
    pub model_name: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub architecture_hints: HashMap<String, String>,
}

impl PromptCompiler {
    /// Create new compiler instance
    pub fn new() -> Self {
        Self {
            analyzers: Vec::new(),
            optimizers: Vec::new(),
            generators: Vec::new(),
        }
    }

    /// Add analyzer
    pub fn add_analyzer(mut self, analyzer: Box<dyn PromptAnalyzer + Send + Sync>) -> Self {
        self.analyzers.push(analyzer);
        self
    }

    /// Add optimizer
    pub fn add_optimizer(mut self, optimizer: Box<dyn PromptOptimizer + Send + Sync>) -> Self {
        self.optimizers.push(optimizer);
        self
    }

    /// Add generator
    pub fn add_generator(mut self, generator: Box<dyn PromptGenerator + Send + Sync>) -> Self {
        self.generators.push(generator);
        self
    }

    /// Compile prompt to intermediate representation
    pub fn compile(&self, prompt: &str) -> Result<CompiledState> {
        // 1. Analysis phase
        let mut ir = self.parse_to_ir(prompt)?;

        for analyzer in &self.analyzers {
            let analysis = analyzer.analyze(prompt)?;
            ir.analysis_metadata
                .insert("analysis".to_string(), serde_json::to_string(&analysis)?);
        }

        // 2. Optimization phase
        for optimizer in &self.optimizers {
            ir = optimizer.optimize(&ir)?;
        }

        // 3. Create compiled state
        let compiled = CompiledState {
            version: env!("CARGO_PKG_VERSION").to_string(),
            ir,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            compilation_metadata: self.generate_metadata(),
        };

        Ok(compiled)
    }

    /// Parse prompt to intermediate representation
    fn parse_to_ir(&self, prompt: &str) -> Result<PromptIR> {
        // Simple parsing logic - would be more complex in actual implementation
        let lines: Vec<&str> = prompt.lines().collect();

        let intent = lines
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unspecified intent".to_string());

        let context = lines
            .iter()
            .skip(1)
            .enumerate()
            .map(|(i, &line)| ContextEntry {
                content: line.to_string(),
                importance: 1.0 / (i + 1) as f32, // Decreasing importance
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                source: "user_input".to_string(),
            })
            .collect();

        Ok(PromptIR {
            intent,
            persona: None,
            context,
            constraints: Vec::new(),
            priority_level: 5,
            token_budget: Some(1000),
            target_models: vec!["gpt-4".to_string()],
            compilation_hints: Vec::new(),
            metadata: HashMap::new(),
            analysis_metadata: HashMap::new(),
        })
    }

    /// Generate compilation metadata
    fn generate_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "compiler_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        metadata.insert(
            "analyzers_count".to_string(),
            self.analyzers.len().to_string(),
        );
        metadata.insert(
            "optimizers_count".to_string(),
            self.optimizers.len().to_string(),
        );
        metadata.insert(
            "generators_count".to_string(),
            self.generators.len().to_string(),
        );
        metadata
    }
}

impl Default for PromptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ModelTarget {
    fn default() -> Self {
        Self {
            model_name: "gpt-4".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            architecture_hints: HashMap::new(),
        }
    }
}
