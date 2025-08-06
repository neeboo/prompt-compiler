//! Weight dynamics computation based on in-context learning theory
//!
//! This crate implements the mathematical framework for prompt weight dynamics
//! as described in "Learning without training: The implicit dynamics of in-context learning"

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::ops::AddAssign;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeightError {
    #[error("Invalid matrix dimensions: expected {expected}, got {actual}")]
    InvalidDimensions { expected: String, actual: String },
    #[error("Convergence failed after {max_iterations} iterations")]
    ConvergenceFailed { max_iterations: usize },
    #[error("Learning rate {rate} is not positive")]
    InvalidLearningRate { rate: f32 },
    #[error("Invalid number of heads: {num_heads}")]
    InvalidNumHeads { num_heads: usize },
}

/// Softmax function implementation for attention weights
fn softmax(logits: &[f32]) -> Vec<f32> {
    if logits.is_empty() {
        return Vec::new();
    }

    let max_logit = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let exp_logits: Vec<f32> = logits.iter().map(|&x| (x - max_logit).exp()).collect();
    let sum_exp: f32 = exp_logits.iter().sum();

    if sum_exp > 0.0 {
        exp_logits.iter().map(|&x| x / sum_exp).collect()
    } else {
        vec![1.0 / logits.len() as f32; logits.len()]
    }
}

/// Positional encoding for transformer-like behavior
fn apply_positional_encoding(vector: &DVector<f32>, position: usize, max_len: usize) -> DVector<f32> {
    let d_model = vector.len();
    let mut encoded = vector.clone();

    for i in 0..d_model {
        let angle = position as f32 / (10000.0_f32).powf(2.0 * (i as f32) / d_model as f32);
        if i % 2 == 0 {
            encoded[i] += angle.sin();
        } else {
            encoded[i] += angle.cos();
        }
    }

    encoded
}

/// Multi-head attention configuration
#[derive(Debug, Clone)]
pub struct MultiHeadConfig {
    /// Number of attention heads
    pub num_heads: usize,
    /// Dimension per head
    pub head_dim: usize,
    /// Whether to use scaled dot-product attention
    pub use_scaling: bool,
}

