use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

// Mock compression implementation for benchmarking
struct SemanticCompressor {
    compression_ratio: f64,
    quality_threshold: f64,
}

impl SemanticCompressor {
    fn new() -> Self {
        Self {
            compression_ratio: 0.7,
            quality_threshold: 0.95,
        }
    }

    fn compress_text(&self, text: &str) -> (Vec<u8>, f64) {
        // Simulate compression algorithm
        let original_size = text.len();
        let compressed_data = text.bytes()
            .enumerate()
            .filter(|(i, _)| i % 3 != 0) // Simple compression simulation
            .map(|(_, b)| b)
            .collect::<Vec<u8>>();
        
        let compression_ratio = compressed_data.len() as f64 / original_size as f64;
        (compressed_data, compression_ratio)
    }

    fn decompress_data(&self, data: &[u8]) -> String {
        // Simulate decompression
        String::from_utf8_lossy(data).to_string()
    }

    fn semantic_quality_score(&self, original: &str, compressed: &str) -> f64 {
        // Simplified semantic quality calculation
        let orig_words: Vec<&str> = original.split_whitespace().collect();
        let comp_words: Vec<&str> = compressed.split_whitespace().collect();
        
        let common_words = orig_words.iter()
            .filter(|word| comp_words.contains(word))
            .count();
        
        common_words as f64 / orig_words.len() as f64
    }
}

fn generate_test_texts() -> Vec<String> {
    vec![
        "The quick brown fox jumps over the lazy dog".repeat(10),
        "In the realm of artificial intelligence and machine learning, context compression plays a crucial role".repeat(20),
        "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety".repeat(15),
        "The implementation of weight dynamics in transformer models requires careful consideration of computational efficiency".repeat(25),
        "Semantic compression algorithms must balance compression ratio with information preservation".repeat(30),
    ]
}

fn bench_compression_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_speed");
    let compressor = SemanticCompressor::new();
    let test_texts = generate_test_texts();
    
    for (i, text) in test_texts.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("text_size", text.len()),
            text,
            |b, text| {
                b.iter(|| {
                    black_box(compressor.compress_text(black_box(text)))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_decompression_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_speed");
    let compressor = SemanticCompressor::new();
    let test_texts = generate_test_texts();
    
    // Pre-compress texts for decompression benchmark
    let compressed_data: Vec<_> = test_texts.iter()
        .map(|text| compressor.compress_text(text).0)
        .collect();
    
    for (i, data) in compressed_data.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("compressed_size", data.len()),
            data,
            |b, data| {
                b.iter(|| {
                    black_box(compressor.decompress_data(black_box(data)))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_quality_assessment(c: &mut Criterion) {
    let mut group = c.benchmark_group("quality_assessment");
    let compressor = SemanticCompressor::new();
    let test_texts = generate_test_texts();
    
    for (i, text) in test_texts.iter().enumerate() {
        let (compressed_data, _) = compressor.compress_text(text);
        let decompressed = compressor.decompress_data(&compressed_data);
        
        group.bench_with_input(
            BenchmarkId::new("text_length", text.len()),
            &(text, &decompressed),
            |b, (original, compressed)| {
                b.iter(|| {
                    black_box(compressor.semantic_quality_score(
                        black_box(original),
                        black_box(compressed)
                    ))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_compression_speed,
    bench_decompression_speed,
    bench_quality_assessment
);
criterion_main!(benches);
