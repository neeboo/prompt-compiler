use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prompt_compiler_weights::{create_random_vector, DynamicsConfig, ImplicitDynamics};

fn benchmark_weight_update_creation(c: &mut Criterion) {
    let context_vector = create_random_vector(64);
    let query_vector = create_random_vector(64);
    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(64, 64, config).unwrap();

    c.bench_function("weight_update_creation", |b| {
        b.iter(|| dynamics.update_step(black_box(&context_vector), black_box(&query_vector)))
    });
}

fn benchmark_sequential_updates(c: &mut Criterion) {
    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(32, 32, config).unwrap();

    let context_tokens: Vec<_> = (0..10).map(|_| create_random_vector(32)).collect();
    let query = create_random_vector(32);

    c.bench_function("sequential_updates_10", |b| {
        b.iter(|| {
            dynamics.compute_sequential_updates(black_box(&context_tokens), black_box(&query))
        })
    });
}

fn benchmark_convergence_prediction(c: &mut Criterion) {
    let config = DynamicsConfig::default();
    let mut dynamics = ImplicitDynamics::new(16, 16, config).unwrap();

    let context_tokens: Vec<_> = (0..5).map(|_| create_random_vector(16)).collect();
    let query = create_random_vector(16);

    let updates = dynamics
        .compute_sequential_updates(&context_tokens, &query)
        .unwrap();

    c.bench_function("convergence_prediction", |b| {
        b.iter(|| dynamics.predict_convergence(black_box(&updates)))
    });
}

criterion_group!(
    benches,
    benchmark_weight_update_creation,
    benchmark_sequential_updates,
    benchmark_convergence_prediction
);
criterion_main!(benches);