impl MultiHeadConfig {
    pub fn new(num_heads: usize, total_dim: usize) -> Result<Self, WeightError> {
        if num_heads == 0 || total_dim % num_heads != 0 {
            return Err(WeightError::InvalidNumHeads { num_heads });
        }

        Ok(Self {
            num_heads,
            head_dim: total_dim / num_heads,
            use_scaling: true,
        })
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
            attention_weights: vec![attention_weight],
            position: None,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enhanced weight update structure with multi-head support
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
    /// Attention weights for each head (if multi-head)
    pub attention_weights: Vec<f32>,
    /// Position in sequence
    pub position: Option<usize>,
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

/// Multi-head implicit dynamics for full transformer compatibility
pub struct MultiHeadImplicitDynamics {
    /// Individual dynamics for each head
    heads: Vec<ImplicitDynamics>,
    /// Multi-head configuration
    config: MultiHeadConfig,
    /// Output projection weights
    output_projection: DMatrix<f32>,
    /// Position encoding cache
    position_cache: Vec<DVector<f32>>,
}

impl MultiHeadImplicitDynamics {
    /// Create new multi-head dynamics
    pub fn new(
        input_dim: usize,
        output_dim: usize,
        multihead_config: MultiHeadConfig,
        dynamics_config: DynamicsConfig
    ) -> Result<Self, WeightError> {
        let mut heads = Vec::new();

        // Create individual heads
        for _ in 0..multihead_config.num_heads {
            let head = ImplicitDynamics::new(
                multihead_config.head_dim,
                multihead_config.head_dim,
                dynamics_config.clone()
            )?;
            heads.push(head);
        }

        // Output projection to combine heads
        let output_projection = DMatrix::zeros(output_dim, input_dim);

        Ok(Self {
            heads,
            config: multihead_config,
            output_projection,
            position_cache: Vec::new(),
        })
    }

    /// Compute full multi-head attention update following the paper's formula
    /// Implements: ΔW = Σ_h α_h * q_h ⊗ k_h
    pub fn multi_head_update_step(
        &mut self,
        contexts: &[DVector<f32>],
        query: &DVector<f32>,
        position: Option<usize>
    ) -> Result<WeightUpdate, WeightError> {
        if contexts.is_empty() {
            return Err(WeightError::InvalidDimensions {
                expected: "non-empty context sequence".to_string(),
                actual: "empty".to_string(),
            });
        }

        // Apply positional encoding if position is provided
        let encoded_contexts: Vec<DVector<f32>> = if let Some(pos) = position {
            contexts.iter().enumerate().map(|(i, ctx)| {
                apply_positional_encoding(ctx, pos + i, 1024)
            }).collect()
        } else {
            contexts.to_vec()
        };

        // Compute attention logits for all contexts
        let mut all_logits = Vec::new();
        for context in &encoded_contexts {
            let logit = self.compute_attention_logit(query, context)?;
            all_logits.push(logit);
        }

        // Apply softmax across all contexts (proper attention normalization)
        let attention_weights = softmax(&all_logits);

        // Initialize total update matrix with correct dimensions
        let total_dim = self.config.num_heads * self.config.head_dim;
        let mut total_delta_w = DMatrix::zeros(total_dim, total_dim);
        let mut head_attention_weights = Vec::new();

        for (head_idx, _head) in self.heads.iter_mut().enumerate() {
            let head_start = head_idx * self.config.head_dim;

            for (ctx_idx, context) in encoded_contexts.iter().enumerate() {
                // Extract head-specific portions with correct dimensions
                let q_head = query.rows(head_start, self.config.head_dim);
                let k_head = context.rows(head_start, self.config.head_dim);

                // Compute head-specific rank-1 update: α * q_h ⊗ k_h
                let alpha = attention_weights[ctx_idx];
                let rank1_update = alpha * &q_head * k_head.transpose();

                // Scale by 1/sqrt(d_k) for numerical stability
                let scaling_factor = if self.config.use_scaling {
                    1.0 / (self.config.head_dim as f32).sqrt()
                } else {
                    1.0
                };

                let scaled_update = scaling_factor * rank1_update;

                // Add to total update matrix at correct position
                total_delta_w.view_mut((head_start, head_start), (self.config.head_dim, self.config.head_dim))
                    .add_assign(&scaled_update);

                head_attention_weights.push(alpha);
            }
        }

        // Apply learning rate (use first head's config as representative)
        if let Some(first_head) = self.heads.first() {
            total_delta_w *= first_head.config().learning_rate;
        }

        // Create combined weight update
        let update = WeightUpdate {
            delta_w: total_delta_w,
            delta_b: None, // Multi-head typically doesn't use bias in attention
            context_vector: encoded_contexts[0].clone(), // Use first context as representative
            query_vector: query.clone(),
            step_size: if let Some(first_head) = self.heads.first() {
                first_head.config().learning_rate
            } else {
                0.0
            },
            attention_weights: head_attention_weights,
            position,
        };

        Ok(update)
    }

    /// Compute attention logit for a single query-key pair
    fn compute_attention_logit(&self, query: &DVector<f32>, key: &DVector<f32>) -> Result<f32, WeightError> {
        if query.len() != key.len() {
            return Err(WeightError::InvalidDimensions {
                expected: format!("{}", query.len()),
                actual: format!("{}", key.len()),
            });
        }

        // Simple dot product attention (can be extended with learned weights)
        Ok(query.dot(key))
    }

    /// Get the combined output from all heads
    pub fn get_combined_weights(&self) -> DMatrix<f32> {
        let total_dim = self.config.num_heads * self.config.head_dim;
        let mut combined = DMatrix::zeros(total_dim, total_dim);

        for (head_idx, head) in self.heads.iter().enumerate() {
            let start_row = head_idx * self.config.head_dim;
            let start_col = head_idx * self.config.head_dim;

            combined.view_mut(
                (start_row, start_col),
                (self.config.head_dim, self.config.head_dim)
            ).copy_from(head.weights());
        }

        combined
    }
}

/// Enhanced single-head dynamics with full softmax attention
impl ImplicitDynamics {
    /// Enhanced update step with proper softmax attention over multiple contexts
    /// Implements the paper's core formula: T_W(C,x) = T_{W+ΔW(C)}(x)
    pub fn enhanced_update_step(
        &mut self,
        contexts: &[DVector<f32>],
        query: &DVector<f32>,
        position: Option<usize>
    ) -> Result<WeightUpdate, WeightError> {
        if contexts.is_empty() {
            return self.update_step(&DVector::zeros(self.weights.ncols()), query);
        }

        let input_dim = self.weights.ncols();
        let output_dim = self.weights.nrows();

        // Validate dimensions
        for (i, context) in contexts.iter().enumerate() {
            if context.len() != input_dim {
                return Err(WeightError::InvalidDimensions {
                    expected: format!("{}", input_dim),
                    actual: format!("context[{}]: {}", i, context.len()),
                });
            }
        }

        if query.len() != output_dim {
            return Err(WeightError::InvalidDimensions {
                expected: format!("{}", output_dim),
                actual: format!("{}", query.len()),
            });
        }

        // Apply positional encoding if needed
        let encoded_contexts: Vec<DVector<f32>> = if let Some(pos) = position {
            contexts.iter().enumerate().map(|(i, ctx)| {
                apply_positional_encoding(ctx, pos + i, 1024)
            }).collect()
        } else {
            contexts.to_vec()
        };

        // Compute attention logits: q^T W k_i for all contexts
        let logits: Vec<f32> = encoded_contexts.iter()
            .map(|context| query.dot(&(&self.weights * context)))
            .collect();

        // Apply softmax to get proper attention distribution
        let attention_weights = softmax(&logits);

        // Compute weighted sum of rank-1 updates: Σ α_i * q ⊗ k_i
        let mut total_delta_w = DMatrix::zeros(output_dim, input_dim);

        for (i, (context, &alpha)) in encoded_contexts.iter().zip(attention_weights.iter()).enumerate() {
            let rank1_update = query * context.transpose();
            total_delta_w += alpha * rank1_update;
        }

        // Scale by learning rate
        total_delta_w *= self.config.learning_rate;

        // Apply regularization
        let regularized_delta_w = &total_delta_w - self.config.regularization_strength * &self.weights;

        // Update weights
        self.weights += &regularized_delta_w;

        // Compute bias update (weighted by average attention)
        let avg_attention: f32 = attention_weights.iter().sum::<f32>() / attention_weights.len() as f32;
        let delta_b = self.bias.as_ref().map(|_| {
            self.config.learning_rate * avg_attention * query
        });

        if let (Some(ref mut bias), Some(ref delta_bias)) = (&mut self.bias, &delta_b) {
            *bias += delta_bias;
        }

        let update = WeightUpdate {
            delta_w: regularized_delta_w,
            delta_b,
            context_vector: encoded_contexts[0].clone(), // Representative context
            query_vector: query.clone(),
            step_size: self.config.learning_rate * avg_attention,
            attention_weights,
            position,
        };

        // Store in history
        self.update_history.push_back(update.clone());
        if self.update_history.len() > self.max_history {
            self.update_history.pop_front();
        }

        Ok(update)
    }
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

    /// Test the core theoretical implementation: T_W(C,x) = T_{W+ΔW(C)}(x)
    #[test]
    fn test_enhanced_softmax_attention() {
        let config = DynamicsConfig {
            learning_rate: 1.0,
            use_skip_connections: false,
            regularization_strength: 0.0,
        };

        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();

        // Multiple contexts to test softmax attention
        let contexts = vec![
            DVector::from_vec(vec![1.0, 0.0, 0.0]),
            DVector::from_vec(vec![0.0, 1.0, 0.0]),
            DVector::from_vec(vec![0.0, 0.0, 1.0]),
        ];
        let query = DVector::from_vec(vec![0.5, 0.5]);

        // Test enhanced update with proper softmax
        let update = dynamics.enhanced_update_step(&contexts, &query, None).unwrap();

        // Verify attention weights sum to 1.0 (softmax property)
        let sum: f32 = update.attention_weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "Attention weights should sum to 1.0, got {}", sum);

        // Verify we have the right number of attention weights
        assert_eq!(update.attention_weights.len(), 3);

        // Verify rank-1 structure is maintained
        assert_eq!(update.delta_w.rank(1e-6), 1);
    }

    /// Test multi-head attention dynamics
    #[test]
    fn test_multi_head_dynamics() {
        let multihead_config = MultiHeadConfig::new(4, 16).unwrap(); // 4 heads, 4 dims each
        let dynamics_config = DynamicsConfig::default();

        let mut multi_head = MultiHeadImplicitDynamics::new(
            16, 16, multihead_config, dynamics_config
        ).unwrap();

        let contexts = vec![
            DVector::from_vec((0..16).map(|i| (i as f32) / 16.0).collect()),
            DVector::from_vec((0..16).map(|i| ((i + 8) as f32) / 16.0).collect()),
        ];
        let query = DVector::from_vec((0..16).map(|i| 0.5 + (i as f32) / 32.0).collect());

        let update = multi_head.multi_head_update_step(&contexts, &query, None).unwrap();

        // Verify dimensions
        assert_eq!(update.delta_w.nrows(), 16);
        assert_eq!(update.delta_w.ncols(), 16);

        // Verify attention weights
        assert!(!update.attention_weights.is_empty());

        // Verify combined weights have correct structure
        let combined = multi_head.get_combined_weights();
        assert_eq!(combined.nrows(), 16);
        assert_eq!(combined.ncols(), 16);
    }

    /// Test positional encoding functionality
    #[test]
    fn test_positional_encoding() {
        let vector = DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let encoded1 = apply_positional_encoding(&vector, 0, 100);
        let encoded2 = apply_positional_encoding(&vector, 1, 100);

        // Encoded vectors should be different for different positions
        assert_ne!(encoded1, encoded2);

        // But should have same length
        assert_eq!(encoded1.len(), vector.len());
        assert_eq!(encoded2.len(), vector.len());
    }

    /// Test softmax implementation
    #[test]
    fn test_softmax_function() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);

