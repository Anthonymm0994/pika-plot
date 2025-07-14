//! Grid view for displaying tabular data.

use egui::{Ui, ScrollArea};
use std::sync::Arc;

/// Grid view panel for displaying tabular data.
pub struct GridView {
    max_rows: usize,
    max_cols: usize,
}

impl GridView {
    pub fn new() -> Self {
        Self {
            max_rows: 100,
            max_cols: 50,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, data: Option<Arc<Vec<Vec<String>>>>) {
        ScrollArea::both()
            .id_source("grid_view")
            .show(ui, |ui| {
                if let Some(data) = data {
                    // Simple placeholder table rendering
                    egui::Grid::new("data_grid")
                        .num_columns(data.get(0).map_or(0, |row| row.len()))
                        .striped(true)
                        .show(ui, |ui| {
                            // Render headers if available
                            if let Some(first_row) = data.get(0) {
                                for header in first_row {
                                    ui.strong(header);
                                }
                                ui.end_row();
                            }
                            
                            // Render data rows
                            for (_row_idx, row) in data.iter().skip(1).take(self.max_rows).enumerate() {
                                for (_col_idx, cell) in row.iter().take(self.max_cols).enumerate() {
                                    ui.label(cell);
                                }
                                ui.end_row();
                            }
                        });
                } else {
                    ui.label("No data to display");
                }
            });
    }
}

// Original arrow-based implementation commented out for now
/*
impl GridView {
    pub fn show_record_batch(&mut self, ui: &mut Ui, batch: Option<&RecordBatch>) {
        ScrollArea::both()
            .id_salt("grid_view")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if let Some(batch) = batch {
                    let schema = batch.schema();
                    let num_rows = batch.num_rows().min(self.max_rows);
                    let num_cols = batch.num_columns().min(self.max_cols);
                    
                    egui::Grid::new("data_grid")
                        .num_columns(num_cols)
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            for i in 0..num_cols {
                                ui.strong(schema.field(i).name());
                            }
                            ui.end_row();
                            
                            // Data rows
                            for row_idx in 0..num_rows {
                                for col_idx in 0..num_cols {
                                    let column = batch.column(col_idx);
                                    let value = self.format_cell_value(column, row_idx);
                                    ui.label(value);
                                }
                                ui.end_row();
                            }
                        });
                } else {
                    ui.label("No data to display");
                }
            });
    }
    
    fn format_cell_value(&self, column: &Arc<dyn arrow::array::Array>, row_idx: usize) -> String {
        if column.is_null(row_idx) {
            return "NULL".to_string();
        }
        
        // Format based on data type
        if let Some(string_array) = column.as_any().downcast_ref::<arrow::array::StringArray>() {
            string_array.value(row_idx).to_string()
        } else if let Some(float_array) = column.as_any().downcast_ref::<arrow::array::Float64Array>() {
            format!("{:.2}", float_array.value(row_idx))
        } else if let Some(int_array) = column.as_any().downcast_ref::<arrow::array::Int64Array>() {
            int_array.value(row_idx).to_string()
        } else {
            "?".to_string()
        }
    }
}
*/ 