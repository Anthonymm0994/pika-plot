# Test Data Directory

This directory contains comprehensive test datasets and tools for testing the CSV import functionality.

## Test Datasets

### Small Test Dataset (`test_complex.csv`)
- **Size**: ~50 rows
- **Purpose**: Quick testing and development
- **Use**: Set header row to 5 for testing

### Medium Test Dataset (`test_medium_complex_full.csv`)
- **Size**: ~10,000 rows (~293KB)
- **Purpose**: Comprehensive testing of performance and edge cases
- **Use**: Set header row to 5 for testing

### Large Test Dataset Template (`test_large_complex.csv`)
- **Size**: Template only (24 lines)
- **Purpose**: Template for generating 100k row dataset
- **Use**: Run generation script to create full dataset

## Generation Scripts

### `generate_large_test_data.py`
- **Purpose**: Generate 100k row comprehensive test dataset
- **Requirements**: Python 3.6+ with numpy
- **Usage**: `python generate_large_test_data.py`

### `generate_medium_dataset.sh`
- **Purpose**: Generate medium test dataset (10k rows)
- **Requirements**: Bash with bc command
- **Usage**: `./generate_medium_dataset.sh`

## Documentation

- `TEST_DATASET_DOCUMENTATION.md` - Comprehensive dataset specifications
- `COMPREHENSIVE_TESTING_SUMMARY.md` - Complete testing infrastructure overview
- `VALIDATION_SUMMARY.md` - Validation results and findings

## Quick Start

1. **For quick testing**: Use `test_complex.csv`
2. **For comprehensive testing**: Use `test_medium_complex_full.csv`
3. **For performance testing**: Generate large dataset with `generate_large_test_data.py`

## Expected Results

All datasets should:
- Import successfully with header row set to 5
- Infer correct data types for all 32 columns
- Handle null values properly
- Filter out garbage lines
- Complete import in reasonable time (< 30 seconds for medium dataset) 