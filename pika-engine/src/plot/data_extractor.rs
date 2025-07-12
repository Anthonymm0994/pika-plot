use arrow::array::{
    Array, ArrayRef, Float64Array, Float32Array, Int64Array, Int32Array, 
    StringArray, BooleanArray, TimestampMillisecondArray,
};
use arrow::record_batch::RecordBatch;
use pika_core::{PikaError, Result};
use std::collections::HashMap;

/// Extracts numeric values from an Arrow array
pub fn extract_numeric_values(array: &ArrayRef) -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(array.len());
    
    match array.data_type() {
        arrow::datatypes::DataType::Float64 => {
            let float_array = array.as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| PikaError::Internal("Failed to cast to Float64Array".to_string()))?;
            
            for i in 0..float_array.len() {
                if float_array.is_null(i) {
                    values.push(f64::NAN);
                } else {
                    values.push(float_array.value(i));
                }
            }
        }
        arrow::datatypes::DataType::Float32 => {
            let float_array = array.as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| PikaError::Internal("Failed to cast to Float32Array".to_string()))?;
            
            for i in 0..float_array.len() {
                if float_array.is_null(i) {
                    values.push(f64::NAN);
                } else {
                    values.push(float_array.value(i) as f64);
                }
            }
        }
        arrow::datatypes::DataType::Int64 => {
            let int_array = array.as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| PikaError::Internal("Failed to cast to Int64Array".to_string()))?;
            
            for i in 0..int_array.len() {
                if int_array.is_null(i) {
                    values.push(f64::NAN);
                } else {
                    values.push(int_array.value(i) as f64);
                }
            }
        }
        arrow::datatypes::DataType::Int32 => {
            let int_array = array.as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| PikaError::Internal("Failed to cast to Int32Array".to_string()))?;
            
            for i in 0..int_array.len() {
                if int_array.is_null(i) {
                    values.push(f64::NAN);
                } else {
                    values.push(int_array.value(i) as f64);
                }
            }
        }
        _ => return Err(PikaError::Internal(
            format!("Column has unsupported numeric type: {:?}", array.data_type())
        )),
    }
    
    Ok(values)
}

/// Extracts string values from an Arrow array
pub fn extract_string_values(array: &ArrayRef) -> Result<Vec<String>> {
    let string_array = array.as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| PikaError::Internal("Failed to cast to StringArray".to_string()))?;
    
    let mut values = Vec::with_capacity(string_array.len());
    for i in 0..string_array.len() {
        if string_array.is_null(i) {
            values.push("null".to_string());
        } else {
            values.push(string_array.value(i).to_string());
        }
    }
    
    Ok(values)
}

/// Extracts timestamp values as f64 (milliseconds since epoch)
pub fn extract_timestamp_values(array: &ArrayRef) -> Result<Vec<f64>> {
    let timestamp_array = array.as_any()
        .downcast_ref::<TimestampMillisecondArray>()
        .ok_or_else(|| PikaError::Internal("Failed to cast to TimestampArray".to_string()))?;
    
    let mut values = Vec::with_capacity(timestamp_array.len());
    for i in 0..timestamp_array.len() {
        if timestamp_array.is_null(i) {
            values.push(f64::NAN);
        } else {
            values.push(timestamp_array.value(i) as f64);
        }
    }
    
    Ok(values)
}

