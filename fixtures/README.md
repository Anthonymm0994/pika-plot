# Test Fixtures

This directory contains sample CSV files for testing Pika-Plot.

## Available Files

- **small.csv** (100 rows): Clean data with no nulls, perfect for unit tests
- **medium.csv** (50 rows): Contains null values, missing data, and edge cases

## Generating Large Test Files

For performance testing, you'll need to generate larger files:

```python
# generate_large_csv.py
import csv
import random
from datetime import datetime, timedelta

def generate_large_csv(filename, num_rows):
    products = ['Widget A', 'Widget B', 'Widget C', 'Widget D', 'Widget E',
                'Gadget A', 'Gadget B', 'Gadget C', 'Tool A', 'Tool B']
    categories = ['Electronics', 'Hardware', 'Software', 'Accessories']
    regions = ['North', 'South', 'East', 'West', 'Central']
    
    start_date = datetime(2024, 1, 1)
    
    with open(filename, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['date', 'product', 'category', 'quantity', 'price', 'customer_id', 'region'])
        
        for i in range(num_rows):
            date = start_date + timedelta(days=i % 365)
            product = random.choice(products)
            category = random.choice(categories)
            quantity = random.randint(1, 100)
            price = round(random.uniform(10.0, 199.99), 2)
            customer_id = f'CUST{random.randint(1, 10000):05d}'
            region = random.choice(regions)
            
            writer.writerow([
                date.strftime('%Y-%m-%d'),
                product,
                category,
                quantity,
                price,
                customer_id,
                region
            ])

# Generate test files
generate_large_csv('large.csv', 1_000_000)      # 1M rows for benchmarks
generate_large_csv('huge.csv', 50_000_000)      # 50M rows for stress tests
```

## Data Characteristics

### small.csv
- Date range: 2024-01-01 to 2024-02-20
- Products: 6 unique (Widget A-D, Gadget B-E, Tool C,F)
- Categories: 2 (Electronics, Hardware)
- Regions: 4 (North, South, East, West)
- No null values
- Consistent format

### medium.csv
- Contains various data quality issues:
  - Missing values (empty cells)
  - NULL strings
  - Missing product names
  - Missing customer IDs
  - Missing regions
- Good for testing error handling and type inference

## Usage in Tests

```rust
// Unit tests
#[test]
fn test_basic_import() {
    let path = Path::new("fixtures/small.csv");
    let result = import_csv(path, &ImportOptions::default()).await?;
    assert_eq!(result.row_count, 100);
}

// Integration tests  
#[test]
fn test_null_handling() {
    let path = Path::new("fixtures/medium.csv");
    let result = import_csv(path, &ImportOptions::default()).await?;
    // Should handle nulls gracefully
}

// Benchmarks
#[bench]
fn bench_large_import(b: &mut Bencher) {
    let path = Path::new("fixtures/large.csv");
    b.iter(|| {
        import_csv(path, &ImportOptions::default())
    });
}
``` 