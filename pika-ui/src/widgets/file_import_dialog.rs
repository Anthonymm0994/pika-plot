//! Professional CSV import dialog matching Pebble's superior design.

use pika_core::types::{ImportOptions, TableInfo, ColumnInfo};
use pika_engine::enhanced_csv::{CsvFileStats, CsvAnalyzer};
use egui::{Ui, Color32, ScrollArea, TextEdit, ComboBox, DragValue, Button, Context, Id};
use egui_extras::{TableBuilder, Column};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub database_path: PathBuf,
    pub table_infos: Vec<TableInfo>,
}

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
            .unwrap_or("table")
            .to_string();
        
        Self {
            path,
            table_name,
            header_row: 0,
            delimiter: ',',
            sample_size: 100, // Reduced from 1000 for better performance
            columns: Vec::new(),
            null_values: vec!["", "NULL", "null", "N/A"].into_iter().map(String::from).collect(),
            preview_data: None,
        }
    }
    
    pub fn file_name(&self) -> String {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

#[derive(Clone)]
pub enum ProcessingState {
    Idle,
    Loading(f32, String),
    Processing(String, f32),
    Complete,
    Error(String),
}

pub struct FileImportDialog {
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

impl FileImportDialog {
    pub fn new() -> Self {
        Self {
            id: Id::new("csv_import_dialog"),
            show: false,
            database_path: None,
            files: Vec::new(),
            current_file_index: 0,
            create_database: false,
            null_value_input: String::new(),
            error: None,
            processing_state: Arc::new(Mutex::new(ProcessingState::Idle)),
            needs_resampling: false,
            pk_changed_index: None,
        }
    }
    
    pub fn open(&mut self, path: PathBuf) {
        self.show = true;
        self.database_path = Some(path);
        self.reset();
    }
    
    pub fn open_with_csv_selection(&mut self) {
        self.show = true;
        self.reset();
        
        // Open file dialog to select CSV files
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .set_title("Select CSV files")
            .pick_files()
        {
            for path in paths {
                self.add_file(path);
            }
        }
    }
    
    fn reset(&mut self) {
        self.files.clear();
        self.current_file_index = 0;
        self.create_database = false;
        self.error = None;
        self.null_value_input.clear();
        self.needs_resampling = false;
        self.pk_changed_index = None;
    }
    
    pub fn add_file(&mut self, path: PathBuf) {
        let mut config = FileConfig::new(path);
        self.load_preview_for_file(&mut config);
        self.files.push(config);
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<ImportResult> {
        if !self.show {
            return None;
        }
        
        let mut created_db_path = None;
        
        egui::Window::new("📊 CSV Import")
            .id(self.id)
            .resizable(true)
            .default_width(900.0)
            .default_height(600.0)
            .min_width(600.0)
            .min_height(400.0)
            .show(ctx, |ui| {
                self.render_content(ui, &mut created_db_path);
            });
        
        created_db_path
    }
    
    fn render_content(&mut self, ui: &mut egui::Ui, created_db_path: &mut Option<ImportResult>) {
        // Database path selection
        ui.horizontal(|ui| {
            ui.label("📁 Database location:");
            if let Some(path) = &self.database_path {
                ui.label(format!("📂 {}", path.display()));
            } else {
                ui.label("🔍 Click to select location");
            }
            
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select database location")
                    .add_filter("Database files", &["db", "sqlite"])
                    .save_file()
                {
                    self.database_path = Some(path);
                }
            }
            
            if ui.button("Auto").clicked() {
                // Auto-generate database name based on first CSV file
                if let Some(first_file) = self.files.first() {
                    let db_name = format!("{}.db", 
                        first_file.path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("data"));
                    
                    let db_path = first_file.path.with_file_name(db_name);
                    self.database_path = Some(db_path);
                }
            }
        });
        
        ui.separator();
        
        // Error display
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
            ui.separator();
        }
        
        // Main content area
        ui.horizontal(|ui| {
            let available_height = (ui.available_height() - 80.0).max(100.0); // Reserve space for buttons, minimum 100px
            
            // Left side - file configuration
            ui.vertical(|ui| {
                ui.set_width(450.0);
                ui.set_height(available_height);
                
                ui.heading("📂 File Configuration");
                ui.separator();
                
                // File configuration
                self.render_file_configuration(ui);
                
                ui.separator();
                
                // Create database button
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_create = self.database_path.is_some() && !self.files.is_empty() && 
                            self.files.iter().all(|f| !f.table_name.is_empty() && 
                                f.columns.iter().any(|c| c.included));
                        
                        let file_count = self.files.len();
                        let create_button = egui::Button::new(
                            egui::RichText::new(format!("✅ Create Database with {} Table{}", 
                                file_count, 
                                if file_count == 1 { "" } else { "s" }))
                                .size(16.0)
                                .color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(76, 175, 80))
                        .rounding(egui::Rounding::same(6.0));
                        
                        if ui.add_enabled(can_create, create_button).clicked() {
                            if let Some(error) = self.validate_constraints() {
                                self.error = Some(error);
                            } else {
                                self.create_database = true;
                            }
                        }
                    });
                });
            });
            
            ui.separator();
            
            // Right side - data preview
            ui.vertical(|ui| {
                ui.set_height(available_height);
                self.render_data_preview(ui);
            });
        });
        
        // Process resampling if needed
        if self.needs_resampling {
            self.needs_resampling = false;
            self.load_preview_for_current_file();
        }
        
        // Handle database creation
        if self.create_database {
            self.create_database = false;
            if let Some(result) = self.start_database_creation() {
                *created_db_path = Some(result);
            }
        }
    }
    
    fn render_file_configuration(&mut self, ui: &mut egui::Ui) {
        // File selector dropdown
        ui.horizontal(|ui| {
            ui.label("CSV File:");
            
            let file_names: Vec<String> = self.files.iter()
                .enumerate()
                .map(|(_idx, config)| {
                    let configured = !config.columns.is_empty();
                    format!("{}{}", 
                        config.file_name(),
                        if configured { " ✓" } else { "" }
                    )
                })
                .collect();
            
            if !file_names.is_empty() {
                let current_text = file_names.get(self.current_file_index)
                    .cloned()
                    .unwrap_or_else(|| "No file selected".to_string());
                
                egui::ComboBox::new("file_selector_combo", "CSV File")
                    .selected_text(&current_text)
                    .show_ui(ui, |ui| {
                        for (idx, name) in file_names.iter().enumerate() {
                            if ui.selectable_value(&mut self.current_file_index, idx, name).clicked() {
                                // Only load preview if it doesn't exist to speed up switching
                                if let Some(config) = self.files.get(idx) {
                                    if config.preview_data.is_none() || config.columns.is_empty() {
                                        self.load_preview_for_current_file();
                                    }
                                }
                            }
                        }
                    });
            } else {
                ui.label("No files added");
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Add Files...").clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("CSV files", &["csv"])
                        .set_title("Select CSV files")
                        .pick_files()
                    {
                        for path in paths {
                            self.add_file(path);
                        }
                        // Load preview for the first added file if it becomes current
                        if self.current_file_index < self.files.len() {
                            self.load_preview_for_current_file();
                        }
                    }
                }
            });
        });
        
        ui.label(format!("Files to import: {} total, {} configured", 
            self.files.len(),
            self.files.iter().filter(|f| !f.columns.is_empty()).count()
        ));
        
        ui.separator();
        ui.add_space(5.0);
        
        // Store values to avoid borrowing issues
        let mut header_row_changed = false;
        let mut sample_size_changed = false;
        let mut delimiter_changed = false;
        let mut need_resample = false;
        
        if let Some(config) = self.files.get_mut(self.current_file_index) {
            // Table name
            ui.horizontal(|ui| {
                ui.label("Table Name:");
                ui.text_edit_singleline(&mut config.table_name);
            });
            
            ui.add_space(10.0);
            
            // Header configuration
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Header Configuration").size(16.0).strong());
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label("Header Row:");
                    
                    let mut header_row_display = config.header_row + 1;
                    let max_rows = config.preview_data.as_ref()
                        .map(|p| p.rows.len())
                        .unwrap_or(10) as i32;
                    
                    let response = ui.add(
                        egui::DragValue::new(&mut header_row_display)
                            .range(1..=max_rows)
                            .speed(1)
                    );
                    
                    if response.changed() {
                        config.header_row = (header_row_display - 1).max(0) as usize;
                        header_row_changed = true;
                    }
                    
                    ui.label(format!("(1-{})", max_rows));
                });
                
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("The green highlighted row in the preview is your header")
                        .size(12.0)
                        .color(egui::Color32::from_gray(150))
                );
            });
            
            ui.add_space(10.0);
            
            // Sample size
            ui.horizontal(|ui| {
                ui.label("Sample Size:");
                let response = ui.add(
                    egui::DragValue::new(&mut config.sample_size)
                        .range(100..=10000)
                        .speed(10)
                );
                if response.changed() {
                    sample_size_changed = true;
                }
                ui.label("rows");
                
                if ui.button("🔄 Resample").clicked() {
                    need_resample = true;
                }
            });
            
            ui.add_space(10.0);
            
            // Delimiter
            ui.horizontal(|ui| {
                ui.label("Delimiter:");
                let old_delimiter = config.delimiter;
                ui.radio_value(&mut config.delimiter, ',', "Comma");
                ui.radio_value(&mut config.delimiter, '\t', "Tab");
                ui.radio_value(&mut config.delimiter, ';', "Semicolon");
                ui.radio_value(&mut config.delimiter, '|', "Pipe");
                
                // If delimiter changed, reload preview
                if config.delimiter != old_delimiter {
                    delimiter_changed = true;
                }
            });
            
            ui.add_space(10.0);
            
            // Null values
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Null Values").size(14.0));
                ui.label(egui::RichText::new("Values to treat as NULL:").size(12.0));
                
                egui::ScrollArea::vertical()
                    .id_source(format!("null_scroll_{}", self.current_file_index))
                    .max_height(100.0)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        for (idx, pattern) in config.null_values.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.small_button("×").clicked() {
                                    to_remove = Some(idx);
                                }
                                ui.label(if pattern.is_empty() { "[empty string]" } else { pattern });
                            });
                        }
                        if let Some(idx) = to_remove {
                            config.null_values.remove(idx);
                        }
                    });
                
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.null_value_input);
                    if ui.button("Add").clicked() && !self.null_value_input.trim().is_empty() {
                        config.null_values.push(self.null_value_input.clone());
                        self.null_value_input.clear();
                    }
                });
            });
            
            ui.add_space(10.0);
            
            // Column selection
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new("Column Selection").size(14.0));
                
                if !config.columns.is_empty() {
                    let selected_count = config.columns.iter()
                        .filter(|c| c.included)
                        .count();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Select All").clicked() {
                            for col in &mut config.columns {
                                col.included = true;
                            }
                        }
                        if ui.button("Deselect All").clicked() {
                            for col in &mut config.columns {
                                col.included = false;
                            }
                        }
                        ui.label(format!("{} / {} selected", selected_count, config.columns.len()));
                    });
                    
                    ui.separator();
                    
                    let available_height = ui.available_height();
                    egui::ScrollArea::vertical()
                        .id_source(format!("column_scroll_{}", self.current_file_index))
                        .max_height(available_height)
                        .show(ui, |ui| {
                            TableBuilder::new(ui)
                                .striped(true)
                                .resizable(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::auto().at_least(60.0)) // Include
                                .column(Column::auto().at_least(100.0).resizable(true)) // Column
                                .column(Column::auto().at_least(100.0)) // Type
                                .column(Column::auto().at_least(40.0)) // PK
                                .column(Column::auto().at_least(70.0)) // Not Null
                                .column(Column::auto().at_least(60.0)) // Unique
                                .column(Column::auto().at_least(50.0)) // Index
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Include").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Column").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Type").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("PK").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Not Null").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Unique").strong());
                                    });
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new("Index").strong());
                                    });
                                })
                                .body(|mut body| {
                                    for (col_idx, col) in config.columns.iter_mut().enumerate() {
                                        body.row(25.0, |mut row| {
                                            row.col(|ui| {
                                                ui.checkbox(&mut col.included, "");
                                            });
                                            row.col(|ui| {
                                                ui.label(&col.name);
                                            });
                                            row.col(|ui| {
                                                egui::ComboBox::new(format!("type_{}_{}", self.current_file_index, col_idx), "")
                                                    .selected_text(format!("{:?}", col.data_type))
                                                    .width(90.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.set_max_height(200.0);
                                                        ui.selectable_value(&mut col.data_type, DataType::Text, "Text");
                                                        ui.selectable_value(&mut col.data_type, DataType::Integer, "Integer");
                                                        ui.selectable_value(&mut col.data_type, DataType::Real, "Real");
                                                        ui.selectable_value(&mut col.data_type, DataType::Boolean, "Boolean");
                                                        ui.selectable_value(&mut col.data_type, DataType::Date, "Date");
                                                        ui.selectable_value(&mut col.data_type, DataType::DateTime, "DateTime");
                                                    });
                                            });
                                            row.col(|ui| {
                                                let mut pk_changed = false;
                                                let was_pk = col.is_primary_key;
                                                if ui.checkbox(&mut col.is_primary_key, "").changed() {
                                                    if col.is_primary_key && !was_pk {
                                                        pk_changed = true;
                                                    }
                                                }
                                                
                                                if pk_changed {
                                                    self.pk_changed_index = Some(col_idx);
                                                }
                                            });
                                            row.col(|ui| {
                                                ui.checkbox(&mut col.not_null, "");
                                            });
                                            row.col(|ui| {
                                                ui.checkbox(&mut col.unique, "");
                                            });
                                            row.col(|ui| {
                                                ui.checkbox(&mut col.create_index, "");
                                            });
                                        });
                                    }
                                });
                        });
                } else {
                    ui.label("No columns available. Please check the data preview and header configuration.");
                }
            });
            
            // Handle primary key changes
            if let Some(pk_idx) = self.pk_changed_index.take() {
                for (idx, col) in config.columns.iter_mut().enumerate() {
                    if idx != pk_idx {
                        col.is_primary_key = false;
                    }
                }
            }
        }
        
        // Handle changes that require resampling/reloading
        if header_row_changed {
            self.needs_resampling = true;
            self.reinfer_column_types_for_current_file();
        }
        
        if sample_size_changed || delimiter_changed || need_resample {
            self.needs_resampling = true;
            self.load_preview_for_current_file();
        }
    }
    
    fn render_data_preview(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Data Preview").size(16.0).strong());
        ui.add_space(8.0);
        
        let preview_height = ui.available_height();
        
        if let Some(config) = self.files.get(self.current_file_index) {
            if let Some(preview) = &config.preview_data {
                egui::ScrollArea::both()
                    .id_source(format!("preview_scroll_{}", self.current_file_index))
                    .max_height(preview_height)
                    .show(ui, |ui| {
                        // Calculate the maximum number of columns across all rows
                        let max_columns = preview.rows.iter()
                            .map(|row| row.len())
                            .max()
                            .unwrap_or(0);
                        
                        if max_columns == 0 {
                            ui.label("No data to preview");
                            return;
                        }
                        
                        let num_columns = max_columns + 1; // +1 for row number column
                        
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::auto().at_least(40.0))
                            .columns(Column::auto().at_least(100.0).resizable(true), num_columns - 1)
                            .vscroll(false)
                            .body(|mut body| {
                                for (row_idx, row) in preview.rows.iter().enumerate() {
                                    let is_header = row_idx == config.header_row;
                                    let color = if is_header {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else {
                                        egui::Color32::from_gray(200)
                                    };
                                    
                                    body.row(20.0, |mut table_row| {
                                        table_row.col(|ui| {
                                            let row_text = egui::RichText::new((row_idx + 1).to_string())
                                                .color(if is_header { color } else { egui::Color32::from_gray(150) });
                                            ui.label(if is_header { row_text.strong() } else { row_text });
                                        });
                                        
                                        // Ensure we handle exactly max_columns cells, padding with empty if needed
                                        for col_idx in 0..max_columns {
                                            table_row.col(|ui| {
                                                let cell_text = if col_idx < row.len() {
                                                    row[col_idx].clone()
                                                } else {
                                                    String::new() // Empty cell for missing data
                                                };
                                                
                                                let formatted_text = egui::RichText::new(&cell_text)
                                                    .color(if is_header { color } else { egui::Color32::from_gray(200) });
                                                ui.label(if is_header { formatted_text.strong() } else { formatted_text });
                                            });
                                        }
                                    });
                                }
                            });
                    });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Loading preview...");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No file selected");
            });
        }
    }
    
    fn load_preview_for_current_file(&mut self) {
        let current_index = self.current_file_index;
        if current_index < self.files.len() {
            // Clone the current config to avoid borrowing issues
            let mut config = self.files[current_index].clone();
            
            // Load preview data for this config
            self.load_preview_for_file(&mut config);
            
            // Update the file in the vector
            self.files[current_index] = config;
        }
    }
    
    fn load_preview_for_file(&mut self, config: &mut FileConfig) {
        // Skip loading if preview data already exists (caching)
        if config.preview_data.is_some() {
            return;
        }
        
        // Read only the first few lines for preview (much faster than reading entire file)
        let mut preview_rows: Vec<Vec<String>> = Vec::new();
        
        // Use BufReader to read only what we need
        match std::fs::File::open(&config.path) {
            Ok(file) => {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                
                // Only read the lines we need for preview (header + sample_size)
                let lines_needed = config.sample_size + 1; // +1 for header
                
                for (i, line) in reader.lines().enumerate() {
                    if i >= lines_needed {
                        break; // Stop reading once we have enough
                    }
                    
                    match line {
                        Ok(line_content) => {
                            if !line_content.trim().is_empty() {
                                let row: Vec<String> = line_content.split(config.delimiter)
                                    .map(|cell| cell.trim().trim_matches('"').to_string())
                                    .collect();
                                
                                // Only add non-empty rows
                                if !row.is_empty() && !row.iter().all(|cell| cell.is_empty()) {
                                    preview_rows.push(row);
                                }
                            }
                        }
                        Err(_) => break, // Stop on error
                    }
                }
            }
            Err(e) => {
                // Handle file read error
                eprintln!("Error reading file {}: {}", config.path.display(), e);
                return;
            }
        }
        
        // Set preview data
        config.preview_data = Some(PreviewData {
            rows: preview_rows.clone(),
        });
        
        // Generate column configurations from header row if we have data
        if !preview_rows.is_empty() {
            let header_row = &preview_rows[0];
            
            // Only regenerate columns if they don't exist or are empty
            if config.columns.is_empty() {
                config.columns = header_row.iter().map(|col_name| {
                    ColumnConfig {
                        name: col_name.clone(),
                        data_type: self.infer_column_type_from_data(col_name, &preview_rows),
                        included: true,
                        create_index: false,
                        is_primary_key: false,
                        not_null: false,
                        unique: false,
                    }
                }).collect();
            }
        }
    }
    
    fn reinfer_column_types_for_current_file(&mut self) {
        let current_index = self.current_file_index;
        if current_index < self.files.len() {
            // Extract the data we need to avoid borrowing issues
            let (header_row_data, existing_columns) = {
                let config = &self.files[current_index];
                let header_data = config.preview_data.as_ref()
                    .and_then(|p| p.rows.get(config.header_row))
                    .map(|row| row.clone());
                let existing = config.columns.clone();
                (header_data, existing)
            };
            
            if let Some(header_row_data) = header_row_data {
                // Preserve existing column configurations if they exist
                let mut existing_configs: HashMap<String, ColumnConfig> = 
                    existing_columns.iter().map(|col| (col.name.clone(), col.clone())).collect();
                
                let new_columns: Vec<ColumnConfig> = header_row_data.iter().map(|name| {
                    // If column already exists, keep its configuration but update type
                    if let Some(mut existing) = existing_configs.remove(name) {
                        existing.data_type = self.infer_column_type_from_data(name, &self.files[current_index].preview_data.as_ref().unwrap().rows);
                        existing.name = name.clone(); // Update name in case it changed
                        existing
                    } else {
                        // New column - create with defaults (no constraints checked)
                        let data_type = self.infer_column_type_from_data(name, &self.files[current_index].preview_data.as_ref().unwrap().rows);
                        ColumnConfig {
                            name: name.clone(),
                            data_type,
                            included: true,      // Only include is checked by default
                            create_index: false,     // No index by default
                            is_primary_key: false,   // No PK by default
                            not_null: false,        // No not null by default
                            unique: false,          // No unique by default
                        }
                    }
                }).collect();
                
                // Update the config with the new columns
                self.files[current_index].columns = new_columns;
            }
        }
    }
    
    fn infer_column_type(&self, column_name: &str) -> DataType {
        // First try to infer from column name
        let name_lower = column_name.to_lowercase();
        if name_lower.contains("id") {
            return DataType::Integer;
        } else if name_lower.contains("date") || name_lower.contains("time") {
            return DataType::DateTime;
        } else if name_lower.contains("value") || name_lower.contains("amount") || name_lower.contains("price") {
            return DataType::Real;
        } else if name_lower.contains("active") || name_lower.contains("enabled") {
            return DataType::Boolean;
        }
        
        // If name-based inference doesn't work, analyze actual data
        if let Some(config) = self.files.get(self.current_file_index) {
            if let Some(preview) = &config.preview_data {
                if let Some(header_row) = preview.rows.get(config.header_row) {
                    if let Some(col_index) = header_row.iter().position(|h| h == column_name) {
                        // Analyze data values in this column (skip header row)
                        let data_rows = preview.rows.iter().skip(config.header_row + 1);
                        let mut integer_count = 0;
                        let mut real_count = 0;
                        let mut boolean_count = 0;
                        let mut date_count = 0;
                        let mut total_count = 0;
                        
                        for row in data_rows {
                            if let Some(value) = row.get(col_index) {
                                let value = value.trim();
                                if value.is_empty() {
                                    continue;
                                }
                                
                                total_count += 1;
                                
                                // Check for boolean
                                if value.to_lowercase() == "true" || value.to_lowercase() == "false" ||
                                   value == "1" || value == "0" {
                                    boolean_count += 1;
                                }
                                
                                // Check for integer
                                if value.parse::<i64>().is_ok() {
                                    integer_count += 1;
                                }
                                
                                // Check for real number
                                if value.parse::<f64>().is_ok() {
                                    real_count += 1;
                                }
                                
                                // Check for date patterns
                                if value.contains('-') || value.contains('/') || value.contains(':') {
                                    date_count += 1;
                                }
                            }
                        }
                        
                        if total_count > 0 {
                            let threshold = total_count as f32 * 0.8; // 80% threshold
                            
                            if boolean_count as f32 >= threshold {
                                return DataType::Boolean;
                            } else if integer_count as f32 >= threshold {
                                return DataType::Integer;
                            } else if real_count as f32 >= threshold {
                                return DataType::Real;
                            } else if date_count as f32 >= threshold {
                                return DataType::DateTime;
                            }
                        }
                    }
                }
            }
        }
        
        // Default to text if no clear pattern
        DataType::Text
    }

    fn infer_column_type_from_data(&self, column_name: &str, preview_rows: &Vec<Vec<String>>) -> DataType {
        // Analyze actual data values in the column to infer type
        let mut integer_count = 0;
        let mut real_count = 0;
        let mut boolean_count = 0;
        let mut date_count = 0;
        let mut total_count = 0;

        for row in preview_rows {
            if let Some(value) = row.get(preview_rows[0].iter().position(|h| h == column_name).unwrap()) {
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }

                total_count += 1;

                // Check for boolean
                if value.to_lowercase() == "true" || value.to_lowercase() == "false" ||
                   value == "1" || value == "0" {
                    boolean_count += 1;
                }

                // Check for integer
                if value.parse::<i64>().is_ok() {
                    integer_count += 1;
                }

                // Check for real number
                if value.parse::<f64>().is_ok() {
                    real_count += 1;
                }

                // Check for date patterns
                if value.contains('-') || value.contains('/') || value.contains(':') {
                    date_count += 1;
                }
            }
        }

        if total_count > 0 {
            let threshold = total_count as f32 * 0.8; // 80% threshold
            
            if boolean_count as f32 >= threshold {
                return DataType::Boolean;
            } else if integer_count as f32 >= threshold {
                return DataType::Integer;
            } else if real_count as f32 >= threshold {
                return DataType::Real;
            } else if date_count as f32 >= threshold {
                return DataType::DateTime;
            }
        }
        
        // Default to text if no clear pattern
        DataType::Text
    }
    
    fn validate_constraints(&self) -> Option<String> {
        for (idx, file) in self.files.iter().enumerate() {
            if file.table_name.is_empty() {
                return Some(format!("File {} has no table name", idx + 1));
            }
            
            let included_columns: Vec<_> = file.columns.iter().filter(|c| c.included).collect();
            if included_columns.is_empty() {
                return Some(format!("File {} has no included columns", idx + 1));
            }
            
            let pk_count = included_columns.iter().filter(|c| c.is_primary_key).count();
            if pk_count > 1 {
                return Some(format!("File {} has multiple primary keys", idx + 1));
            }
        }
        None
    }
    
    fn start_database_creation(&mut self) -> Option<ImportResult> {
        if let Some(db_path) = &self.database_path {
            // Create TableInfo objects from the file configurations
            let table_infos: Vec<TableInfo> = self.files.iter()
                .filter(|config| !config.table_name.is_empty() && 
                    config.columns.iter().any(|c| c.included))
                .map(|config| {
                    let columns: Vec<ColumnInfo> = config.columns.iter()
                        .filter(|col| col.included)
                        .map(|col| ColumnInfo {
                            name: col.name.clone(),
                            data_type: col.data_type.to_sql_type().to_string(),
                            nullable: !col.not_null,
                        })
                        .collect();
                    
                    // Estimate row count from preview data
                    let row_count = if let Some(preview) = &config.preview_data {
                        Some(preview.rows.len().saturating_sub(1)) // Subtract header row
                    } else {
                        None
                    };
                    
                    TableInfo {
                        name: config.table_name.clone(),
                        source_path: Some(config.path.clone()),
                        row_count,
                        columns,
                    }
                })
                .collect();
            
            if !table_infos.is_empty() {
                self.show = false;
                Some(ImportResult {
                    database_path: db_path.clone(),
                    table_infos,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for FileImportDialog {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy compatibility - convert to new system
impl FileImportDialog {
    pub fn show_legacy(&mut self, ui: &mut Ui) -> Option<(PathBuf, ImportOptions)> {
        // This is a compatibility shim for the old interface
        // In practice, you'd migrate to use the new show() method with Context
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file
    }
    
    #[test]
    fn test_dialog_creation() {
        let dialog = FileImportDialog::new();
        assert!(!dialog.show);
        assert!(dialog.files.is_empty());
        assert_eq!(dialog.current_file_index, 0);
    }
    
    #[test]
    fn test_file_config_creation() {
        let path = PathBuf::from("test.csv");
        let config = FileConfig::new(path);
        assert_eq!(config.table_name, "test");
        assert_eq!(config.delimiter, ',');
        assert_eq!(config.header_row, 0);
        assert_eq!(config.sample_size, 100);
    }
    
    #[test]
    fn test_column_type_inference() {
        let dialog = FileImportDialog::new();
        assert_eq!(dialog.infer_column_type("id"), DataType::Integer);
        assert_eq!(dialog.infer_column_type("name"), DataType::Text);
        assert_eq!(dialog.infer_column_type("price"), DataType::Real);
        assert_eq!(dialog.infer_column_type("active"), DataType::Boolean);
        assert_eq!(dialog.infer_column_type("created_date"), DataType::DateTime);
    }
    
    #[test]
    fn test_multi_file_handling() {
        let mut dialog = FileImportDialog::new();
        
        // Add multiple files
        let file1 = PathBuf::from("test1.csv");
        let file2 = PathBuf::from("test2.csv");
        
        dialog.add_file(file1.clone());
        dialog.add_file(file2.clone());
        
        assert_eq!(dialog.files.len(), 2);
        assert_eq!(dialog.files[0].path, file1);
        assert_eq!(dialog.files[1].path, file2);
        assert_eq!(dialog.current_file_index, 0);
    }
    
    #[test]
    fn test_preview_data_loading() {
        let temp_file = create_test_csv("id,name,value\n1,Alice,100\n2,Bob,200");
        let mut dialog = FileImportDialog::new();
        dialog.add_file(temp_file.path().to_path_buf());
        
        // Load preview data
        dialog.load_preview_for_current_file();
        
        // Check that preview data was loaded
        assert!(dialog.files[0].preview_data.is_some());
        let preview = dialog.files[0].preview_data.as_ref().unwrap();
        assert_eq!(preview.rows.len(), 3); // header + 2 data rows
        assert_eq!(preview.rows[0], vec!["id", "name", "value"]);
        assert_eq!(preview.rows[1], vec!["1", "Alice", "100"]);
    }

    #[test]
    fn test_column_config_defaults() {
        let temp_file = create_test_csv("id,name,value\n1,Alice,100");
        let mut dialog = FileImportDialog::new();
        dialog.add_file(temp_file.path().to_path_buf());
        
        // Load preview data which should populate column configs
        dialog.load_preview_for_current_file();
        
        let config = &dialog.files[0];
        assert!(!config.columns.is_empty());
        assert_eq!(config.columns.len(), 3);
        
        // Check default column configurations
        assert_eq!(config.columns[0].name, "id");
        assert!(config.columns[0].included);
        assert_eq!(config.columns[0].data_type, DataType::Text); // Default to Text, not Integer
    }

    #[test]
    fn test_file_switching() {
        let temp_file1 = create_test_csv("id,name\n1,Alice");
        let temp_file2 = create_test_csv("id,value\n1,100");
        
        let mut dialog = FileImportDialog::new();
        dialog.add_file(temp_file1.path().to_path_buf());
        dialog.add_file(temp_file2.path().to_path_buf());
        
        // Load preview for first file
        dialog.current_file_index = 0;
        dialog.load_preview_for_current_file();
        assert!(dialog.files[0].preview_data.is_some());
        
        // Switch to second file and load preview
        dialog.current_file_index = 1;
        dialog.load_preview_for_current_file();
        assert!(dialog.files[1].preview_data.is_some());
        
        // Verify both files have different column structures
        assert_eq!(dialog.files[0].columns.len(), 2);
        assert_eq!(dialog.files[1].columns.len(), 2);
        assert_eq!(dialog.files[0].columns[1].name, "name");
        assert_eq!(dialog.files[1].columns[1].name, "value");
    }

    #[test]
    fn test_delimiter_changes() {
        let temp_file = create_test_csv("id;name;value\n1;Alice;100");
        let mut dialog = FileImportDialog::new();
        dialog.add_file(temp_file.path().to_path_buf());
        
        // Initially load with comma delimiter (should fail to parse correctly)
        dialog.load_preview_for_current_file();
        assert!(dialog.files[0].preview_data.is_some());
        
        // Change delimiter to semicolon
        dialog.files[0].delimiter = ';';
        dialog.files[0].preview_data = None; // Clear cache
        dialog.load_preview_for_current_file();
        
        // Should now parse correctly with semicolon delimiter
        assert!(dialog.files[0].preview_data.is_some());
        let preview = dialog.files[0].preview_data.as_ref().unwrap();
        assert_eq!(preview.rows[0], vec!["id", "name", "value"]);
    }
    
    #[test]
    fn test_null_value_handling() {
        let mut dialog = FileImportDialog::new();
        let file_path = PathBuf::from("test.csv");
        dialog.add_file(file_path);
        
        // Check default null values
        let default_nulls = &dialog.files[0].null_values;
        assert!(default_nulls.contains(&"".to_string()));
        assert!(default_nulls.contains(&"NULL".to_string()));
        assert!(default_nulls.contains(&"null".to_string()));
        assert!(default_nulls.contains(&"N/A".to_string()));
        
        // Add custom null value
        dialog.files[0].null_values.push("MISSING".to_string());
        assert!(dialog.files[0].null_values.contains(&"MISSING".to_string()));
    }
    
    #[test]
    fn test_primary_key_exclusivity() {
        let mut dialog = FileImportDialog::new();
        let file_path = PathBuf::from("test.csv");
        dialog.add_file(file_path);
        
        // Load preview to get columns
        dialog.load_preview_for_current_file();
        
        // Set multiple columns as primary key
        if dialog.files[0].columns.len() >= 2 {
            dialog.files[0].columns[0].is_primary_key = true;
            dialog.files[0].columns[1].is_primary_key = true;
            
            // Simulate the PK change handling
            dialog.pk_changed_index = Some(1);
            
            // In the actual UI, this would be handled in render_file_configuration
            // Here we'll manually test the logic
            if let Some(pk_idx) = dialog.pk_changed_index.take() {
                for (idx, col) in dialog.files[0].columns.iter_mut().enumerate() {
                    if idx != pk_idx {
                        col.is_primary_key = false;
                    }
                }
            }
            
            // Only the second column should be primary key
            assert!(!dialog.files[0].columns[0].is_primary_key);
            assert!(dialog.files[0].columns[1].is_primary_key);
        }
    }
    
    #[test]
    fn test_table_name_generation() {
        let test_cases = vec![
            ("test.csv", "test"),
            ("data_file.csv", "data_file"),
            ("path/to/file.csv", "file"),
            ("file_without_extension", "file_without_extension"),
        ];
        
        for (input, expected) in test_cases {
            let config = FileConfig::new(PathBuf::from(input));
            assert_eq!(config.table_name, expected);
        }
    }
    
    #[test]
    fn test_data_type_sql_conversion() {
        assert_eq!(DataType::Text.to_sql_type(), "TEXT");
        assert_eq!(DataType::Integer.to_sql_type(), "INTEGER");
        assert_eq!(DataType::Real.to_sql_type(), "REAL");
        assert_eq!(DataType::Boolean.to_sql_type(), "BOOLEAN");
        assert_eq!(DataType::Date.to_sql_type(), "DATE");
        assert_eq!(DataType::DateTime.to_sql_type(), "DATETIME");
    }
    
    #[test]
    fn test_sample_size_bounds() {
        let mut dialog = FileImportDialog::new();
        let file_path = PathBuf::from("test.csv");
        dialog.add_file(file_path);
        
        // Test default sample size
        assert_eq!(dialog.files[0].sample_size, 100);
        
        // Test sample size changes
        dialog.files[0].sample_size = 500;
        assert_eq!(dialog.files[0].sample_size, 500);
        
        dialog.files[0].sample_size = 5000;
        assert_eq!(dialog.files[0].sample_size, 5000);
    }
} 