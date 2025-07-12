use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use pika_engine::plot::{extract_xy_points, extract_numeric_values, extract_category_values};
use std::sync::Arc;

fn create_test_batch(size: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("category", DataType::Utf8, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let x_data: Vec<f64> = (0..size).map(|i| i as f64).collect();
    let y_data: Vec<f64> = (0..size).map(|i| (i as f64).sin() * 100.0).collect();
    let categories: Vec<String> = (0..size)
        .map(|i| format!("Category_{}", i % 10))
        .collect();
    let values: Vec<i64> = (0..size).map(|i| (i * 10) as i64).collect();

    let x_array = Float64Array::from(x_data);
    let y_array = Float64Array::from(y_data);
    let category_array = StringArray::from(categories);
    let value_array = Int64Array::from(values);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(x_array),
            Arc::new(y_array),
            Arc::new(category_array),
            Arc::new(value_array),
        ],
    ).unwrap()
}

fn benchmark_extract_xy_points(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_xy_points");
    
    for size in [100, 1000, 10_000, 100_000].iter() {
        let batch = create_test_batch(*size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &batch,
            |b, batch| {
                b.iter(|| {
                    let points = extract_xy_points(black_box(batch), "x", "y").unwrap();
                    black_box(points);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_extract_numeric_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_numeric_values");
    
    for size in [100, 1000, 10_000, 100_000].iter() {
        let batch = create_test_batch(*size);
        let array = batch.column(0); // x column
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            array,
            |b, array| {
                b.iter(|| {
                    let values = extract_numeric_values(black_box(array)).unwrap();
                    black_box(values);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_extract_category_values(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_category_values");
    
    for size in [100, 1000, 10_000, 100_000].iter() {
        let batch = create_test_batch(*size);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &batch,
            |b, batch| {
                b.iter(|| {
                    let pairs = extract_category_values(
                        black_box(batch),
                        "category",
                        "value"
                    ).unwrap();
                    black_box(pairs);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_extract_xy_points,
    benchmark_extract_numeric_values,
    benchmark_extract_category_values
);
criterion_main!(benches); 