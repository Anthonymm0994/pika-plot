# Test Data for Duplicate Detection Feature

This directory contains comprehensive test data for testing the duplicate detection functionality.

## Test Files Overview

### Simple Test Files
- **`simple_duplicates.csv`** - Basic test with obvious duplicates
- **`mixed_data_types.csv`** - Mixed data types with duplicates
- **`with_nulls.csv`** - Test data with null values to test null handling

### Complex Test Files
- **`test_complex.csv`** - Medium complexity dataset
- **`test_medium_complex.csv`** - Medium-sized complex dataset
- **`test_medium_complex_full.csv`** - Full medium complexity dataset
- **`test_large_complex.csv`** - Large complex dataset (~30MB, ~100k records)

### Other Test Files
- **`test_data.csv`** - Basic test data
- **`test_numeric.csv`** - Numeric data testing

## Test Scenarios

### 1. Simple Duplicates (`simple_duplicates.csv`)
- **Purpose**: Test basic duplicate detection
- **Structure**: 13 records, 5 groups (A-E)
- **Duplicates**: Clear 1:1 duplicates within groups
- **Columns**: group_id, name, value, timestamp, id

### 2. Mixed Data Types (`mixed_data_types.csv`)
- **Purpose**: Test duplicate detection with various data types
- **Structure**: 20 records, 10 groups (A-J)
- **Data Types**: Strings, integers, booleans, dates, floats
- **Columns**: group_id, name, age, salary, is_active, created_date, department, rating

### 3. Null Value Handling (`with_nulls.csv`)
- **Purpose**: Test how null values are handled in duplicate detection
- **Structure**: 26 records, 14 groups (A-N)
- **Null Patterns**: Various combinations of null values
- **Columns**: group_id, name, age, salary, is_active, created_date, department, rating

### 4. Large Complex Dataset (`test_large_complex.csv`)
- **Purpose**: Performance and scalability testing
- **Size**: ~30MB, ~100,000 records
- **Complexity**: Multiple data types, various duplicate patterns
- **Use Case**: Stress testing the duplicate detection algorithm

## Expected Results

### Simple Duplicates
- Group A: 3 identical records
- Group B: 3 identical records  
- Group C: 2 identical records
- Group D: 2 identical records
- Group E: 3 identical records

### Mixed Data Types
- Groups A, B, C, D, E, G, I, J: 2-3 duplicates each
- Groups F, H: No duplicates (control groups)

### Null Value Handling
- Groups A-J: Standard duplicates
- Group K: Null age, salary, created_date, department
- Group L: Null salary, rating
- Group M: Null age
- Group N: Null is_active, rating

## Usage Instructions

1. **Load any CSV file** into the application
2. **Click "🔍 Detect Duplicate Blocks"** in the sidebar
3. **Select group_id** as the grouping column
4. **Choose columns to ignore** (e.g., record_id, timestamp)
5. **Run detection** and review results
6. **Create clean Arrow file** to remove duplicates

## Generation Scripts

- **`generate_large_test_data.py`** - Existing script for large dataset generation
- **`generate_large_dataset.py`** - New script for controlled duplicate generation

## Testing Checklist

- [ ] Basic duplicate detection works
- [ ] Mixed data types handled correctly
- [ ] Null values processed properly
- [ ] Large datasets perform well
- [ ] UI integration functions correctly
- [ ] Clean Arrow file creation works
- [ ] Error handling works for edge cases

## File Sizes and Records

| File | Size | Records | Purpose |
|------|------|---------|---------|
| simple_duplicates.csv | ~500B | 13 | Basic testing |
| mixed_data_types.csv | ~1KB | 20 | Data type testing |
| with_nulls.csv | ~1.3KB | 26 | Null handling |
| test_complex.csv | ~3.5KB | ~100 | Medium complexity |
| test_medium_complex.csv | ~14KB | ~500 | Medium dataset |
| test_medium_complex_full.csv | ~400KB | ~10k | Full medium dataset |
| test_large_complex.csv | ~30MB | ~100k | Performance testing |

## Notes

- All test files use `group_id` as the primary grouping column
- Duplicate patterns are controlled and predictable
- Files are designed to test different aspects of the duplicate detection algorithm
- Large files are suitable for performance benchmarking 