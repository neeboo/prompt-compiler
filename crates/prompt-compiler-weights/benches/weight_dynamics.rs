use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prompt_compiler_weights::{
    ImplicitDynamics, DynamicsConfig, WeightUpdate, WeightUpdateMetadata,
    create_random_weights, create_random_vector
};

fn benchmark_weight_update_creation(c: &mut Criterion) {
    let weights = create_random_weights(64, 64);
    let context_vector = create_random_vector(64);
    let query_vector = create_random_vector(64);
    
    let metadata = WeightUpdateMetadata {
        target_model: "transformer".to_string(),
        layer_id: "mlp_0".to_string(),
        context_length: 1,
        computed_at: 0,
        update_norm: 0.0,
    };

    c.bench_function("weight_update_creation", |b| {
        b.iter(|| {
            WeightUpdate::new(
                black_box(&weights),
                black_box(context_vector.clone()),
                black_box(query_vector.clone()),
                black_box(metadata.clone()),
            )
        })
    });
}

fn benchmark_sequential_updates(c: &mut Criterion) {
    let weights = create_random_weights(32, 32);
    let config = DynamicsConfig::default();
    let dynamics = ImplicitDynamics::new(weights, config);
    
    let context_tokens: Vec<_> = (0..10)
        .map(|_| create_random_vector(32))
        .collect();
    let query = create_random_vector(32);

    c.bench_function("sequential_updates_10", |b| {
        b.iter(|| {
            dynamics.compute_sequential_updates(
                black_box(&context_tokens),
                black_box(&query),
            )
        })
    });
}

fn benchmark_convergence_prediction(c: &mut Criterion) {
    let weights = create_random_weights(16, 16);
    let config = DynamicsConfig::default();
    let dynamics = ImplicitDynamics::new(weights, config);
    
    let context_tokens: Vec<_> = (0..5)
        .map(|_| create_random_vector(16))
        .collect();
    let query = create_random_vector(16);
    
    let updates = dynamics.compute_sequential_updates(&context_tokens, &query).unwrap();

    c.bench_function("convergence_prediction", |b| {
        b.iter(|| {
            dynamics.predict_convergence(black_box(&updates))
        })
    });
}

criterion_group!(
    benches,
    benchmark_weight_update_creation,
    benchmark_sequential_updates,
    benchmark_convergence_prediction
);
criterion_main!(benches);
