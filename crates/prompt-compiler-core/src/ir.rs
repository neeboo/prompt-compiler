//! Prompt Intermediate Representation (IR) - Based on in-context learning weight update theory

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt Intermediate Representation - Incorporating weight dynamics theory
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptIR {
    // Basic information
    pub intent: String,
    pub persona: Option<String>,
    pub context: Vec<ContextEntry>,
    pub constraints: Vec<String>,
    
    // Enhanced fields
    pub priority_level: u8,           // Priority 1-10
    pub token_budget: Option<u32>,    // Token budget limit
    pub target_models: Vec<String>,   // Target model list
    pub compilation_hints: Vec<String>, // Compilation hints
    pub metadata: HashMap<String, String>, // Extended metadata
    
    // Analysis metadata
    pub analysis_metadata: HashMap<String, String>,
}

/// Context entry - Corresponds to context tokens in the paper
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContextEntry {
    pub content: String,
    pub importance: f32,      // Importance weight (affects weight update magnitude)
    pub timestamp: u64,       // Timestamp
    pub source: String,       // Source identifier
}

/// Compiled state - Contains weight update information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompiledState {
    pub version: String,
    pub ir: PromptIR,
    pub created_at: u64,
    pub compilation_metadata: HashMap<String, String>,
}

/// Compilation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStats {
    pub total_compilations: u64,
    pub avg_compilation_time_ms: f64,
    pub avg_weight_updates_per_prompt: f32,
    pub most_common_targets: Vec<String>,
    pub convergence_rate: f32,
}

/// Compilation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationSummary {
    pub intent: String,
    pub context_count: usize,
    pub effectiveness: f32,
    pub is_converged: bool,
    pub target_models: Vec<String>,
}

impl PromptIR {
    /// Create new PromptIR
    pub fn new(intent: String) -> Self {
        Self {
            intent,
            persona: None,
            context: Vec::new(),
            constraints: Vec::new(),
            priority_level: 5,
            token_budget: None,
            target_models: vec!["gpt-4".to_string()],
            compilation_hints: Vec::new(),
            metadata: HashMap::new(),
            analysis_metadata: HashMap::new(),
        }
    }

    /// Add context entry
    pub fn add_context(&mut self, content: String, importance: f32, source: String) {
        self.context.push(ContextEntry {
            content,
            importance,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs(),
            source,
        });
    }

    /// Calculate total importance score
    pub fn total_importance(&self) -> f32 {
        self.context.iter().map(|entry| entry.importance).sum()
    }

    /// Get high importance context
    pub fn high_importance_context(&self, threshold: f32) -> Vec<&ContextEntry> {
        self.context.iter()
            .filter(|entry| entry.importance >= threshold)
            .collect()
    }

    /// Validate IR integrity
    pub fn validate(&self) -> Result<(), String> {
        if self.intent.is_empty() {
            return Err("Intent cannot be empty".to_string());
        }

        if self.priority_level < 1 || self.priority_level > 10 {
            return Err("Priority level must be between 1-10".to_string());
        }

        if let Some(budget) = self.token_budget {
            if budget == 0 {
                return Err("Token budget must be greater than 0".to_string());
            }
        }

        Ok(())
    }
}

impl CompiledState {
    /// Get compilation summary
    pub fn summary(&self) -> CompilationSummary {
        CompilationSummary {
            intent: self.ir.intent.clone(),
            context_count: self.ir.context.len(),
            effectiveness: 0.0, // Will be computed with weight updates
            is_converged: false, // Will be computed with weight updates
            target_models: self.ir.target_models.clone(),
        }
    }
}
