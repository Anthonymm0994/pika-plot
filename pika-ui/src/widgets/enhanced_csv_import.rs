//! CSV import dialog based on Pebble's design.

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
    pub preview_data: Option<(Vec<String>, Vec<Vec<String>>)>,
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
                let mut needs_preview_load = false;
                for (i, file) in self.files.iter().enumerate() {
                    if ui.selectable_label(i == self.current_file_index, &file.file_name()).clicked() {
                        self.current_file_index = i;
                        // Don't call load_preview here, mark it for later
                        needs_preview_load = true;
                    }
                }
                
                // Load preview after iteration
                if needs_preview_load {
                    self.load_preview_for_current_file();
                }
            });
            
            ui.separator();
            
            // Main content area
            if let Some(current_file) = self.files.get_mut(self.current_file_index) {
                // Clone values needed inside the closure to avoid borrow issues
                let mut needs_render = false;
                
                ui.horizontal(|ui| {
                    ui.label("Quick Settings:");
                    if ui.button("⚙️ Configure").clicked() {
                        needs_render = true;
                    }
                });
                
                if needs_render {
                    // Use index-based method to avoid borrowing issues
                    self.render_file_configuration_for_index(ui, self.current_file_index);
                }
            }
        }
        
        created_db_path
    }
    
    fn render_file_configuration_for_index(&mut self, ui: &mut Ui, file_index: usize) {
        if file_index >= self.files.len() {
            return;
        }
        
        // Inline the configuration rendering to avoid borrowing issues
        let config = &mut self.files[file_index];
        
        ui.heading(format!("📄 {}", config.file_name()));
        
        ui.horizontal(|ui| {
            ui.label("Table name:");
            ui.text_edit_singleline(&mut config.table_name);
        });
        
        // Add other configuration UI here...
        ui.separator();
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
                    self.load_preview_for_file(self.current_file_index);
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
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| { ui.label("Column Name"); });
                                header.col(|ui| { ui.label("Data Type"); });
                                header.col(|ui| { ui.label("Include"); });
                                header.col(|ui| { ui.label("Notes"); });
                            })
                            .body(|mut body| {
                                let num_columns = config.columns.len();
                                for i in 0..num_columns {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| { 
                                            ui.label(&config.columns[i].name); 
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{:?}", config.columns[i].data_type));
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut config.columns[i].included, "");
                                        });
                                        row.col(|ui| {
                                            // Check for duplicate column names
                                            let current_name = &config.columns[i].name;
                                            let mut is_duplicate = false;
                                            for j in 0..num_columns {
                                                if i != j && config.columns[j].name == *current_name {
                                                    is_duplicate = true;
                                                    break;
                                                }
                                            }
                                            if is_duplicate {
                                                ui.colored_label(Color32::YELLOW, "⚠️ Duplicate name");
                                            }
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
        
        if let Some((headers, preview)) = &config.preview_data {
            ScrollArea::both()
                .max_height(400.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .columns(Column::auto().at_least(80.0), headers.len())
                        .header(20.0, |mut header| {
                            for (i, header_text) in headers.iter().enumerate() {
                                header.col(|ui| {
                                    ui.strong(header_text);
                                });
                            }
                        })
                        .body(|mut body| {
                            let skip_rows = if config.header_row == 0 { 1 } else { 0 };
                            for row in preview.iter().skip(skip_rows).take(50) {
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
            self.load_preview_for_file(self.current_file_index);
        }
    }
    
    fn load_preview_for_file(&mut self, file_index: usize) {
        if file_index >= self.files.len() {
            return;
        }
        
        let file_path = self.files[file_index].path.clone();
        
        match std::fs::File::open(&file_path) {
            Ok(file) => {
                let mut reader = csv::ReaderBuilder::new()
                    .delimiter(self.files[file_index].delimiter as u8)
                    .has_headers(self.files[file_index].header_row == 0) // Assuming header_row 0 means headers are present
                    .from_reader(file);
                
                // Read headers
                let headers = if self.files[file_index].header_row == 0 {
                    match reader.headers() {
                        Ok(h) => h.iter().map(|s| s.to_string()).collect(),
                        Err(_) => vec![]
                    }
                } else {
                    vec![]
                };
                
                // Read preview rows
                let mut rows = Vec::new();
                for (i, result) in reader.records().enumerate() {
                    if i >= 10 { break; } // Limit preview to 10 rows
                    
                    match result {
                        Ok(record) => {
                            rows.push(record.iter().map(|s| s.to_string()).collect());
                        }
                        Err(_) => break
                    }
                }
                
                self.files[file_index].preview_data = Some((headers, rows));
            }
            Err(e) => {
                self.error = Some(format!("Failed to read file: {}", e));
            }
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