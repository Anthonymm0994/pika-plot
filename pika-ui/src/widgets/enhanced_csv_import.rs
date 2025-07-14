//! Enhanced CSV import dialog inspired by Pebble's superior design.

use egui::{Ui, Color32, ScrollArea, TextEdit, ComboBox, DragValue, Button, Context, Id};
use egui_extras::{TableBuilder, Column};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Text,
    Integer,
    Real,
    Boolean,
    Date,
    DateTime,
}

impl DataType {
    pub fn to_sql_type(&self) -> &'static str {
        match self {
            DataType::Text => "TEXT",
            DataType::Integer => "INTEGER", 
            DataType::Real => "REAL",
            DataType::Boolean => "BOOLEAN",
            DataType::Date => "DATE",
            DataType::DateTime => "DATETIME",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileConfig {
    pub path: PathBuf,
    pub table_name: String,
    pub header_row: usize,
    pub delimiter: char,
    pub sample_size: usize,
    pub columns: Vec<ColumnConfig>,
    pub null_values: Vec<String>,
    pub preview_data: Option<PreviewData>,
}

#[derive(Debug, Clone)]
pub struct ColumnConfig {
    pub name: String,
    pub data_type: DataType,
    pub included: bool,
    pub create_index: bool,
    pub is_primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub rows: Vec<Vec<String>>,
}

impl FileConfig {
    pub fn new(path: PathBuf) -> Self {
        let table_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_data")
            .to_string();
            
        Self {
            path,
            table_name,
            header_row: 0,
            delimiter: ',',
            sample_size: 1000,
            columns: Vec::new(),
            null_values: vec!["".to_string(), "NULL".to_string(), "null".to_string()],
            preview_data: None,
        }
    }
    
    pub fn file_name(&self) -> String {
        self.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum ProcessingState {
    Idle,
    Loading(f32, String),
    Processing(String, f32),
    Complete,
    Error(String),
}

/// Enhanced CSV import dialog with Pebble-inspired design
pub struct EnhancedCsvImportDialog {
    id: Id,
    pub show: bool,
    pub database_path: Option<PathBuf>,
    pub files: Vec<FileConfig>,
    pub current_file_index: usize,
    pub create_database: bool,
    
    // UI state
    null_value_input: String,
    error: Option<String>,
    processing_state: Arc<Mutex<ProcessingState>>,
    needs_resampling: bool,
    pk_changed_index: Option<usize>,
}

impl Default for EnhancedCsvImportDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedCsvImportDialog {
    pub fn new() -> Self {
        Self {
            id: Id::new("enhanced_csv_import"),
            show: false,
            database_path: None,
            files: Vec::new(),
            current_file_index: 0,
            create_database: true,
            null_value_input: String::new(),
            error: None,
            processing_state: Arc::new(Mutex::new(ProcessingState::Idle)),
            needs_resampling: false,
            pk_changed_index: None,
        }
    }
    
    pub fn open(&mut self, path: PathBuf) {
        self.show = true;
        self.files = vec![FileConfig::new(path)];
        self.current_file_index = 0;
        self.load_preview_for_current_file();
    }
    
    pub fn open_with_csv_selection(&mut self) {
        self.show = true;
        self.files.clear();
        self.current_file_index = 0;
        
        // Open file dialog for CSV selection
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .set_title("Select CSV files to import")
            .pick_files()
        {
            for file in files {
                self.add_file(file);
            }
            if !self.files.is_empty() {
                self.load_preview_for_current_file();
            }
        }
    }
    
    fn reset(&mut self) {
        self.files.clear();
        self.current_file_index = 0;
        self.error = None;
        self.null_value_input.clear();
        *self.processing_state.lock().unwrap() = ProcessingState::Idle;
    }
    
    pub fn add_file(&mut self, path: PathBuf) {
        self.files.push(FileConfig::new(path));
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<PathBuf> {
        let mut result = None;
        
        if self.show {
            egui::Window::new("📊 Enhanced CSV Import")
                .id(self.id)
                .collapsible(false)
                .resizable(true)
                .default_width(900.0)
                .default_height(700.0)
                .show(ctx, |ui| {
                    result = self.render_content(ui);
                });
        }
        
        result
    }
    
    fn render_content(&mut self, ui: &mut Ui) -> Option<PathBuf> {
        let mut created_db_path = None;
        
        // Header
        ui.horizontal(|ui| {
            ui.heading("📊 Enhanced CSV Import");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Cancel").clicked() {
                    self.show = false;
                }
                if ui.button("✅ Import All").clicked() {
                    if let Some(path) = self.start_database_creation() {
                        created_db_path = Some(path);
                        self.show = false;
                    }
                }
            });
        });
        
        ui.separator();
        
        // Error display
        if let Some(ref error) = self.error {
            ui.colored_label(Color32::RED, format!("❌ Error: {}", error));
            ui.separator();
        }
        
        // File selection area
        if self.files.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("📂 Select CSV Files");
                ui.label("Click the button below to select CSV files for import");
                ui.add_space(20.0);
                
                if ui.button("📂 Select CSV Files").clicked() {
                    self.open_with_csv_selection();
                }
                ui.add_space(50.0);
            });
        } else {
            // File tabs
            ui.horizontal(|ui| {
                ui.label("Files:");
                for (i, file) in self.files.iter().enumerate() {
                    let selected = i == self.current_file_index;
                    if ui.selectable_label(selected, file.file_name()).clicked() {
                        self.current_file_index = i;
                        self.load_preview_for_current_file();
                    }
                }
                
                if ui.button("➕ Add More").clicked() {
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("CSV files", &["csv"])
                        .pick_files()
                    {
                        for file in files {
                            self.add_file(file);
                        }
                    }
                }
            });
            
            ui.separator();
            
            // Main content area
            if let Some(current_file) = self.files.get_mut(self.current_file_index) {
                ui.horizontal(|ui| {
                    // Left panel - File configuration
                    ui.vertical(|ui| {
                        ui.set_width(400.0);
                        self.render_file_configuration(ui, current_file);
                    });
                    
                    ui.separator();
                    
                    // Right panel - Data preview
                    ui.vertical(|ui| {
                        self.render_data_preview(ui, current_file);
                    });
                });
            }
        }
        
