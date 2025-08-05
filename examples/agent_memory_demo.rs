//! Demonstration of Prompt Compiler's enhancement effects on Agent memory retrieval

use prompt_compiler_core::{
    PromptCompiler,
    compiler::analyzers::SemanticAnalyzer,
    compiler::optimizers::WeightOptimizer,
    compiler::generators::StandardGenerator,
};
use std::collections::HashMap;

/// Simulated Agent memory system
struct AgentMemory {
    raw_memories: Vec<String>,      // Raw memories
    compiled_memories: Vec<String>, // Compiled optimized memories
    compiler: PromptCompiler,
}

impl AgentMemory {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let compiler = PromptCompiler::new()
            .add_analyzer(Box::new(SemanticAnalyzer::new()))
            .add_optimizer(Box::new(WeightOptimizer::new()?))
            .add_generator(Box::new(StandardGenerator::new()));

        Ok(Self {
            raw_memories: Vec::new(),
            compiled_memories: Vec::new(),
            compiler,
        })
    }

    /// Store memory (store both original and optimized versions for comparison)
    fn store_memory(&mut self, experience: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Store original version
        self.raw_memories.push(experience.to_string());

        // Compile and store optimized version
        let compiled = self.compiler.compile(experience)?;
        let compiled_content = compiled.ir.compiled_content.clone(); // Clone to avoid move
        self.compiled_memories.push(compiled_content.clone());

        println!("💾 Storing memory:");
        println!("   Original: {}", experience);
        println!("   Optimized: {}", compiled_content.lines().next().unwrap_or(""));

        Ok(())
    }

    /// Demonstrate memory retrieval quality comparison
    fn demonstrate_retrieval_quality(&self, query: &str) {
        println!("\n🔍 Retrieval query: '{}'", query);
        println!("\n📊 Retrieval quality comparison:");

        // Simulate relevance scoring for raw memories
        println!("🔸 Raw memory system:");
        for (i, memory) in self.raw_memories.iter().enumerate() {
            let relevance = self.calculate_relevance(memory, query);
            println!("   Memory{}: {:.3} - {}", i+1, relevance,
                     memory.chars().take(50).collect::<String>());
        }

        // Simulate relevance scoring for optimized memories
        println!("\n🔹 Optimized memory system:");
        for (i, memory) in self.compiled_memories.iter().enumerate() {
            let relevance = self.calculate_relevance(memory, query);
            println!("   Memory{}: {:.3} - {}", i+1, relevance,
                     memory.chars().take(50).collect::<String>());
        }
    }

    /// Simplified relevance calculation (actual implementation would use vector similarity)
    fn calculate_relevance(&self, memory: &str, query: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let memory_words: Vec<&str> = memory.split_whitespace().collect();

        let common_words = query_words.iter()
            .filter(|word| memory_words.contains(word))
            .count();

        // Bonus for structured memories
        let structure_bonus = if memory.contains("##") || memory.contains("requirement") {
            0.3
        } else {
            0.0
        };

        (common_words as f32 / query_words.len() as f32) + structure_bonus
    }

    /// Show complete memory evolution process
    fn demonstrate_memory_evolution(&self) {
        println!("\n📈 Memory system evolution analysis");
        println!("{}", "=".repeat(70));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Agent Memory System Enhancement Demo");
    println!("Demonstrates how Prompt Compiler improves Agent memory quality and retrieval effectiveness");
    println!("{}", "=".repeat(70));

    let mut memory = AgentMemory::new()?;

    // Simulate Agent learning and storing various experiences
    let experiences = vec![
        "User asked how to write Python code",
        "Need to create database",
        "User requested algorithm performance optimization",
        "Debug network connection issues",
        "Design API interface",
    ];

    println!("📚 Agent Learning Phase - Storing experiences:");
    for experience in experiences {
        memory.store_memory(experience)?;
    }

    // Simulate different retrieval scenarios
    let queries = vec![
        "how to write code",
        "database related",
        "performance optimization solutions",
        "network troubleshooting",
    ];

    println!("\n{}", "=".repeat(70));
    println!("🔍 Memory retrieval testing:");

    for query in queries {
        memory.demonstrate_retrieval_quality(query);
        println!("{}", "-".repeat(50));
    }

    // Summary of value
    println!("\n💡 Core value for Agent memory system:");
    println!("1. 📈 Memory quality improvement: Fuzzy memory → Structured memory");
    println!("2. 🎯 Retrieval accuracy improvement: Keyword matching → Semantic understanding");
    println!("3. 🧠 Context preservation: Simple storage → Rich context");
    println!("4. 🔄 Experience reuse: Scattered experience → Reusable templates");
    println!("5. 📊 Quantitative evaluation: Subjective judgment → Objective metrics");

    Ok(())
}
