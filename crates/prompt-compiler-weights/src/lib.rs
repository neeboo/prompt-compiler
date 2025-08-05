//! Weight dynamics computation based on in-context learning theory
//! 
//! This crate implements the mathematical framework for prompt weight dynamics
//! as described in recent in-context learning research papers.

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeightError {
    #[error("Invalid matrix dimensions: expected {expected}, got {actual}")]
    InvalidDimensions { expected: String, actual: String },
    #[error("Convergence failed after {max_iterations} iterations")]
    ConvergenceFailed { max_iterations: usize },
    #[error("Learning rate {rate} is not positive")]
    InvalidLearningRate { rate: f32 },
}

/// Weight update structure containing incremental changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightUpdate {
    /// Weight matrix increment (rank-1 matrix)
    pub delta_w: DMatrix<f32>,
    /// Bias vector increment
    pub delta_b: Option<DVector<f32>>,
    /// Context vector used for this update
    pub context_vector: DVector<f32>,
    /// Query vector used for this update
    pub query_vector: DVector<f32>,
    /// Step size used for this update
    pub step_size: f32,
}

impl WeightUpdate {
    /// Calculate effectiveness score based on update magnitude and stability
    pub fn effectiveness_score(&self) -> f32 {
        let update_norm = self.delta_w.norm();
        let context_norm = self.context_vector.norm();
        let query_norm = self.query_vector.norm();

        // Effectiveness is proportional to update magnitude but inversely to input magnitude
        // This prevents large inputs from dominating the score
        if context_norm > 0.0 && query_norm > 0.0 {
            update_norm / (context_norm * query_norm).sqrt()
        } else {
            0.0
        }
    }
}

/// Configuration for dynamics computation
#[derive(Debug, Clone)]
pub struct DynamicsConfig {
    /// Learning rate (corresponds to h in the paper)
    pub learning_rate: f32,
    /// Whether to use skip connections
    pub use_skip_connections: bool,
    /// Regularization strength
    pub regularization_strength: f32,
}

/// Convergence metrics for the dynamics
#[derive(Debug, Clone)]
pub struct ConvergenceMetrics {
    /// History of gradient norms
    pub gradient_norms: Vec<f32>,
    /// Rate of convergence
    pub convergence_rate: f32,
    /// Whether the dynamics have converged
    pub is_converged: bool,
}

/// Main structure for implicit weight dynamics computation
/// 
/// Implements the mathematical framework from "What and How does In-Context Learning Learn?"
/// and related papers on transformer weight dynamics.
pub struct ImplicitDynamics {
    /// Current weight matrix
    weights: DMatrix<f32>,
    /// Current bias vector
    bias: Option<DVector<f32>>,
    /// Configuration parameters
    config: DynamicsConfig,
    /// History of updates for convergence analysis
    update_history: VecDeque<WeightUpdate>,
    /// Maximum history size
    max_history: usize,
}

impl ImplicitDynamics {
    /// Create new dynamics instance
    /// 
    /// # Arguments
    /// * `input_dim` - Input dimension
    /// * `output_dim` - Output dimension
    /// * `config` - Configuration parameters
    /// 
    /// # Returns
    /// New ImplicitDynamics instance
    pub fn new(input_dim: usize, output_dim: usize, config: DynamicsConfig) -> Result<Self, WeightError> {
        if config.learning_rate <= 0.0 {
            return Err(WeightError::InvalidLearningRate { 
                rate: config.learning_rate 
            });
        }

        Ok(Self {
            weights: DMatrix::zeros(output_dim, input_dim),
            bias: Some(DVector::zeros(output_dim)),
            config,
            update_history: VecDeque::new(),
            max_history: 1000,
        })
    }

