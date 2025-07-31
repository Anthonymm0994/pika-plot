import pyarrow as pa
import pyarrow.feather as feather
import pandas as pd
import numpy as np

print("Creating simple Arrow files for JavaScript testing...")

# Create simple test data
data = {
    'x': np.random.randn(100),
    'y': np.random.randn(100),
    'category': np.random.choice(['A', 'B', 'C'], 100),
    'value': np.random.randint(1, 100, 100)
}

df = pd.DataFrame(data)
print(f"Created DataFrame: {df.shape}")

# Create Arrow table
table = pa.Table.from_pandas(df)
print(f"Created Arrow table: {table.num_rows} rows, {table.num_columns} columns")

# Save as uncompressed Arrow file using RecordBatchFileWriter
with pa.ipc.RecordBatchFileWriter('simple_test.arrow', table.schema) as writer:
    writer.write_table(table)

print("✅ Saved simple_test.arrow (uncompressed)")

# Create time series data
dates = pd.date_range('2023-01-01', periods=50, freq='D')
ts_data = {
    'date': dates,
    'value': np.random.randn(50).cumsum(),
    'volume': np.random.randint(100, 1000, 50)
}

ts_df = pd.DataFrame(ts_data)
ts_table = pa.Table.from_pandas(ts_df)

with pa.ipc.RecordBatchFileWriter('simple_timeseries.arrow', ts_table.schema) as writer:
    writer.write_table(ts_table)

print("✅ Saved simple_timeseries.arrow (uncompressed)")

print("\n📋 Files created:")
print("- simple_test.arrow (100 rows, 4 columns)")
print("- simple_timeseries.arrow (50 rows, 3 columns)")
print("\nThese files should work with the JavaScript Arrow library.") 