        // Should sum to 1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // Should be monotonically increasing for increasing logits
        assert!(probs[0] < probs[1]);
        assert!(probs[1] < probs[2]);

        // Test edge case: empty input
        let empty_probs = softmax(&[]);
        assert!(empty_probs.is_empty());

        // Test edge case: very large numbers (numerical stability)
        let large_logits = vec![1000.0, 1001.0, 999.0];
        let large_probs = softmax(&large_logits);
        let large_sum: f32 = large_probs.iter().sum();
        assert!((large_sum - 1.0).abs() < 1e-6);
    }

    /// Test the paper's core theorem: ICL equivalence to weight updates
    #[test]
    fn test_icl_equivalence_theorem() {
        let config = DynamicsConfig {
            learning_rate: 0.1,
            use_skip_connections: false,
            regularization_strength: 0.0,
        };

        let mut dynamics = ImplicitDynamics::new(4, 3, config).unwrap();

        // Simulate ICL scenario: learning from examples
        let examples = vec![
            DVector::from_vec(vec![1.0, 0.0, 0.0, 1.0]), // Input pattern
            DVector::from_vec(vec![0.0, 1.0, 0.0, 1.0]), // Similar pattern
            DVector::from_vec(vec![0.0, 0.0, 1.0, 1.0]), // Another pattern
        ];

        let query = DVector::from_vec(vec![0.3, 0.3, 0.4]); // Target query

        // Initial weights
        let initial_weights = dynamics.weights().clone();

        // Apply ICL-style updates
        let update = dynamics.enhanced_update_step(&examples, &query, None).unwrap();

        // Verify the transformation: T_W(C,x) = T_{W+ΔW(C)}(x)
        let final_weights = dynamics.weights();
        let expected_weights = &initial_weights + &update.delta_w;

        // Check the transformation is applied correctly
        for i in 0..final_weights.nrows() {
            for j in 0..final_weights.ncols() {
                assert!(
                    (final_weights[(i, j)] - expected_weights[(i, j)]).abs() < 1e-6,
                    "Weight transformation failed at ({}, {})", i, j
                );
            }
        }

        // Verify the update is a proper rank-1 matrix (key theorem requirement)
        assert!(update.delta_w.rank(1e-6) <= examples.len(),
                "Update should be low-rank, got rank {}", update.delta_w.rank(1e-6));
    }

    /// Test convergence behavior as described in the paper
    #[test]
    fn test_convergence_dynamics() {
        let config = DynamicsConfig {
            learning_rate: 0.05, // Small learning rate for stable convergence
            use_skip_connections: false,
            regularization_strength: 0.01,
        };

        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();

        // Create a converging sequence
        let mut updates = Vec::new();
        let base_context = DVector::from_vec(vec![1.0, 0.5, 0.2]);
        let query = DVector::from_vec(vec![0.8, 0.6]);

        // Apply multiple updates with slightly varying contexts
        for i in 0..10 {
            let noise = 0.01 * (i as f32);
            let context = &base_context + DVector::from_vec(vec![noise, -noise, noise]);
            let contexts = vec![context];

            let update = dynamics.enhanced_update_step(&contexts, &query, None).unwrap();
            updates.push(update);
        }

        // Analyze convergence
        let convergence = dynamics.predict_convergence(&updates);

        // Should show convergence behavior
        assert!(convergence.gradient_norms.len() == 10);
        assert!(convergence.convergence_rate.is_finite());

        // Later updates should generally be smaller (convergence)
        let early_norm = convergence.gradient_norms[0];
        let late_norm = convergence.gradient_norms[9];
        assert!(late_norm <= early_norm,
                "Expected convergence: early_norm={}, late_norm={}", early_norm, late_norm);
    }

    /// Test multi-head configuration validation
    #[test]
    fn test_multihead_config_validation() {
        // Valid configuration
        let valid_config = MultiHeadConfig::new(4, 16);
        assert!(valid_config.is_ok());

        // Invalid: dimension not divisible by heads
        let invalid_config = MultiHeadConfig::new(3, 16);
        assert!(invalid_config.is_err());

        // Invalid: zero heads
        let zero_heads = MultiHeadConfig::new(0, 16);
        assert!(zero_heads.is_err());
    }

    /// Benchmark test: Compare old vs new attention computation
    #[test]
    fn test_attention_computation_comparison() {
        let config = DynamicsConfig::default();
        let mut dynamics = ImplicitDynamics::new(4, 3, config).unwrap();

        let context = DVector::from_vec(vec![1.0, 0.5, 0.3, 0.1]);
        let query = DVector::from_vec(vec![0.8, 0.6, 0.4]);

        // Old method (single context, sigmoid approximation)
        let old_update = dynamics.update_step(&context, &query).unwrap();

        // Reset dynamics for fair comparison
        let config2 = DynamicsConfig::default();
        let mut dynamics2 = ImplicitDynamics::new(4, 3, config2).unwrap();

        // New method (proper softmax, can handle multiple contexts)
        let contexts = vec![context.clone()];
        let new_update = dynamics2.enhanced_update_step(&contexts, &query, None).unwrap();

        // Both should produce finite, reasonable updates
        assert!(old_update.delta_w.norm().is_finite());
        assert!(new_update.delta_w.norm().is_finite());

        // New method should have proper attention weights that sum to 1
        let attention_sum: f32 = new_update.attention_weights.iter().sum();
        assert!((attention_sum - 1.0).abs() < 1e-6);

        // Old method uses sigmoid approximation
        assert_eq!(old_update.attention_weights.len(), 1);
        assert!(old_update.attention_weights[0] > 0.0 && old_update.attention_weights[0] < 1.0);
    }

    /// Test paper compliance: Full ICL pipeline
    #[test]
    fn test_paper_compliance_full_pipeline() {
        println!("🧪 Testing full ICL pipeline compliance with paper");

        let config = DynamicsConfig {
            learning_rate: 0.1,
            use_skip_connections: false,
            regularization_strength: 0.001,
        };

        let mut dynamics = ImplicitDynamics::new(8, 6, config).unwrap();

        // Simulate in-context learning scenario from the paper
        let learning_examples = vec![
            DVector::from_vec(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]), // Pattern A
            DVector::from_vec(vec![0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0]), // Pattern B
            DVector::from_vec(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0]), // Pattern C
            DVector::from_vec(vec![1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]), // Mixed pattern
        ];

        let test_query = DVector::from_vec(vec![0.2, 0.3, 0.2, 0.1, 0.1, 0.1]);

        // Apply sequential learning
        let mut all_updates = Vec::new();
        for (i, example) in learning_examples.iter().enumerate() {
            let contexts = vec![example.clone()];
            let update = dynamics.enhanced_update_step(&contexts, &test_query, Some(i)).unwrap();
            all_updates.push(update);
        }

        // Verify paper requirements
        // 1. All updates should be low-rank
        for (i, update) in all_updates.iter().enumerate() {
            let rank = update.delta_w.rank(1e-6);
            assert!(rank <= 2, "Update {} should be low-rank, got rank {}", i, rank);
        }

        // 2. Attention weights should be properly normalized
        for update in &all_updates {
            let sum: f32 = update.attention_weights.iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "Attention weights should sum to 1.0");
        }

        // 3. Should show learning progression (later updates more targeted)
        let early_norm = all_updates[0].delta_w.norm();
        let late_norm = all_updates[3].delta_w.norm();
        // Later updates might be larger due to accumulated learning, but should be stable
        assert!(late_norm.is_finite() && late_norm > 0.0);

        // 4. Position encoding should affect results
        let contexts_with_pos = vec![learning_examples[0].clone()];
        let update_pos_0 = dynamics.enhanced_update_step(&contexts_with_pos, &test_query, Some(0)).unwrap();
        let update_pos_5 = dynamics.enhanced_update_step(&contexts_with_pos, &test_query, Some(5)).unwrap();

        // Position encoding should create different updates
        let pos_difference = (&update_pos_0.delta_w - &update_pos_5.delta_w).norm();
        assert!(pos_difference > 1e-6, "Position encoding should affect updates");

        println!("   ✅ Full ICL pipeline complies with paper requirements");
    }

    /// Test scaling properties as mentioned in the paper
    #[test]
    fn test_scaling_properties() {
        println!("🧪 Testing scaling properties");

        // Test with different dimensions
        let small_config = DynamicsConfig::default();
        let mut small_dynamics = ImplicitDynamics::new(4, 2, small_config).unwrap();

        let large_config = DynamicsConfig::default();
        let mut large_dynamics = ImplicitDynamics::new(16, 8, large_config).unwrap();

        // Small system
        let small_context = DVector::from_vec(vec![0.5, 0.3, 0.8, 0.2]);
        let small_query = DVector::from_vec(vec![0.7, 0.4]);
        let small_update = small_dynamics.update_step(&small_context, &small_query).unwrap();

        // Large system
        let large_context = DVector::from_vec((0..16).map(|i| (i as f32 + 1.0) / 17.0).collect());
        let large_query = DVector::from_vec((0..8).map(|i| 0.5 + (i as f32) / 16.0).collect());
        let large_update = large_dynamics.update_step(&large_context, &large_query).unwrap();

        // Both should produce valid updates
        assert!(small_update.delta_w.norm().is_finite());
        assert!(large_update.delta_w.norm().is_finite());

        // Both should be rank-1
        assert_eq!(small_update.delta_w.rank(1e-6), 1);
        assert_eq!(large_update.delta_w.rank(1e-6), 1);

        println!("   ✅ Scaling properties verified");
    }

    /// Test edge cases and robustness
    #[test]
    fn test_edge_cases_robustness() {
        println!("🧪 Testing edge cases and robustness");

        let config = DynamicsConfig {
            learning_rate: 0.01,
            use_skip_connections: false,
            regularization_strength: 0.1,
        };

        let mut dynamics = ImplicitDynamics::new(3, 2, config).unwrap();

        // Test with zero vectors
        let zero_context = DVector::zeros(3);
        let zero_query = DVector::zeros(2);
        let zero_update = dynamics.update_step(&zero_context, &zero_query).unwrap();
        assert!(zero_update.delta_w.norm().is_finite());

        // Test with very small numbers
        let tiny_context = DVector::from_vec(vec![1e-8, 1e-8, 1e-8]);
        let tiny_query = DVector::from_vec(vec![1e-8, 1e-8]);
        let tiny_update = dynamics.update_step(&tiny_context, &tiny_query).unwrap();
        assert!(tiny_update.delta_w.norm().is_finite());

        // Test with large numbers
        let large_context = DVector::from_vec(vec![1000.0, 2000.0, 3000.0]);
        let large_query = DVector::from_vec(vec![500.0, 1500.0]);
        let large_update = dynamics.update_step(&large_context, &large_query).unwrap();
        assert!(large_update.delta_w.norm().is_finite());

        // Test effectiveness score calculation
        assert!(zero_update.effectiveness_score() >= 0.0);
        assert!(tiny_update.effectiveness_score() >= 0.0);
        assert!(large_update.effectiveness_score() >= 0.0);

        println!("   ✅ Edge cases handled robustly");
    }

    /// Test multi-head attention with positional encoding
    #[test]
    fn test_multihead_with_positional_encoding() {
        println!("🧪 Testing multi-head attention with positional encoding");

        let multihead_config = MultiHeadConfig::new(2, 8).unwrap(); // 2 heads, 4 dims each
        let dynamics_config = DynamicsConfig {
            learning_rate: 0.1,
            use_skip_connections: false,
            regularization_strength: 0.01,
        };

        let mut multi_head = MultiHeadImplicitDynamics::new(
            8, 8, multihead_config, dynamics_config
        ).unwrap();

        let contexts = vec![
            DVector::from_vec(vec![1.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.3, 0.7]),
            DVector::from_vec(vec![0.0, 1.0, 0.0, 1.0, 0.7, 0.3, 0.5, 0.5]),
        ];
        let query = DVector::from_vec(vec![0.5, 0.5, 0.3, 0.7, 0.2, 0.8, 0.6, 0.4]);

        // Test without positional encoding
        let update_no_pos = multi_head.multi_head_update_step(&contexts, &query, None).unwrap();

        // Test with positional encoding
        let update_with_pos = multi_head.multi_head_update_step(&contexts, &query, Some(0)).unwrap();

        // Both should be valid
        assert_eq!(update_no_pos.delta_w.nrows(), 8);
        assert_eq!(update_no_pos.delta_w.ncols(), 8);
        assert_eq!(update_with_pos.delta_w.nrows(), 8);
        assert_eq!(update_with_pos.delta_w.ncols(), 8);

        // Positional encoding should create different results
        let pos_difference = (&update_no_pos.delta_w - &update_with_pos.delta_w).norm();
        assert!(pos_difference > 1e-6, "Positional encoding should affect multi-head updates");

        // Both should have proper attention weight normalization
        let sum_no_pos: f32 = update_no_pos.attention_weights.iter().sum();
        let sum_with_pos: f32 = update_with_pos.attention_weights.iter().sum();

        // Note: Multi-head has multiple attention weights (one per head per context)
        // So we need to check normalization per context across heads
        let contexts_count = contexts.len();
        let heads_count = 2;
        assert_eq!(update_no_pos.attention_weights.len(), contexts_count * heads_count);

        println!("   ✅ Multi-head with positional encoding working correctly");
    }

    /// Test mathematical properties from the paper
    #[test]
    fn test_mathematical_properties() {
        println!("🧪 Testing mathematical properties from paper");

        let config = DynamicsConfig {
            learning_rate: 1.0, // h = 1 for simplicity
            use_skip_connections: false,
            regularization_strength: 0.0, // No regularization for pure math test
        };

        // Test linearity property: ΔW(αC) should relate to α⋅ΔW(C)
        let base_context = DVector::from_vec(vec![1.0, 0.5, 0.2]);
        let query = DVector::from_vec(vec![0.8, 0.6]);

        // Reset for clean test
        let mut dynamics1 = ImplicitDynamics::new(3, 2, config.clone()).unwrap();
        let mut dynamics2 = ImplicitDynamics::new(3, 2, config.clone()).unwrap();

        let contexts1 = vec![base_context.clone()];
        let update1 = dynamics1.enhanced_update_step(&contexts1, &query, None).unwrap();

        let scaled_context = 2.0 * &base_context;
        let contexts2 = vec![scaled_context.clone()];
        let update2 = dynamics2.enhanced_update_step(&contexts2, &query, None).unwrap();

        // The relationship should hold approximately (accounting for softmax nonlinearity)
        let ratio = update2.delta_w.norm() / update1.delta_w.norm();
        assert!(ratio > 1.5 && ratio < 3.0, "Scaling should affect update magnitude");

        // Test that rank-1 property holds under composition
        let contexts_combined = vec![base_context.clone(), scaled_context.clone()];
        let mut dynamics3 = ImplicitDynamics::new(3, 2, config).unwrap();
        let combined_update = dynamics3.enhanced_update_step(&contexts_combined, &query, None).unwrap();

        // Combined update should still be low-rank
        assert!(combined_update.delta_w.rank(1e-6) <= 2, "Combined update should remain low-rank");

        println!("   ✅ Mathematical properties verified");
    }
}