    /// Apply a single weight update step
    /// 
    /// Implements the implicit gradient step:
    /// W_{t+1} = W_t + h * δW
    /// where δW is computed from the attention mechanism
    /// 
    /// # Arguments
    /// * `context` - Context vector (key/value)
    /// * `query` - Query vector
    /// 
    /// # Returns
    /// WeightUpdate structure containing the applied changes
    pub fn update_step(&mut self, context: &DVector<f32>, query: &DVector<f32>) -> Result<WeightUpdate, WeightError> {
        let input_dim = self.weights.ncols();
        let output_dim = self.weights.nrows();

        // Validate dimensions
        if context.len() != input_dim {
            return Err(WeightError::InvalidDimensions {
                expected: format!("{}", input_dim),
                actual: format!("{}", context.len()),
            });
        }
        if query.len() != output_dim {
            return Err(WeightError::InvalidDimensions {
                expected: format!("{}", output_dim),
                actual: format!("{}", query.len()),
            });
        }

        // Compute attention score: α = softmax(q^T W k)
        let attention_logit = query.dot(&(&self.weights * context));
        let attention_weight = attention_logit.exp() / (1.0 + attention_logit.exp()); // sigmoid approximation

        // Compute rank-1 update: δW = h * α * q * k^T
        let delta_w = self.config.learning_rate * attention_weight * query * context.transpose();
        
        // Compute bias update if bias exists
        let delta_b = self.bias.as_ref().map(|_| {
            self.config.learning_rate * attention_weight * query
        });

        // Apply regularization
        let regularized_delta_w = &delta_w - self.config.regularization_strength * &self.weights;

        // Update weights
        self.weights += &regularized_delta_w;
        if let (Some(ref mut bias), Some(ref delta_bias)) = (&mut self.bias, &delta_b) {
            *bias += delta_bias;
        }

        let update = WeightUpdate {
            delta_w: regularized_delta_w,
            delta_b,
            context_vector: context.clone(),
            query_vector: query.clone(),
            step_size: self.config.learning_rate * attention_weight,
        };

        // Store in history
        self.update_history.push_back(update.clone());
        if self.update_history.len() > self.max_history {
            self.update_history.pop_front();
        }

        Ok(update)
    }

    /// Compute multiple update steps until convergence
    /// 
    /// # Arguments
    /// * `contexts` - Sequence of context vectors
    /// * `queries` - Sequence of query vectors
    /// * `max_iterations` - Maximum number of iterations
    /// * `tolerance` - Convergence tolerance
    /// 
    /// # Returns
    /// Vector of all weight updates applied
    pub fn converge(&mut self, 
                   contexts: &[DVector<f32>], 
                   queries: &[DVector<f32>],
                   max_iterations: usize,
                   tolerance: f32) -> Result<Vec<WeightUpdate>, WeightError> {
        if contexts.len() != queries.len() {
            return Err(WeightError::InvalidDimensions {
                expected: format!("contexts.len() = {}", contexts.len()),
                actual: format!("queries.len() = {}", queries.len()),
            });
        }

        let mut updates = Vec::new();
        let mut prev_norm = f32::INFINITY;

        for iteration in 0..max_iterations {
            let context_idx = iteration % contexts.len();
            let query_idx = iteration % queries.len();

            let update = self.update_step(&contexts[context_idx], &queries[query_idx])?;
            let current_norm = update.delta_w.norm();
            
            updates.push(update);

            // Check convergence
            if (prev_norm - current_norm).abs() < tolerance {
                return Ok(updates);
            }
            prev_norm = current_norm;
        }

        Err(WeightError::ConvergenceFailed { max_iterations })
    }

    /// Compute sequential weight updates for a series of context-query pairs
    ///
    /// # Arguments
    /// * `contexts` - Sequence of context vectors
    /// * `query` - Single query vector to use for all updates
    ///
    /// # Returns
    /// Vector of weight updates computed sequentially
    pub fn compute_sequential_updates(&mut self, contexts: &[DVector<f32>], query: &DVector<f32>) -> Result<Vec<WeightUpdate>, WeightError> {
        let mut updates = Vec::new();

        for context in contexts {
            let update = self.update_step(context, query)?;
            updates.push(update);
        }

        Ok(updates)
    }

