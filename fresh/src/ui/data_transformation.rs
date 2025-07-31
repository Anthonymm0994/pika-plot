use egui;
use datafusion::arrow::array::{ArrayRef, StringArray, Int64Array, Float64Array, BooleanArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::core::{Database, TableInfo, DataTransformer, TransformationType, TransformationConfig};
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TransformationDialog {
    pub visible: bool,
    pub selected_table: Option<String>,
    pub transformation_type: Option<TransformationType>,
    pub selected_columns: Vec<String>,
    pub output_column_name: String,
    pub available_tables: Vec<TableInfo>,
    pub available_columns: Vec<String>,
    pub bin_size: String,
    pub time_column: Option<String>,
    pub grouping_columns: Vec<String>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}

impl Default for TransformationDialog {
    fn default() -> Self {
        Self {
            visible: false,
            selected_table: None,
            transformation_type: None,
            selected_columns: Vec::new(),
            output_column_name: String::new(),
            available_tables: Vec::new(),
            available_columns: Vec::new(),
            bin_size: "1".to_string(),
            time_column: None,
            grouping_columns: Vec::new(),
            error_message: None,
            success_message: None,
        }
    }
}

impl TransformationDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ctx: &egui::Context, database: &Database) -> Option<TransformationRequest> {
        if !self.visible {
            return None;
        }

        let mut result = None;
        let mut table_selected = false;
        let mut selected_table_name = None;
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let mut transform_type_changed = false;
        let mut new_transform_type = None;

        // Local copies of all user-editable state
        let mut selected_columns = self.selected_columns.clone();
        let mut output_column_name = self.output_column_name.clone();
        let mut time_column = self.time_column.clone();
        let mut grouping_columns = self.grouping_columns.clone();
        let mut bin_size = self.bin_size.clone();