/// Extracts x,y point pairs from a RecordBatch
pub fn extract_xy_points(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
) -> Result<Vec<(f64, f64)>> {
    let x_array = batch.column_by_name(x_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", x_column)))?;
    
    let y_array = batch.column_by_name(y_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", y_column)))?;
    
    let x_values = extract_numeric_values(x_array)?;
    let y_values = extract_numeric_values(y_array)?;
    
    if x_values.len() != y_values.len() {
        return Err(PikaError::Internal(
            format!("Column length mismatch: {} vs {}", x_values.len(), y_values.len())
        ));
    }
    
    let points: Vec<(f64, f64)> = x_values.into_iter()
        .zip(y_values.into_iter())
        .filter(|(x, y)| !x.is_nan() && !y.is_nan())
        .collect();
    
    Ok(points)
}

/// Extracts x,y,z point triplets from a RecordBatch
pub fn extract_xyz_points(
    batch: &RecordBatch,
    x_column: &str,
    y_column: &str,
    z_column: &str,
) -> Result<Vec<(f64, f64, f64)>> {
    let x_array = batch.column_by_name(x_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", x_column)))?;
    
    let y_array = batch.column_by_name(y_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", y_column)))?;
    
    let z_array = batch.column_by_name(z_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", z_column)))?;
    
    let x_values = extract_numeric_values(x_array)?;
    let y_values = extract_numeric_values(y_array)?;
    let z_values = extract_numeric_values(z_array)?;
    
    if x_values.len() != y_values.len() || x_values.len() != z_values.len() {
        return Err(PikaError::Internal("Column length mismatch".to_string()));
    }
    
    let points: Vec<(f64, f64, f64)> = x_values.into_iter()
        .zip(y_values.into_iter())
        .zip(z_values.into_iter())
        .filter(|((x, y), z)| !x.is_nan() && !y.is_nan() && !z.is_nan())
        .map(|((x, y), z)| (x, y, z))
        .collect();
    
    Ok(points)
}

/// Extracts category-value pairs from a RecordBatch
pub fn extract_category_values(
    batch: &RecordBatch,
    category_column: &str,
    value_column: &str,
) -> Result<Vec<(String, f64)>> {
    let cat_array = batch.column_by_name(category_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", category_column)))?;
    
    let val_array = batch.column_by_name(value_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", value_column)))?;
    
    let categories = extract_string_values(cat_array)?;
    let values = extract_numeric_values(val_array)?;
    
    if categories.len() != values.len() {
        return Err(PikaError::Internal("Column length mismatch".to_string()));
    }
    
    let pairs: Vec<(String, f64)> = categories.into_iter()
        .zip(values.into_iter())
        .filter(|(_, v)| !v.is_nan())
        .collect();
    
    Ok(pairs)
}

/// Aggregates category-value pairs by summing values for each category
pub fn aggregate_by_category(pairs: Vec<(String, f64)>) -> HashMap<String, f64> {
    let mut aggregated = HashMap::new();
    
    for (category, value) in pairs {
        *aggregated.entry(category).or_insert(0.0) += value;
    }
    
    aggregated
}

/// Extracts time series data from a RecordBatch
pub fn extract_time_series(
    batch: &RecordBatch,
    time_column: &str,
    value_columns: &[String],
) -> Result<HashMap<String, Vec<(f64, f64)>>> {
    let time_array = batch.column_by_name(time_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", time_column)))?;
    
    let time_values = match time_array.data_type() {
        arrow::datatypes::DataType::Timestamp(_, _) => extract_timestamp_values(time_array)?,
        _ => extract_numeric_values(time_array)?,
    };
    
    let mut series_data = HashMap::new();
    
    for value_col in value_columns {
        let val_array = batch.column_by_name(value_col)
            .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", value_col)))?;
        
        let values = extract_numeric_values(val_array)?;
        
        if time_values.len() != values.len() {
            return Err(PikaError::Internal("Column length mismatch".to_string()));
        }
        
        let points: Vec<(f64, f64)> = time_values.iter()
            .zip(values.iter())
            .filter(|(t, v)| !t.is_nan() && !v.is_nan())
            .map(|(&t, &v)| (t, v))
            .collect();
        
        series_data.insert(value_col.clone(), points);
    }
    
    Ok(series_data)
}

/// Extracts OHLC (Open, High, Low, Close) data for candlestick charts
pub fn extract_ohlc_data(
    batch: &RecordBatch,
    time_column: &str,
    open_column: &str,
    high_column: &str,
    low_column: &str,
    close_column: &str,
) -> Result<Vec<(f64, f64, f64, f64, f64)>> {
    let time_array = batch.column_by_name(time_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", time_column)))?;
    let open_array = batch.column_by_name(open_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", open_column)))?;
    let high_array = batch.column_by_name(high_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", high_column)))?;
    let low_array = batch.column_by_name(low_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", low_column)))?;
    let close_array = batch.column_by_name(close_column)
        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", close_column)))?;
    
    let time_values = extract_timestamp_values(time_array)?;
    let open_values = extract_numeric_values(open_array)?;
    let high_values = extract_numeric_values(high_array)?;
    let low_values = extract_numeric_values(low_array)?;
    let close_values = extract_numeric_values(close_array)?;
    
    let len = time_values.len();
    if open_values.len() != len || high_values.len() != len || 
       low_values.len() != len || close_values.len() != len {
        return Err(PikaError::Internal("Column length mismatch".to_string()));
    }
    
    let ohlc_data: Vec<(f64, f64, f64, f64, f64)> = (0..len)
        .filter(|&i| {
            !time_values[i].is_nan() && !open_values[i].is_nan() && 
            !high_values[i].is_nan() && !low_values[i].is_nan() && 
            !close_values[i].is_nan()
        })
        .map(|i| (time_values[i], open_values[i], high_values[i], low_values[i], close_values[i]))
        .collect();
    
    Ok(ohlc_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn create_test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("category", DataType::Utf8, false),
            Field::new("value", DataType::Int32, false),
        ]));

        let x_array = Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let y_array = Float64Array::from(vec![2.0, 4.0, 6.0, 8.0, 10.0]);
        let category_array = StringArray::from(vec!["A", "B", "A", "B", "A"]);
        let value_array = Int32Array::from(vec![10, 20, 30, 40, 50]);

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

    #[test]
    fn test_extract_xy_points() {
        let batch = create_test_batch();
        let points = extract_xy_points(&batch, "x", "y").unwrap();
        
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], (1.0, 2.0));
        assert_eq!(points[4], (5.0, 10.0));
    }

    #[test]
    fn test_extract_category_values() {
        let batch = create_test_batch();
        let pairs = extract_category_values(&batch, "category", "value").unwrap();
        
        assert_eq!(pairs.len(), 5);
        assert_eq!(pairs[0], ("A".to_string(), 10.0));
        assert_eq!(pairs[1], ("B".to_string(), 20.0));
    }

    #[test]
    fn test_aggregate_by_category() {
        let pairs = vec![
            ("A".to_string(), 10.0),
            ("B".to_string(), 20.0),
            ("A".to_string(), 30.0),
            ("B".to_string(), 40.0),
            ("A".to_string(), 50.0),
        ];
        
        let aggregated = aggregate_by_category(pairs);
        
        assert_eq!(aggregated.get("A"), Some(&90.0));
        assert_eq!(aggregated.get("B"), Some(&60.0));
    }
} 