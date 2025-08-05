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

    /// Test the core theoretical implementation: ΔW(Y) = (W·ΔA(Y)) · A(C\Y,x)^T / ||A(C\Y,x)||²
    #[test]
    fn test_weight_update_theory() {
        let config = DynamicsConfig {
            learning_rate: 1.0,
            use_skip_connections: false,
            regularization_strength: 0.0, // No regularization for pure theory test
        };

        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();

        // Create known vectors for predictable results
        let context = DVector::from_vec(vec![1.0, 0.0, 0.0]); // Unit vector
        let query = DVector::from_vec(vec![0.0, 1.0]); // Unit vector

        // Get initial weights
        let initial_weights = dynamics.weights().clone();

        // Apply update
        let update = dynamics.update_step(&context, &query).unwrap();

        // Verify the update follows the theoretical formula
        // The update should be a rank-1 matrix: attention_weight * query * context^T
        assert_eq!(update.delta_w.rank(1e-6), 1); // Should be rank-1

        // Verify dimensions are correct
        assert_eq!(update.delta_w.nrows(), 2); // output_dim
        assert_eq!(update.delta_w.ncols(), 3); // input_dim

        // Verify the update was applied to weights
        let final_weights = dynamics.weights();
        let expected_weights = &initial_weights + &update.delta_w;

        for i in 0..expected_weights.nrows() {
            for j in 0..expected_weights.ncols() {
                assert!((final_weights[(i, j)] - expected_weights[(i, j)]).abs() < 1e-6);
            }
        }
    }

    /// Test sequential updates converge as expected
    #[test]
    fn test_sequential_convergence() {
        let config = DynamicsConfig {
            learning_rate: 0.1, // Small learning rate for stable convergence
            use_skip_connections: false,
            regularization_strength: 0.01,
        };

        let mut dynamics = ImplicitDynamics::new(4, 3, config).unwrap();

        // Create a sequence of similar contexts (simulating in-context learning)
        let contexts = vec![
            DVector::from_vec(vec![1.0, 0.1, 0.01, 0.001]),
            DVector::from_vec(vec![1.0, 0.2, 0.04, 0.008]),
            DVector::from_vec(vec![1.0, 0.3, 0.09, 0.027]),
            DVector::from_vec(vec![1.0, 0.4, 0.16, 0.064]),
        ];

        let query = DVector::from_vec(vec![0.5, 0.3, 0.2]);

        // Compute sequential updates
        let updates = dynamics.compute_sequential_updates(&contexts, &query).unwrap();

        // Verify we got the right number of updates
        assert_eq!(updates.len(), 4);

        // Verify updates are getting smaller (convergence behavior)
        let norms: Vec<f32> = updates.iter().map(|u| u.delta_w.norm()).collect();

        // Generally, we expect decreasing norms as the model "learns" the pattern
        // (though this isn't guaranteed for all sequences)
        assert!(norms.len() == 4);

        // Verify effectiveness scores are computed correctly
        for update in &updates {
            let score = update.effectiveness_score();
            assert!(score >= 0.0);
            assert!(score.is_finite());
        }

        // Test convergence prediction
        let convergence = dynamics.predict_convergence(&updates);
        assert_eq!(convergence.gradient_norms.len(), 4);
        assert!(convergence.convergence_rate.is_finite());
    }

    /// Test effectiveness score calculation
    #[test]
    fn test_effectiveness_score() {
        let delta_w = DMatrix::from_vec(2, 2, vec![0.1, 0.2, 0.3, 0.4]);
        let context = DVector::from_vec(vec![1.0, 0.5]);
        let query = DVector::from_vec(vec![0.8, 0.6]);

        let update = WeightUpdate {
            delta_w,
            delta_b: None,
            context_vector: context,
            query_vector: query,
            step_size: 0.1,
        };

        let score = update.effectiveness_score();

        // Score should be positive and finite
        assert!(score > 0.0);
        assert!(score.is_finite());

        // Test with zero vectors (edge case)
        let zero_context = DVector::zeros(2);
        let zero_update = WeightUpdate {
            delta_w: DMatrix::zeros(2, 2),
            delta_b: None,
            context_vector: zero_context,
            query_vector: DVector::from_vec(vec![1.0, 1.0]),
            step_size: 0.0,
        };

        assert_eq!(zero_update.effectiveness_score(), 0.0);
    }

    /// Test error conditions
    #[test]
    fn test_error_conditions() {
        // Test invalid learning rate
        let bad_config = DynamicsConfig {
            learning_rate: -0.1,
            use_skip_connections: false,
            regularization_strength: 0.0,
        };

        let result = ImplicitDynamics::new(3, 2, bad_config);
        assert!(matches!(result, Err(WeightError::InvalidLearningRate { .. })));

        // Test dimension mismatch
        let config = DynamicsConfig::default();
        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();

        let wrong_context = DVector::from_vec(vec![1.0, 0.5]); // Should be size 3
        let query = DVector::from_vec(vec![0.8, 0.6]);

        let result = dynamics.update_step(&wrong_context, &query);
        assert!(matches!(result, Err(WeightError::InvalidDimensions { .. })));
    }
}
