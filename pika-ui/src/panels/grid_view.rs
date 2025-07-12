use egui::{Ui, Color32, ScrollArea, Vec2};
use arrow::record_batch::RecordBatch;
use arrow::array::Array;
use pika_engine::plot::{extract_numeric_values, extract_string_values};

/// Grid view for displaying data in a table format
pub struct GridView {
    /// Maximum number of rows to display
    max_rows: usize,
    /// Maximum number of columns to display
    max_cols: usize,
    /// Current page
    current_page: usize,
    /// Rows per page
    rows_per_page: usize,
    /// Selected cells
    selected_cells: Vec<(usize, usize)>,
}

impl GridView {
    pub fn new() -> Self {
        Self {
            max_rows: 1000,
            max_cols: 50,
            current_page: 0,
            rows_per_page: 100,
            selected_cells: Vec::new(),
        }
    }
    
    /// Render the grid view
    pub fn render(&mut self, ui: &mut Ui, data: &RecordBatch) {
        let schema = data.schema();
        let num_rows = data.num_rows();
        let num_cols = data.num_columns().min(self.max_cols);
        
        // Pagination controls
        ui.horizontal(|ui| {
            ui.label(format!("Rows: {} | Columns: {}", num_rows, data.num_columns()));
            
            ui.separator();
            
            let total_pages = (num_rows + self.rows_per_page - 1) / self.rows_per_page;
            
            if ui.button("◀").clicked() && self.current_page > 0 {
                self.current_page -= 1;
            }
            
            ui.label(format!("Page {} of {}", self.current_page + 1, total_pages));
            
            if ui.button("▶").clicked() && self.current_page < total_pages - 1 {
                self.current_page += 1;
            }
            
            ui.separator();
            
            ui.label("Rows per page:");
            ui.add(egui::DragValue::new(&mut self.rows_per_page)
                .speed(10)
                .clamp_range(10..=500));
        });
        
        ui.separator();
        
        // Calculate visible row range
        let start_row = self.current_page * self.rows_per_page;
        let end_row = (start_row + self.rows_per_page).min(num_rows);
        
        // Grid rendering
        ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("data_grid")
                    .striped(true)
                    .min_col_width(100.0)
                    .max_col_width(300.0)
                    .show(ui, |ui| {
                        // Header row
                        ui.label(""); // Row number column
                        for field in schema.fields().iter().take(num_cols) {
                            ui.label(egui::RichText::new(field.name())
                                .strong()
                                .color(Color32::from_rgb(150, 150, 255)));
                        }
                        ui.end_row();
                        
                        // Data rows
                        for row_idx in start_row..end_row {
                            // Row number
                            ui.label(egui::RichText::new(format!("{}", row_idx + 1))
                                .color(Color32::from_gray(150)));
                            
                            // Cell values
                            for col_idx in 0..num_cols {
                                let column = data.column(col_idx);
                                let cell_value = self.get_cell_value(column, row_idx);
                                
                                let is_selected = self.selected_cells.contains(&(row_idx, col_idx));
                                
                                let response = ui.selectable_label(is_selected, cell_value);
                                
                                if response.clicked() {
                                    if ui.input(|i| i.modifiers.ctrl) {
                                        // Multi-select with Ctrl
                                        if is_selected {
                                            self.selected_cells.retain(|&(r, c)| r != row_idx || c != col_idx);
                                        } else {
                                            self.selected_cells.push((row_idx, col_idx));
                                        }
                                    } else {
                                        // Single select
                                        self.selected_cells.clear();
                                        self.selected_cells.push((row_idx, col_idx));
                                    }
                                }
                                
                                // Show tooltip with full value on hover
                                response.on_hover_text(&self.get_cell_value(column, row_idx));
                            }
                            
                            ui.end_row();
                        }
                    });
            });
        
        // Selection info
        if !self.selected_cells.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Selected: {} cells", self.selected_cells.len()));
                
                if ui.button("Copy").clicked() {
                    self.copy_selection(data);
                }
                
                if ui.button("Clear Selection").clicked() {
                    self.selected_cells.clear();
                }
            });
        }
    }
    
    /// Get the string representation of a cell value
    fn get_cell_value(&self, column: &dyn Array, row_idx: usize) -> String {
        if column.is_null(row_idx) {
            return "null".to_string();
        }
        
        // Try different array types
        if let Ok(values) = extract_string_values(&column.to_data()) {
            if let Some(value) = values.get(row_idx) {
                return value.clone();
            }
        }
        
        if let Ok(values) = extract_numeric_values(&column.to_data()) {
            if let Some(value) = values.get(row_idx) {
                if value.fract() == 0.0 && value.abs() < 1e10 {
                    return format!("{:.0}", value);
                } else {
                    return format!("{:.4}", value);
                }
            }
        }
        
        // Fallback to arrow's display
        arrow::util::display::array_value_to_string(column, row_idx)
            .unwrap_or_else(|_| "error".to_string())
    }
    
    /// Copy selected cells to clipboard
    fn copy_selection(&self, data: &RecordBatch) {
        if self.selected_cells.is_empty() {
            return;
        }
        
        // Sort selected cells by row then column
        let mut sorted_cells = self.selected_cells.clone();
        sorted_cells.sort_by_key(|&(r, c)| (r, c));
        
        // Build clipboard text
        let mut clipboard_text = String::new();
        let mut current_row = sorted_cells[0].0;
        
        for &(row, col) in &sorted_cells {
            if row != current_row {
                clipboard_text.push('\n');
                current_row = row;
            } else if !clipboard_text.is_empty() && !clipboard_text.ends_with('\n') {
                clipboard_text.push('\t');
            }
            
            let column = data.column(col);
            clipboard_text.push_str(&self.get_cell_value(column, row));
        }
        
        // Copy to clipboard
        ui.output_mut(|o| o.copied_text = clipboard_text);
    }
}

/// Grid view panel that can be shown in the UI
pub struct GridViewPanel {
    grid_view: GridView,
    current_data: Option<RecordBatch>,
}

impl GridViewPanel {
    pub fn new() -> Self {
        Self {
            grid_view: GridView::new(),
            current_data: None,
        }
    }
    
    pub fn set_data(&mut self, data: RecordBatch) {
        self.current_data = Some(data);
        self.grid_view.current_page = 0;
        self.grid_view.selected_cells.clear();
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(data) = &self.current_data {
            self.grid_view.render(ui, data);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No data to display");
                ui.label(egui::RichText::new("Load data to view in grid format").weak());
            });
        }
    }
} 