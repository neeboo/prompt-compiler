use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Mock weight dynamics implementation for benchmarking
struct WeightDynamics {
    dimension: usize,
    learning_rate: f64,
    convergence_threshold: f64,
}

impl WeightDynamics {
    fn new(dimension: usize) -> Self {
        Self {
            dimension,
            learning_rate: 0.01,
            convergence_threshold: 1e-4,
        }
    }

    fn compute_weight_update(&self, context: &[f64], target: &[f64]) -> Vec<f64> {
        let mut delta_w = vec![0.0; self.dimension * self.dimension];
        
        for i in 0..self.dimension {
            for j in 0..self.dimension {
                let idx = i * self.dimension + j;
                delta_w[idx] = self.learning_rate * context[i] * target[j];
            }
        }
        
        delta_w
    }

    fn converge_weights(&self, contexts: &[Vec<f64>], targets: &[Vec<f64>]) -> (Vec<f64>, usize) {
        let mut weights = vec![0.0; self.dimension * self.dimension];
        let mut iterations = 0;
        
        loop {
            let mut total_error = 0.0;
            
            for (context, target) in contexts.iter().zip(targets.iter()) {
                let delta_w = self.compute_weight_update(context, target);
                
                for (w, dw) in weights.iter_mut().zip(delta_w.iter()) {
                    *w += dw;
                }
                
                // Compute error (simplified)
                let error: f64 = delta_w.iter().map(|x| x * x).sum::<f64>().sqrt();
                total_error += error;
            }
            
            iterations += 1;
            
            if total_error < self.convergence_threshold || iterations > 200 {
                break;
            }
        }
        
        (weights, iterations)
    }
}

fn generate_test_data(size: usize, dimension: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut contexts = Vec::new();
    let mut targets = Vec::new();
    
    for i in 0..size {
        let context: Vec<f64> = (0..dimension)
            .map(|j| ((i + j) as f64 * 0.1).sin())
            .collect();
        
        let target: Vec<f64> = (0..dimension)
            .map(|j| ((i + j) as f64 * 0.1).cos())
            .collect();
            
        contexts.push(context);
        targets.push(target);
    }
    
    (contexts, targets)
}

fn bench_weight_update_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("weight_update_creation");
    
    for dimension in [64, 128, 256, 512].iter() {
        let weight_dynamics = WeightDynamics::new(*dimension);
        let context = vec![0.5; *dimension];
        let target = vec![0.3; *dimension];
        
        group.bench_with_input(
            BenchmarkId::new("dimension", dimension),
            dimension,
            |b, _| {
                b.iter(|| {
                    black_box(weight_dynamics.compute_weight_update(
                        black_box(&context),
                        black_box(&target)
                    ))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_convergence_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("convergence_prediction");
    group.measurement_time(Duration::from_secs(10));
    
    for &context_count in [10, 50, 100].iter() {
        let dimension = 128;
        let weight_dynamics = WeightDynamics::new(dimension);
        let (contexts, targets) = generate_test_data(context_count, dimension);
        
        group.bench_with_input(
            BenchmarkId::new("contexts", context_count),
            &context_count,
            |b, _| {
                b.iter(|| {
                    black_box(weight_dynamics.converge_weights(
                        black_box(&contexts),
                        black_box(&targets)
                    ))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_sequential_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_updates_10");
    
    let dimension = 128;
    let weight_dynamics = WeightDynamics::new(dimension);
    let (contexts, targets) = generate_test_data(10, dimension);
    
    group.bench_function("sequential_10_contexts", |b| {
        b.iter(|| {
            let mut weights = vec![0.0; dimension * dimension];
            
            for (context, target) in contexts.iter().zip(targets.iter()) {
                let delta_w = weight_dynamics.compute_weight_update(
                    black_box(context),
                    black_box(target)
                );
                
                for (w, dw) in weights.iter_mut().zip(delta_w.iter()) {
                    *w += dw;
                }
            }
            
            black_box(weights)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_weight_update_creation,
    bench_convergence_prediction,
    bench_sequential_updates
);
criterion_main!(benches);