    /// Predict convergence based on a sequence of updates
    ///
    /// # Arguments
    /// * `updates` - Sequence of weight updates to analyze
    ///
    /// # Returns
    /// Convergence metrics prediction
    pub fn predict_convergence(&self, updates: &[WeightUpdate]) -> ConvergenceMetrics {
        let gradient_norms: Vec<f32> = updates
            .iter()
            .map(|update| update.delta_w.norm())
            .collect();

        let convergence_rate = if gradient_norms.len() > 3 {
            let recent_avg = gradient_norms.iter().rev().take(2).sum::<f32>() / 2.0;
            let early_avg = gradient_norms.iter().take(2).sum::<f32>() / 2.0;
            if early_avg > 0.0 {
                (early_avg - recent_avg) / early_avg
            } else {
                0.0
            }
        } else {
            0.0
        };

        let is_converged = gradient_norms.last().unwrap_or(&1.0) < &0.01;

        ConvergenceMetrics {
            gradient_norms,
            convergence_rate,
            is_converged,
        }
    }

    /// Get current weights matrix
    pub fn weights(&self) -> &DMatrix<f32> {
        &self.weights
    }

    /// Get current bias vector
    pub fn bias(&self) -> Option<&DVector<f32>> {
        self.bias.as_ref()
    }

    /// Get configuration
    pub fn config(&self) -> &DynamicsConfig {
        &self.config
    }

    /// Analyze convergence properties
    pub fn analyze_convergence(&self) -> ConvergenceMetrics {
        let gradient_norms: Vec<f32> = self.update_history
            .iter()
            .map(|update| update.delta_w.norm())
            .collect();

        let convergence_rate = if gradient_norms.len() > 5 {
            let recent_avg = gradient_norms.iter().rev().take(3).sum::<f32>() / 3.0;
            let early_avg = gradient_norms.iter().take(3).sum::<f32>() / 3.0;
            (early_avg - recent_avg) / early_avg
        } else {
            0.0
        };

        let is_converged = gradient_norms.last().unwrap_or(&1.0) < &0.01;

        ConvergenceMetrics {
            gradient_norms,
            convergence_rate,
            is_converged,
        }
    }
}

impl Default for DynamicsConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1.0, // Corresponds to h = 1/||A(x)||² in paper
            use_skip_connections: true,
            regularization_strength: 0.01,
        }
    }
}

/// Utility function: Create random weight matrix for testing
pub fn create_random_weights(rows: usize, cols: usize) -> DMatrix<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    DMatrix::from_fn(rows, cols, |_, _| rng.gen::<f32>())
}

/// Utility function: Create random vector for testing
pub fn create_random_vector(size: usize) -> DVector<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    DVector::from_fn(size, |_, _| rng.gen::<f32>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamics_creation() {
        let config = DynamicsConfig::default();
        let dynamics = ImplicitDynamics::new(10, 5, config).unwrap();
        
        assert_eq!(dynamics.weights().nrows(), 5);
        assert_eq!(dynamics.weights().ncols(), 10);
        assert!(dynamics.bias().is_some());
    }

    #[test]
    fn test_single_update() {
        let config = DynamicsConfig::default();
        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();
        
        let context = DVector::from_vec(vec![1.0, 0.5, -0.3]);
        let query = DVector::from_vec(vec![0.8, -0.2]);
        
        let update = dynamics.update_step(&context, &query).unwrap();
        
        assert_eq!(update.delta_w.nrows(), 2);
        assert_eq!(update.delta_w.ncols(), 3);
        assert_eq!(update.context_vector, context);
        assert_eq!(update.query_vector, query);
    }

    #[test]
    fn test_convergence_analysis() {
        let config = DynamicsConfig::default();
        let mut dynamics = ImplicitDynamics::new(2, 2, config).unwrap();
        
        // Apply several updates
        for i in 0..10 {
            let context = DVector::from_vec(vec![1.0, i as f32 * 0.1]);
            let query = DVector::from_vec(vec![0.5, -i as f32 * 0.05]);
            let _ = dynamics.update_step(&context, &query).unwrap();
        }
        
        let metrics = dynamics.analyze_convergence();
        assert_eq!(metrics.gradient_norms.len(), 10);
    }
}
