use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::time::Instant;

// Mock RocksDB-like storage implementation for benchmarking
struct MockStorage {
    data: HashMap<String, Vec<u8>>,
    cache: HashMap<String, Vec<u8>>,
    cache_size_limit: usize,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            cache: HashMap::new(),
            cache_size_limit: 1000,
        }
    }

    fn put(&mut self, key: &str, value: &[u8]) -> Result<(), &'static str> {
        self.data.insert(key.to_string(), value.to_vec());
        
        // Update cache
        if self.cache.len() < self.cache_size_limit {
            self.cache.insert(key.to_string(), value.to_vec());
        }
        
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Try cache first
        if let Some(value) = self.cache.get(key) {
            return Some(value.clone());
        }
        
        // Fallback to main storage
        self.data.get(key).cloned()
    }

    fn batch_put(&mut self, items: &[(String, Vec<u8>)]) -> Result<(), &'static str> {
        for (key, value) in items {
            self.put(key, value)?;
        }
        Ok(())
    }

    fn range_query(&self, start: &str, end: &str) -> Vec<(String, Vec<u8>)> {
        self.data.iter()
            .filter(|(k, _)| k.as_str() >= start && k.as_str() < end)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn delete(&mut self, key: &str) -> bool {
        self.cache.remove(key);
        self.data.remove(key).is_some()
    }
}

fn generate_test_data(count: usize) -> Vec<(String, Vec<u8>)> {
    (0..count)
        .map(|i| {
            let key = format!("key_{:06}", i);
            let value = format!("value_{}", i).repeat(10).into_bytes();
            (key, value)
        })
        .collect()
}

fn bench_single_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_writes");
    
    for &count in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(count);
        
        group.bench_with_input(
            BenchmarkId::new("record_count", count),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let mut storage = MockStorage::new();
                    for (key, value) in data.iter() {
                        black_box(storage.put(black_box(key), black_box(value)).unwrap());
                    }
                    black_box(storage)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_batch_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_writes");
    
    for &count in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(count);
        
        group.bench_with_input(
            BenchmarkId::new("batch_size", count),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let mut storage = MockStorage::new();
                    black_box(storage.batch_put(black_box(data)).unwrap());
                    black_box(storage)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_point_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_reads");
    
    for &count in [100, 1000, 10000].iter() {
        let test_data = generate_test_data(count);
        let mut storage = MockStorage::new();
        
        // Pre-populate storage
        for (key, value) in &test_data {
            storage.put(key, value).unwrap();
        }
        
        group.bench_with_input(
            BenchmarkId::new("dataset_size", count),
            &test_data,
            |b, data| {
                b.iter(|| {
                    for (key, _) in data.iter().take(100) { // Read first 100 items
                        black_box(storage.get(black_box(key)));
                    }
                })
            },
        );
    }
    
    group.finish();
}

fn bench_range_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");
    
    for &count in [1000, 5000, 10000].iter() {
        let test_data = generate_test_data(count);
        let mut storage = MockStorage::new();
        
        // Pre-populate storage
        for (key, value) in &test_data {
            storage.put(key, value).unwrap();
        }
        
        group.bench_with_input(
            BenchmarkId::new("dataset_size", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let start = "key_000000";
                    let end = "key_000100";
                    black_box(storage.range_query(black_box(start), black_box(end)))
                })
            },
        );
    }
    
    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    
    let test_data = generate_test_data(1000);
    let mut storage = MockStorage::new();
    
    // Pre-populate with some data
    for (key, value) in test_data.iter().take(500) {
        storage.put(key, value).unwrap();
    }
    
    group.bench_function("read_write_mix", |b| {
        b.iter(|| {
            let mut local_storage = MockStorage::new();
            
            // Copy existing data
            for (key, value) in storage.data.iter() {
                local_storage.put(key, value).unwrap();
            }
            
            // Mixed operations: 70% reads, 20% writes, 10% deletes
            for i in 0..100 {
                match i % 10 {
                    0 => {
                        // Delete operation
                        let key = format!("key_{:06}", i % 500);
                        black_box(local_storage.delete(black_box(&key)));
                    }
                    1 | 2 => {
                        // Write operation
                        let key = format!("key_{:06}", 500 + i);
                        let value = format!("new_value_{}", i).into_bytes();
                        black_box(local_storage.put(black_box(&key), black_box(&value)).unwrap());
                    }
                    _ => {
                        // Read operation
                        let key = format!("key_{:06}", i % 500);
                        black_box(local_storage.get(black_box(&key)));
                    }
                }
            }
            
            black_box(local_storage)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_writes,
    bench_batch_writes,
    bench_point_reads,
    bench_range_queries,
    bench_mixed_workload
);
criterion_main!(benches);
