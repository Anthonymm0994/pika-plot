//! Performance benchmarks for pika-engine.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pika_engine::{Engine, database::Database};
use pika_core::types::ImportOptions;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn create_test_csv(dir: &TempDir, rows: usize) -> PathBuf {
    let path = dir.path().join(format!("bench_data_{}.csv", rows));
    let mut content = String::from("id,value,category\n");
    
    for i in 0..rows {
        content.push_str(&format!("{},{},{}\n", 
            i, 
            i as f64 * 1.5, 
            if i % 3 == 0 { "A" } else if i % 3 == 1 { "B" } else { "C" }
        ));
    }
    
    std::fs::write(&path, content).unwrap();
    path
}

fn bench_csv_import(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    
    let mut group = c.benchmark_group("csv_import");
    group.sample_size(10); // Reduce sample size for longer operations
    
    for rows in [1000, 10000, 100000].iter() {
        let csv_path = create_test_csv(&temp_dir, *rows);
        
        group.throughput(criterion::Throughput::Bytes(
            std::fs::metadata(&csv_path).unwrap().len()
        ));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(rows), 
            rows, 
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let engine = Engine::new(None).await.unwrap();
                    let options = ImportOptions {
                        has_header: true,
                        delimiter: Some(','),
                        sample_size: None,
                    };
                    engine.import_file(&csv_path, options).await.unwrap()
                });
            }
        );
    }
    
    group.finish();
}

fn bench_query_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("query_execution");
    
    // Setup: Create engine with test data
    let engine = rt.block_on(async {
        let engine = Engine::new(None).await.unwrap();
        // Would import test data here
        engine
    });
    
    group.bench_function("simple_select", |b| {
        b.to_async(&rt).iter(|| async {
            engine.execute_query("SELECT 1").await.unwrap()
        });
    });
    
    group.bench_function("aggregation", |b| {
        b.to_async(&rt).iter(|| async {
            engine.execute_query(
                "SELECT category, COUNT(*), AVG(value) FROM test_data GROUP BY category"
            ).await.unwrap()
        });
    });
    
    group.finish();
}

fn bench_gpu_buffer_alignment(c: &mut Criterion) {
    use pika_engine::gpu::align_buffer_size;
    
    c.bench_function("buffer_alignment", |b| {
        b.iter(|| {
            // Benchmark the alignment calculation
            for size in [100, 1000, 10000, 100000, 1000000] {
                black_box(align_buffer_size(size));
            }
        });
    });
}

fn bench_cache_operations(c: &mut Criterion) {
    use pika_engine::cache::CacheManager;
    use std::sync::Arc;
    
    let cache = Arc::new(CacheManager::new_with_limit(100 * 1024 * 1024));
    
    let mut group = c.benchmark_group("cache_operations");
    
    group.bench_function("cache_insert", |b| {
        let mut counter = 0;
        b.iter(|| {
            let key = format!("query_{}", counter);
            let value = vec![0u8; 1024]; // 1KB value
            cache.put_query(key, value);
            counter += 1;
        });
    });
    
    group.bench_function("cache_lookup_hit", |b| {
        // Pre-populate cache
        for i in 0..100 {
            cache.put_query(format!("test_{}", i), vec![0u8; 1024]);
        }
        
        b.iter(|| {
            black_box(cache.get_query("test_50"));
        });
    });
    
    group.bench_function("cache_lookup_miss", |b| {
        b.iter(|| {
            black_box(cache.get_query("nonexistent_key"));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_csv_import,
    bench_query_execution,
    bench_gpu_buffer_alignment,
    bench_cache_operations
);
criterion_main!(benches); 