        egui::Window::new("Data Transformation")
            .open(&mut self.visible)
            .resizable(true)
            .default_size([500.0, 600.0])
            .show(ctx, |ui| {
                ui.heading("Data Transformation");
                ui.separator();

                // Table selection
                ui.label("Select Table:");
                egui::ComboBox::from_id_source("table_select")
                    .selected_text(self.selected_table.as_deref().unwrap_or("Select a table"))
                    .show_ui(ui, |ui| {
                        for table in &self.available_tables {
                            if ui.selectable_label(
                                self.selected_table.as_deref() == Some(&table.name),
                                &table.name,
                            ).clicked() {
                                selected_table_name = Some(table.name.clone());
                                table_selected = true;
                            }
                        }
                    });

                if let Some(_table_name) = &self.selected_table {
                    ui.separator();
                    ui.label("Transformation Type:");
                    let current_transform_type = self.transformation_type.clone();
                    egui::ComboBox::from_id_source("transform_type")
                        .selected_text(match &current_transform_type {
                            Some(TransformationType::Delta) => "Delta (Single Column)",
                            Some(TransformationType::DeltaMultiple) => "Delta (Multiple Columns)",
                            Some(TransformationType::TimeBin) => "Time Bin Column",
                            Some(TransformationType::RowId) => "Row ID Columns",
                            None => "Select transformation type",
                        })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(
                                current_transform_type.as_ref() == Some(&TransformationType::Delta),
                                "Delta (Single Column)",
                            ).clicked() {
                                new_transform_type = Some(TransformationType::Delta);
                                transform_type_changed = true;
                            }
                            if ui.selectable_label(
                                current_transform_type.as_ref() == Some(&TransformationType::DeltaMultiple),
                                "Delta (Multiple Columns)",
                            ).clicked() {
                                new_transform_type = Some(TransformationType::DeltaMultiple);
                                transform_type_changed = true;
                            }
                            if ui.selectable_label(
                                current_transform_type.as_ref() == Some(&TransformationType::TimeBin),
                                "Time Bin Column",
                            ).clicked() {
                                new_transform_type = Some(TransformationType::TimeBin);
                                transform_type_changed = true;
                            }
                            if ui.selectable_label(
                                current_transform_type.as_ref() == Some(&TransformationType::RowId),
                                "Row ID Columns",
                            ).clicked() {
                                new_transform_type = Some(TransformationType::RowId);
                                transform_type_changed = true;
                            }
                        });

                    // Configuration based on transformation type
                    if let Some(transform_type) = &self.transformation_type {
                        ui.separator();
                        let available_columns = self.available_columns.clone();
                        match transform_type {
                            TransformationType::Delta => {
                                show_delta_config_with_data(ui, &available_columns, &mut selected_columns);
                            }
                            TransformationType::DeltaMultiple => {
                                show_delta_multiple_config_with_data(ui, &available_columns, &mut selected_columns);
                            }
                            TransformationType::TimeBin => {
                                show_time_bin_config_with_data(ui, &available_columns, &mut time_column, &mut bin_size);
                            }
                            TransformationType::RowId => {
                                show_row_id_config_with_data(ui, &available_columns, &mut grouping_columns);
                            }
                        }

                        // Output column name
                        ui.separator();
                        ui.label("Output Column Name:");
                        ui.text_edit_singleline(&mut output_column_name);

                        // Apply button
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Apply Transformation").clicked() {
                                apply_clicked = true;
                            }
                            if ui.button("Cancel").clicked() {
                                cancel_clicked = true;
                            }
                        });

                        // Error/success messages
                        if let Some(ref error) = self.error_message {
                            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), error);
                        }
                        if let Some(ref success) = self.success_message {
                            ui.colored_label(egui::Color32::from_rgb(100, 255, 100), success);
                        }
                    }
                }
            });
        // Handle table selection outside the closure
        if table_selected {
            if let Some(table_name) = selected_table_name {
                self.selected_table = Some(table_name);
                self.update_available_columns(database);
                self.reset_transformation_state();
            }
        }
        // Handle transformation type change outside the closure
        if transform_type_changed {
            if let Some(new_type) = new_transform_type {
                self.transformation_type = Some(new_type);
                self.reset_column_selection();
            }
        }
        // Write back local state to self
        self.selected_columns = selected_columns;
        self.output_column_name = output_column_name;
        self.time_column = time_column;
        self.grouping_columns = grouping_columns;
        self.bin_size = bin_size;
        // Handle button clicks outside the closure
        if apply_clicked {
            if let Some(table_name) = &self.selected_table {
                if !self.output_column_name.is_empty() {
                    if let Some(transform_type) = &self.transformation_type {
                        result = Some(TransformationRequest {
                            table_name: table_name.clone(),
                            transformation_type: transform_type.clone(),
                            selected_columns: self.selected_columns.clone(),
                            output_column_name: self.output_column_name.clone(),
                            bin_size: self.bin_size.clone(),
                            time_column: self.time_column.clone(),
                            grouping_columns: if self.grouping_columns.is_empty() {
                                None
                            } else {
                                Some(self.grouping_columns.clone())
                            },
                        });
                        self.success_message = Some("Transformation applied successfully!".to_string());
                        self.visible = false;
                    }
                } else {
                    self.error_message = Some("Output column name is required".to_string());
                }
            }
        }
        if cancel_clicked {
            self.visible = false;
            self.reset();
        }
        result
    }

    fn show_delta_config_with_data(&mut self, ui: &mut egui::Ui, available_columns: &[String], selected_columns: &[String]) {
        ui.label("Select Column:");
        egui::ComboBox::from_id_source("column_select")
            .selected_text(selected_columns.first().unwrap_or(&"Select a column".to_string()))
            .show_ui(ui, |ui| {
                for column in available_columns {
                    if ui.selectable_label(
                        selected_columns.first().map_or(false, |c| c == column),
                        column,
                    ).clicked() {
                        self.selected_columns = vec![column.clone()];
                    }
                }
            });
    }

    fn show_delta_multiple_config_with_data(&mut self, ui: &mut egui::Ui, available_columns: &[String], selected_columns: &[String]) {
        ui.label("Select Columns (Multiple):");
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for column in available_columns {
                    let mut is_selected = selected_columns.contains(column);
                    if ui.checkbox(&mut is_selected, column).clicked() {
                        if is_selected {
                            if !self.selected_columns.contains(column) {
                                self.selected_columns.push(column.clone());
                            }
                        } else {
                            self.selected_columns.retain(|c| c != column);
                        }
                    }
                }
            });
    }

    fn show_time_bin_config_with_data(&mut self, ui: &mut egui::Ui, available_columns: &[String], time_column: &Option<String>, bin_size: &String) {
        ui.label("Time Column:");
        egui::ComboBox::from_id_source("time_column_select")
            .selected_text(time_column.as_deref().unwrap_or("Select time column"))
            .show_ui(ui, |ui| {
                for column in available_columns {
                    if ui.selectable_label(
                        time_column.as_deref() == Some(column),
                        column,
                    ).clicked() {
                        self.time_column = Some(column.clone());
                    }
                }
            });
        
        ui.label("Bin Size (seconds):");
        ui.text_edit_singleline(&mut self.bin_size);
    }

    fn show_row_id_config_with_data(&mut self, ui: &mut egui::Ui, available_columns: &[String], grouping_columns: &[String]) {
        ui.label("Grouping Columns (Optional):");
        ui.label("Leave empty for global row IDs only");
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for column in available_columns {
                    let mut is_selected = grouping_columns.contains(column);
                    if ui.checkbox(&mut is_selected, column).clicked() {
                        if is_selected {
                            if !self.grouping_columns.contains(column) {
                                self.grouping_columns.push(column.clone());
                            }
                        } else {
                            self.grouping_columns.retain(|c| c != column);
                        }
                    }
                }
            });
    }

    pub fn update_available_tables(&mut self, database: &Database) {
        self.available_tables = database.get_tables().unwrap_or_default();
    }

    pub fn update_available_columns(&mut self, database: &Database) {
        if let Some(table_name) = &self.selected_table {
            if let Ok(columns) = database.get_column_names(&format!("SELECT * FROM {}", table_name)) {
                self.available_columns = columns;
            }
        }
    }

    fn reset_transformation_state(&mut self) {
        self.transformation_type = None;
        self.selected_columns.clear();
        self.output_column_name.clear();
        self.time_column = None;
        self.grouping_columns.clear();
        self.error_message = None;
        self.success_message = None;
    }

    fn reset_column_selection(&mut self) {
        self.selected_columns.clear();
        self.time_column = None;
        self.grouping_columns.clear();
    }

    fn reset(&mut self) {
        self.selected_table = None;
        self.reset_transformation_state();
    }
}

