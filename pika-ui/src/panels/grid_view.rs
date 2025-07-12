use egui::{Ui, ScrollArea};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

pub struct GridView {
    rows_per_page: usize,
    current_page: usize,
}

pub struct GridViewPanel {
    grid_view: GridView,
}

impl GridViewPanel {
    pub fn new() -> Self {
        Self {
            grid_view: GridView::new(),
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, batch: Option<&RecordBatch>) {
        if let Some(batch) = batch {
            self.grid_view.show(ui, batch);
        } else {
            ui.label("No data to display");
        }
    }
}

impl GridView {
    pub fn new() -> Self {
        Self {
            rows_per_page: 100,
            current_page: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, batch: &RecordBatch) {
        ui.heading("Data Grid");
        ui.separator();
        
        // Display column headers
        ui.horizontal(|ui| {
            for field in batch.schema().fields() {
                ui.label(field.name());
            }
        });
        
        ui.separator();
        
        // Display data in scrollable area
        ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                self.show_data_rows(ui, batch);
            });
        
        ui.separator();
        
        // Grid controls
        ui.horizontal(|ui| {
            ui.label("Rows per page:");
            ui.add(egui::DragValue::new(&mut self.rows_per_page)
                .range(10..=500));
                
            if ui.button("Export").clicked() {
                // Export functionality would go here
            }
        });
    }
    
    fn show_data_rows(&mut self, ui: &mut Ui, batch: &RecordBatch) {
        let max_rows = std::cmp::min(self.rows_per_page, batch.num_rows());
        
        for row_idx in 0..max_rows {
            ui.horizontal(|ui| {
                ui.label(format!("Row {}", row_idx));
                
                // Display values for each column
                for col_idx in 0..batch.num_columns() {
                    let column = batch.column(col_idx);
                    let value_str = self.extract_cell_value(column, row_idx);
                    ui.label(value_str);
                }
            });
        }
    }
    
    fn extract_cell_value(&self, column: &Arc<dyn arrow::array::Array>, row_idx: usize) -> String {
        // Check bounds first
        if row_idx >= column.len() {
            return "N/A".to_string();
        }
        
        // Try to extract as string first
        if let Some(string_array) = column.as_any().downcast_ref::<arrow::array::StringArray>() {
            return string_array.value(row_idx).to_string();
        }
        
        // Try to extract as float64
        if let Some(float_array) = column.as_any().downcast_ref::<arrow::array::Float64Array>() {
            return format!("{:.2}", float_array.value(row_idx));
        }
        
        // Try to extract as int64
        if let Some(int_array) = column.as_any().downcast_ref::<arrow::array::Int64Array>() {
            return int_array.value(row_idx).to_string();
        }
        
        // Fallback
        "N/A".to_string()
    }
}

impl Default for GridView {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GridViewPanel {
    fn default() -> Self {
        Self::new()
    }
} 