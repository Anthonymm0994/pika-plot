#!/usr/bin/env python3
"""
Script to create a test Arrow file for the JavaScript Arrow Data Explorer
"""

import pyarrow as pa
import pyarrow.feather as feather
import pandas as pd
import numpy as np
from datetime import datetime, timedelta

def create_test_arrow_file():
    """Create a test Arrow file with various data types"""
    
    # Create sample data
    np.random.seed(42)
    n_rows = 100
    
    # Generate timestamps
    base_time = datetime(2024, 1, 1, 0, 0, 0)
    timestamps = [base_time + timedelta(hours=i) for i in range(n_rows)]
    
    # Create DataFrame with various data types
    data = {
        'id': list(range(1, n_rows + 1)),
        'timestamp': timestamps,
        'value': np.random.randn(n_rows) * 100 + 500,
        'category': np.random.choice(['A', 'B', 'C', 'D'], n_rows),
        'temperature': np.random.uniform(15, 35, n_rows),
        'humidity': np.random.uniform(30, 90, n_rows),
        'is_active': np.random.choice([True, False], n_rows),
        'score': np.random.randint(0, 100, n_rows)
    }
    
    df = pd.DataFrame(data)
    
    # Convert to Arrow table
    table = pa.Table.from_pandas(df)
    
    # Save as Arrow file
    output_file = 'test_data.arrow'
    feather.write_feather(table, output_file)
    
    print(f"Created test Arrow file: {output_file}")
    print(f"Data shape: {table.num_rows} rows, {table.num_columns} columns")
    print(f"Schema: {table.schema}")
    
    return output_file

def create_time_series_arrow_file():
    """Create a time series Arrow file"""
    
    np.random.seed(42)
    n_rows = 200
    
    # Generate time series data
    base_time = datetime(2024, 1, 1, 0, 0, 0)
    timestamps = [base_time + timedelta(minutes=i*15) for i in range(n_rows)]
    
    # Create time series with some patterns
    time_values = np.arange(n_rows)
    sine_wave = 50 * np.sin(time_values * 0.1) + 100
    trend = time_values * 0.5 + 200
    noise = np.random.randn(n_rows) * 10
    
    data = {
        'timestamp': timestamps,
        'value': sine_wave + trend + noise,
        'temperature': 20 + 10 * np.sin(time_values * 0.05) + np.random.randn(n_rows) * 2,
        'humidity': 60 + 20 * np.cos(time_values * 0.03) + np.random.randn(n_rows) * 5,
        'pressure': 1013 + np.random.randn(n_rows) * 10,
        'wind_speed': np.random.exponential(5, n_rows)
    }
    
    df = pd.DataFrame(data)
    table = pa.Table.from_pandas(df)
    
    output_file = 'time_series_data.arrow'
    feather.write_feather(table, output_file)
    
    print(f"Created time series Arrow file: {output_file}")
    print(f"Data shape: {table.num_rows} rows, {table.num_columns} columns")
    
    return output_file

if __name__ == "__main__":
    print("Creating test Arrow files...")
    
    # Create basic test file
    test_file = create_test_arrow_file()
    
    # Create time series file
    time_series_file = create_time_series_arrow_file()
    
    print("\nTest files created successfully!")
    print("You can now use these files to test the JavaScript Arrow Data Explorer.") 