#[derive(Debug)]
pub struct TransformationRequest {
    pub table_name: String,
    pub transformation_type: TransformationType,
    pub selected_columns: Vec<String>,
    pub output_column_name: String,
    pub bin_size: String,
    pub time_column: Option<String>,
    pub grouping_columns: Option<Vec<String>>,
}

pub struct TransformationManager {
    pub transformer: DataTransformer,
    pub output_directory: PathBuf,
}

impl TransformationManager {
    pub fn new() -> Self {
        let output_dir = PathBuf::from("transformed_data");
        std::fs::create_dir_all(&output_dir).ok();
        
        Self {
            transformer: DataTransformer::new(),
            output_directory: output_dir,
        }
    }

    pub fn apply_transformation(&self, request: &TransformationRequest, database: &Database) -> Result<String> {
        // Get the data from the database
        let query = format!("SELECT * FROM {}", request.table_name);
        let rows = database.execute_query(&query)?;
        
        if rows.is_empty() {
            return Err(anyhow!("No data found in table"));
        }

        // Convert the rows to a RecordBatch
        let batch = self.convert_rows_to_batch(&rows)?;
        
        let transformed_batch = match request.transformation_type {
            TransformationType::Delta => {
                if request.selected_columns.len() != 1 {
                    return Err(anyhow!("Delta transformation requires exactly one column"));
                }
                self.transformer.apply_delta(&batch, &request.selected_columns[0], &request.output_column_name)?
            }
            TransformationType::DeltaMultiple => {
                if request.selected_columns.is_empty() {
                    return Err(anyhow!("Delta multiple transformation requires at least one column"));
                }
                self.transformer.apply_delta_multiple(&batch, &request.selected_columns, &request.output_column_name)?
            }
            TransformationType::TimeBin => {
                let time_column = request.time_column.as_ref()
                    .ok_or_else(|| anyhow!("Time column is required for time binning"))?;
                let bin_size: f64 = request.bin_size.parse()
                    .map_err(|_| anyhow!("Invalid bin size"))?;
                self.transformer.apply_time_bin(&batch, time_column, bin_size, &request.output_column_name)?
            }
            TransformationType::RowId => {
                self.transformer.apply_row_id(&batch, &request.output_column_name, request.grouping_columns.as_deref())?
            }
        };

        // Save the transformed data
        let output_filename = format!("{}_{}.arrow", request.table_name, request.output_column_name);
        let output_path = self.output_directory.join(output_filename);
        self.transformer.save_transformed_data(&transformed_batch, &output_path)?;

        Ok(output_path.to_string_lossy().to_string())
    }