        created_db_path
    }
    
    fn render_file_configuration(&mut self, ui: &mut Ui, config: &mut FileConfig) {
        ui.heading("⚙️ File Configuration");
        
        // Basic settings
        ui.group(|ui| {
            ui.label("📋 Basic Settings");
            
            ui.horizontal(|ui| {
                ui.label("Table name:");
                ui.text_edit_singleline(&mut config.table_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Delimiter:");
                ComboBox::from_id_source("delimiter")
                    .selected_text(match config.delimiter {
                        ',' => "Comma (,)",
                        ';' => "Semicolon (;)",
                        '\t' => "Tab",
                        '|' => "Pipe (|)",
                        _ => "Other",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.delimiter, ',', "Comma (,)");
                        ui.selectable_value(&mut config.delimiter, ';', "Semicolon (;)");
                        ui.selectable_value(&mut config.delimiter, '\t', "Tab");
                        ui.selectable_value(&mut config.delimiter, '|', "Pipe (|)");
                    });
                
                if ui.button("🔄 Refresh Preview").clicked() {
                    self.load_preview_for_file(config);
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Header row:");
                ui.add(DragValue::new(&mut config.header_row).range(0..=10));
            });
            
            ui.horizontal(|ui| {
                ui.label("Sample size:");
                ui.add(DragValue::new(&mut config.sample_size).range(100..=10000));
            });
        });
        
        ui.add_space(10.0);
        
        // Column configuration
        if !config.columns.is_empty() {
            ui.group(|ui| {
                ui.label("📊 Column Configuration");
                
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .column(Column::auto().at_least(120.0)) // Name
                            .column(Column::auto().at_least(80.0))  // Type
                            .column(Column::auto().at_least(60.0))  // Include
                            .column(Column::auto().at_least(50.0))  // PK
                            .column(Column::auto().at_least(70.0))  // Not Null
                            .column(Column::auto().at_least(60.0))  // Unique
                            .column(Column::auto().at_least(60.0))  // Index
                            .header(20.0, |mut header| {
                                header.col(|ui| { ui.strong("Column"); });
                                header.col(|ui| { ui.strong("Type"); });
                                header.col(|ui| { ui.strong("Include"); });
                                header.col(|ui| { ui.strong("PK"); });
                                header.col(|ui| { ui.strong("Not Null"); });
                                header.col(|ui| { ui.strong("Unique"); });
                                header.col(|ui| { ui.strong("Index"); });
                            })
                            .body(|mut body| {
                                for (i, column) in config.columns.iter_mut().enumerate() {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.text_edit_singleline(&mut column.name);
                                        });
                                        row.col(|ui| {
                                            ComboBox::from_id_source(format!("type_{}", i))
                                                .selected_text(format!("{:?}", column.data_type))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut column.data_type, DataType::Text, "Text");
                                                    ui.selectable_value(&mut column.data_type, DataType::Integer, "Integer");
                                                    ui.selectable_value(&mut column.data_type, DataType::Real, "Real");
                                                    ui.selectable_value(&mut column.data_type, DataType::Boolean, "Boolean");
                                                    ui.selectable_value(&mut column.data_type, DataType::Date, "Date");
                                                    ui.selectable_value(&mut column.data_type, DataType::DateTime, "DateTime");
                                                });
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut column.included, "");
                                        });
                                        row.col(|ui| {
                                            if ui.checkbox(&mut column.is_primary_key, "").changed() && column.is_primary_key {
                                                // Ensure only one primary key
                                                for (j, other_col) in config.columns.iter_mut().enumerate() {
                                                    if j != i {
                                                        other_col.is_primary_key = false;
                                                    }
                                                }
                                                self.pk_changed_index = Some(i);
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut column.not_null, "");
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut column.unique, "");
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut column.create_index, "");
                                        });
                                    });
                                }
                            });
                    });
            });
        }
        
        ui.add_space(10.0);
        
        // Null values configuration
        ui.group(|ui| {
            ui.label("🚫 Null Value Handling");
            
            ui.horizontal(|ui| {
                ui.label("Null values:");
                ui.text_edit_singleline(&mut self.null_value_input);
                if ui.button("➕ Add").clicked() && !self.null_value_input.trim().is_empty() {
                    config.null_values.push(self.null_value_input.trim().to_string());
                    self.null_value_input.clear();
                }
            });
            
            ui.horizontal_wrapped(|ui| {
                let mut to_remove = None;
                for (i, null_val) in config.null_values.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("\"{}\"", null_val));
                        if ui.small_button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_remove {
                    config.null_values.remove(i);
                }
            });
        });
    }
    
    fn render_data_preview(&mut self, ui: &mut Ui, config: &FileConfig) {
        ui.heading("👁️ Data Preview");
        
        if let Some(ref preview) = config.preview_data {
            ScrollArea::both()
                .max_height(400.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .columns(Column::auto().at_least(80.0), preview.rows.first().map_or(0, |r| r.len()))
                        .header(20.0, |mut header| {
                            if let Some(first_row) = preview.rows.first() {
                                for (i, cell) in first_row.iter().enumerate() {
                                    header.col(|ui| {
                                        if config.header_row == 0 {
                                            ui.strong(cell);
                                        } else {
                                            ui.strong(&format!("Column {}", i + 1));
                                        }
                                    });
                                }
                            }
                        })
                        .body(|mut body| {
                            let skip_rows = if config.header_row == 0 { 1 } else { 0 };
                            for row in preview.rows.iter().skip(skip_rows).take(50) {
                                body.row(18.0, |mut table_row| {
                                    for cell in row.iter() {
                                        table_row.col(|ui| {
                                            ui.label(cell);
                                        });
                                    }
                                });
                            }
                        });
                });
        } else {
            ui.label("No preview data available. Click 'Refresh Preview' to load data.");
        }
    }
    
    fn load_preview_for_current_file(&mut self) {
        if let Some(config) = self.files.get_mut(self.current_file_index) {
            self.load_preview_for_file(config);
        }
    }
    
    fn load_preview_for_file(&mut self, config: &mut FileConfig) {
        // Simple CSV reading for preview
        if let Ok(content) = std::fs::read_to_string(&config.path) {
            let mut rows = Vec::new();
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(config.delimiter as u8)
                .has_headers(false)
                .from_reader(content.as_bytes());
            
            for (i, result) in reader.records().enumerate() {
                if i >= config.sample_size {
                    break;
                }
                if let Ok(record) = result {
                    let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                    rows.push(row);
                }
            }
            
            // Infer column types and create column configs
            if !rows.is_empty() && config.columns.is_empty() {
                let header_row_idx = config.header_row;
                let headers = if header_row_idx < rows.len() {
                    rows[header_row_idx].clone()
                } else {
                    (0..rows[0].len()).map(|i| format!("Column_{}", i + 1)).collect()
                };
                
                for (i, header) in headers.iter().enumerate() {
                    let data_type = self.infer_column_type_from_data(&rows, i, header_row_idx);
                    config.columns.push(ColumnConfig {
                        name: header.clone(),
                        data_type,
                        included: true,
                        create_index: false,
                        is_primary_key: false,
                        not_null: false,
                        unique: false,
                    });
                }
            }
            
            config.preview_data = Some(PreviewData { rows });
        }
    }
    
    fn infer_column_type_from_data(&self, rows: &[Vec<String>], col_idx: usize, header_row: usize) -> DataType {
        let data_rows = if header_row == 0 { &rows[1..] } else { rows };
        
        for row in data_rows.iter().take(100) {
            if let Some(cell) = row.get(col_idx) {
                if !cell.trim().is_empty() {
                    // Try to infer type from the first non-empty value
                    if cell.parse::<i64>().is_ok() {
                        return DataType::Integer;
                    } else if cell.parse::<f64>().is_ok() {
                        return DataType::Real;
                    } else if cell.to_lowercase() == "true" || cell.to_lowercase() == "false" {
                        return DataType::Boolean;
                    }
                    // Add date/datetime inference logic here if needed
                }
            }
        }
        
        DataType::Text // Default fallback
    }
    
    fn start_database_creation(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            self.error = Some("No files selected for import".to_string());
            return None;
        }
        
        // For now, just return a mock database path
        // In a real implementation, this would create the database and import the data
        let db_path = std::env::temp_dir().join("pika_import.db");
        println!("📊 Would create database at: {:?}", db_path);
        
        for (i, file) in self.files.iter().enumerate() {
            println!("  📄 File {}: {} -> table '{}'", i + 1, file.file_name(), file.table_name);
            println!("    Columns: {}", file.columns.len());
            println!("    Delimiter: {:?}", file.delimiter);
        }
        
        Some(db_path)
    }
} 