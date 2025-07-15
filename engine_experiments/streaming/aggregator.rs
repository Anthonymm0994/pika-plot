use std::collections::HashMap;
use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use crate::{Result, PikaError};

/// Aggregation function types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregateFunction {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

/// Aggregation state for a single column
#[derive(Debug, Clone)]
struct AggregateState {
    function: AggregateFunction,
    sum: f64,
    count: i64,
    min: f64,
    max: f64,
}

impl AggregateState {
    fn new(function: AggregateFunction) -> Self {
        Self {
            function,
            sum: 0.0,
            count: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    fn update(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn result(&self) -> f64 {
        match self.function {
            AggregateFunction::Sum => self.sum,
            AggregateFunction::Avg => {
                if self.count > 0 {
                    self.sum / self.count as f64
                } else {
                    0.0
                }
            }
            AggregateFunction::Min => {
                if self.count > 0 {
                    self.min
                } else {
                    0.0
                }
            }
            AggregateFunction::Max => {
                if self.count > 0 {
                    self.max
                } else {
                    0.0
                }
            }
            AggregateFunction::Count => self.count as f64,
        }
    }
}

/// Streaming aggregator for real-time data aggregation
pub struct StreamingAggregator {
    group_by: Option<String>,
    aggregates: HashMap<String, (String, AggregateFunction)>, // column -> (alias, function)
    states: HashMap<String, HashMap<String, AggregateState>>, // group -> column -> state
}

impl StreamingAggregator {
    pub fn new() -> Self {
        Self {
            group_by: None,
            aggregates: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// Set the group by column
    pub fn group_by(&mut self, column: String) -> &mut Self {
        self.group_by = Some(column);
        self
    }

    /// Add an aggregate function
    pub fn aggregate(&mut self, column: String, alias: String, function: AggregateFunction) -> &mut Self {
        self.aggregates.insert(column, (alias, function));
        self
    }

    /// Process a batch and update aggregation state
    pub fn process_batch(&mut self, batch: &RecordBatch) -> Result<()> {
        let schema = batch.schema();
        
        // Get group by column if specified
        let groups = if let Some(ref group_col) = self.group_by {
            let col_idx = schema.index_of(group_col)
                .map_err(|_| PikaError::Internal(format!("Group by column {} not found", group_col)))?;
            
            let array = batch.column(col_idx);
            match array.data_type() {
                DataType::Utf8 => {
                    let string_array = array.as_any().downcast_ref::<StringArray>()
                        .ok_or_else(|| PikaError::Internal("Failed to cast to StringArray".to_string()))?;
                    
                    (0..string_array.len())
                        .map(|i| string_array.value(i).to_string())
                        .collect::<Vec<_>>()
                }
                _ => return Err(PikaError::Internal("Group by column must be string type".to_string())),
            }
        } else {
            vec!["__all__".to_string(); batch.num_rows()]
        };

        // Process each aggregate column
        for (col_name, (_, function)) in &self.aggregates {
            let col_idx = schema.index_of(col_name)
                .map_err(|_| PikaError::Internal(format!("Aggregate column {} not found", col_name)))?;
            
            let array = batch.column(col_idx);
            let values = match array.data_type() {
                DataType::Float64 => {
                    let float_array = array.as_any().downcast_ref::<Float64Array>()
                        .ok_or_else(|| PikaError::Internal("Failed to cast to Float64Array".to_string()))?;
                    (0..float_array.len()).map(|i| float_array.value(i)).collect::<Vec<_>>()
                }
                DataType::Int32 | DataType::Int64 => {
                    let int_array = array.as_any().downcast_ref::<Int64Array>()
                        .ok_or_else(|| PikaError::Internal("Failed to cast to Int64Array".to_string()))?;
                    (0..int_array.len()).map(|i| int_array.value(i) as f64).collect::<Vec<_>>()
                }
                _ => return Err(PikaError::Internal("Aggregate column must be numeric type".to_string())),
            };

            // Update states
            for (i, group) in groups.iter().enumerate() {
                let group_states = self.states.entry(group.clone()).or_insert_with(HashMap::new);
                let state = group_states.entry(col_name.clone())
                    .or_insert_with(|| AggregateState::new(*function));
                state.update(values[i]);
            }
        }

        Ok(())
    }

    /// Get the current aggregation results as a RecordBatch
    pub fn get_results(&self) -> Result<RecordBatch> {
        if self.states.is_empty() {
            return Err(PikaError::Internal("No data to aggregate".to_string()));
        }

        let mut fields = vec![];
        let mut arrays: Vec<ArrayRef> = vec![];

        // Add group by column if present
        if let Some(ref group_col) = self.group_by {
            fields.push(Field::new(group_col, DataType::Utf8, false));
            let groups: Vec<String> = self.states.keys().cloned().collect();
            arrays.push(Arc::new(StringArray::from(groups)) as ArrayRef);
        }

        // Add aggregate columns
        for (col_name, (alias, _)) in &self.aggregates {
            fields.push(Field::new(alias, DataType::Float64, false));
            
            let mut values = vec![];
            for group in self.states.keys() {
                if let Some(group_states) = self.states.get(group) {
                    if let Some(state) = group_states.get(col_name) {
                        values.push(state.result());
                    } else {
                        values.push(0.0);
                    }
                }
            }
            
            arrays.push(Arc::new(Float64Array::from(values)) as ArrayRef);
        }

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, arrays)
            .map_err(|e| PikaError::Internal(format!("Failed to create result batch: {}", e)))
    }

    /// Clear the aggregation state
    pub fn clear(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;

    fn create_test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("category", DataType::Utf8, false),
            Field::new("value", DataType::Int32, false),
        ]));

        let categories = StringArray::from(vec!["A", "B", "A", "B", "A"]);
        let values = Int32Array::from(vec![10, 20, 30, 40, 50]);

        RecordBatch::try_new(
            schema,
            vec![Arc::new(categories), Arc::new(values)],
        ).unwrap()
    }

    #[test]
    fn test_aggregation() {
        let mut aggregator = StreamingAggregator::new();
        aggregator
            .group_by("category".to_string())
            .aggregate("value".to_string(), "sum_value".to_string(), AggregateFunction::Sum)
            .aggregate("value".to_string(), "avg_value".to_string(), AggregateFunction::Avg);

        let batch = create_test_batch();
        aggregator.process_batch(&batch).unwrap();

        let results = aggregator.get_results().unwrap();
        assert_eq!(results.num_rows(), 2); // Two groups: A and B
        assert_eq!(results.num_columns(), 3); // group, sum, avg
    }
} 