    fn convert_rows_to_batch(&self, rows: &Vec<Vec<String>>) -> Result<RecordBatch> {
        // This is a simplified conversion - in a real implementation, you'd need to handle
        // the actual data types and convert the string data appropriately
        use datafusion::arrow::array::StringArray;
        
        let num_rows = rows.len();
        let num_cols = if num_rows > 0 { rows[0].len() } else { 0 };
        
        if num_rows == 0 || num_cols == 0 {
            return Err(anyhow!("No data to convert"));
        }
        
        // Create string arrays for each column
        let mut arrays: Vec<ArrayRef> = Vec::new();
        let mut field_names: Vec<String> = Vec::new();
        
        for col_idx in 0..num_cols {
            let mut column_data: Vec<String> = Vec::new();
            for row in rows {
                if col_idx < row.len() {
                    column_data.push(row[col_idx].clone());
                } else {
                    column_data.push("".to_string());
                }
            }
            
            let field_name = format!("column_{}", col_idx);
            field_names.push(field_name.clone());
            
            let string_array = StringArray::from(column_data);
            arrays.push(Arc::new(string_array));
        }
        
        // Create schema
        let fields: Vec<Arc<Field>> = field_names.iter()
            .map(|name| Arc::new(Field::new(name, DataType::Utf8, true)))
            .collect();
        let schema = Arc::new(Schema::new(fields));
        
        Ok(RecordBatch::try_new(schema, arrays)?)
    }
} 

// Refactored config methods to use local variables
fn show_delta_config_with_data(ui: &mut egui::Ui, available_columns: &[String], selected_columns: &mut Vec<String>) {
    ui.label("Select Column:");
    egui::ComboBox::from_id_source("column_select")
        .selected_text(selected_columns.first().unwrap_or(&"Select a column".to_string()))
        .show_ui(ui, |ui| {
            for column in available_columns {
                if ui.selectable_label(
                    selected_columns.first().map_or(false, |c| c == column),
                    column,
                ).clicked() {
                    selected_columns.clear();
                    selected_columns.push(column.clone());
                }
            }
        });
}
fn show_delta_multiple_config_with_data(ui: &mut egui::Ui, available_columns: &[String], selected_columns: &mut Vec<String>) {
    ui.label("Select Columns (Multiple):");
    egui::ScrollArea::vertical()
        .max_height(150.0)
        .show(ui, |ui| {
            for column in available_columns {
                let mut is_selected = selected_columns.contains(column);
                if ui.checkbox(&mut is_selected, column).clicked() {
                    if is_selected {
                        if !selected_columns.contains(column) {
                            selected_columns.push(column.clone());
                        }
                    } else {
                        selected_columns.retain(|c| c != column);
                    }
                }
            }
        });
}
fn show_time_bin_config_with_data(ui: &mut egui::Ui, available_columns: &[String], time_column: &mut Option<String>, bin_size: &mut String) {
    ui.label("Time Column:");
    egui::ComboBox::from_id_source("time_column_select")
        .selected_text(time_column.as_deref().unwrap_or("Select time column"))
        .show_ui(ui, |ui| {
            for column in available_columns {
                if ui.selectable_label(
                    time_column.as_deref() == Some(column),
                    column,
                ).clicked() {
                    *time_column = Some(column.clone());
                }
            }
        });
    ui.label("Bin Size (seconds):");
    ui.text_edit_singleline(bin_size);
}
fn show_row_id_config_with_data(ui: &mut egui::Ui, available_columns: &[String], grouping_columns: &mut Vec<String>) {
    ui.label("Grouping Columns (Optional):");
    ui.label("Leave empty for global row IDs only");
    egui::ScrollArea::vertical()
        .max_height(150.0)
        .show(ui, |ui| {
            for column in available_columns {
                let mut is_selected = grouping_columns.contains(column);
                if ui.checkbox(&mut is_selected, column).clicked() {
                    if is_selected {
                        if !grouping_columns.contains(column) {
                            grouping_columns.push(column.clone());
                        }
                    } else {
                        grouping_columns.retain(|c| c != column);
                    }
                }
            }
        });
} 