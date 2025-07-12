use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pika_engine::Engine;
use tokio::runtime::Runtime;
use tempfile::TempDir;
use std::path::Path;

fn create_test_csv(dir: &Path, rows: usize) -> std::path::PathBuf {
    let csv_path = dir.join(format!("test_{}.csv", rows));
    let mut wtr = csv::Writer::from_path(&csv_path).unwrap();
    
    // Write header
    wtr.write_record(&["id", "value", "category", "timestamp"]).unwrap();
    
    // Write data
    for i in 0..rows {
        wtr.write_record(&[
            i.to_string(),
            (i as f64 * 1.5).to_string(),
            format!("cat_{}", i % 10),
            format!("2024-01-{:02}", (i % 28) + 1),
        ]).unwrap();
    }
    
    wtr.flush().unwrap();
    csv_path
}

fn benchmark_csv_import(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("csv_import");
    
    for rows in [100, 1000, 10_000].iter() {
        let temp_dir = TempDir::new().unwrap();
        let csv_path = create_test_csv(temp_dir.path(), *rows);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(rows),
            &csv_path,
            |b, path| {
                b.iter(|| {
                    rt.block_on(async {
                        let engine = Engine::new().await.unwrap();
                        engine.import_csv(black_box(path), "test_table").await.unwrap();
                    });
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_simple_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("simple_queries");
    
    // Setup: Create engine with test data
    let temp_dir = TempDir::new().unwrap();
    let csv_path = create_test_csv(temp_dir.path(), 10_000);
    
    let engine = rt.block_on(async {
        let engine = Engine::new().await.unwrap();
        engine.import_csv(&csv_path, "test_table").await.unwrap();
        engine
    });
    
    // Benchmark different queries
    let queries = vec![
        ("select_all", "SELECT * FROM test_table"),
        ("count", "SELECT COUNT(*) FROM test_table"),
        ("filter", "SELECT * FROM test_table WHERE value > 5000"),
        ("group_by", "SELECT category, COUNT(*) FROM test_table GROUP BY category"),
        ("order_by", "SELECT * FROM test_table ORDER BY value DESC LIMIT 100"),
    ];
    
    for (name, query) in queries {
        group.bench_function(name, |b| {
            b.iter(|| {
                rt.block_on(async {
                    let result = engine.execute_query(black_box(query)).await.unwrap();
                    black_box(result);
                });
            });
        });
    }
    
    group.finish();
}

fn benchmark_aggregation_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("aggregation_queries");
    
    // Setup: Create engine with larger dataset
    let temp_dir = TempDir::new().unwrap();
    let csv_path = create_test_csv(temp_dir.path(), 100_000);
    
    let engine = rt.block_on(async {
        let engine = Engine::new().await.unwrap();
        engine.import_csv(&csv_path, "test_table").await.unwrap();
        engine
    });
    
    let queries = vec![
        ("sum", "SELECT SUM(value) FROM test_table"),
        ("avg", "SELECT AVG(value) FROM test_table"),
        ("min_max", "SELECT MIN(value), MAX(value) FROM test_table"),
        ("stddev", "SELECT STDDEV(value) FROM test_table"),
        ("percentile", "SELECT PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY value) FROM test_table"),
    ];
    
    for (name, query) in queries {
        group.bench_function(name, |b| {
            b.iter(|| {
                rt.block_on(async {
                    let result = engine.execute_query(black_box(query)).await.unwrap();
                    black_box(result);
                });
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_csv_import,
    benchmark_simple_queries,
    benchmark_aggregation_queries
);
criterion_main!(benches); 