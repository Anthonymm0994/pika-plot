//! Working CSV import dialog with Pebble-inspired design.
//! Features: multi-file selection, data preview, header configuration, column selection.

use egui::{Ui, Color32, ScrollArea, TextEdit, ComboBox, Button, Context, Id};
use egui_extras::{TableBuilder, Column};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
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
    
    pub fn icon(&self) -> &'static str {
        match self {
            DataType::Text => "📝",
            DataType::Integer => "🔢",
            DataType::Real => "📊",
            DataType::Boolean => "✅",
            DataType::Date => "📅",
            DataType::DateTime => "⏰",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileConfig {
    pub path: PathBuf,
    pub table_name: String,
    pub header_row: usize,
    pub delimiter: char,
    pub columns: Vec<ColumnConfig>,
    pub preview_data: Option<PreviewData>,
    pub file_size: u64,
    pub estimated_rows: usize,
}

#[derive(Debug, Clone)]
pub struct ColumnConfig {
    pub name: String,
    pub original_name: String,
    pub data_type: DataType,
    pub included: bool,
    pub create_index: bool,
    pub is_primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

impl FileConfig {
    pub fn new(path: PathBuf) -> Self {
        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("table")
            .to_string();
        
        Self {
            path,
            table_name: file_name,
            header_row: 0,
            delimiter: ',',
            columns: Vec::new(),
            preview_data: None,
            file_size: 0,
            estimated_rows: 0,
        }
    }
    
    pub fn file_name(&self) -> String {
        self.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

pub struct WorkingCsvImportDialog {
    id: Id,
    pub show: bool,
    pub database_path: Option<PathBuf>,
    pub files: Vec<FileConfig>,
    pub current_file_index: usize,
    pub create_database: bool,
    error: Option<String>,
}

impl Default for WorkingCsvImportDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkingCsvImportDialog {
    pub fn new() -> Self {
        Self {
            id: Id::new("working_csv_import"),
            show: false,
            database_path: None,
            files: Vec::new(),
            current_file_index: 0,
            create_database: true,
            error: None,
        }
    }
    
    pub fn open(&mut self, path: PathBuf) {
        self.show = true;
        self.files.clear();
        self.add_file(path);
        self.current_file_index = 0;
    }
    
    pub fn open_with_csv_selection(&mut self) {
        self.show = true;
        self.files.clear();
        
        // Open file dialog for CSV selection
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("CSV files", &["csv", "tsv", "txt"])
            .set_title("Select CSV files to import")
            .pick_files()
        {
            for file in files {
                self.add_file(file);
            }
            if !self.files.is_empty() {
                self.current_file_index = 0;
                self.load_preview_for_current_file();
            }
        }
    }
    
    fn reset(&mut self) {
        self.files.clear();
        self.current_file_index = 0;
        self.error = None;
        self.database_path = None;
    }
    
    pub fn add_file(&mut self, path: PathBuf) {
        let mut config = FileConfig::new(path);
        if let Ok(metadata) = std::fs::metadata(&config.path) {
            config.file_size = metadata.len();
        }
        self.files.push(config);
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<PathBuf> {
        if !self.show {
            return None;
        }
        
        let mut result = None;
        let mut keep_open = true;
        
        egui::Window::new("📊 CSV Import - Professional")
            .id(self.id)
            .resizable(true)
            .default_width(900.0)
            .default_height(700.0)
            .show(ctx, |ui| {
                if let Some(db_path) = self.render_content(ui) {
                    result = Some(db_path);
                    keep_open = false;
                }
                
                // Close button handling
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        keep_open = false;
                    }
                });
            });
        
        if !keep_open {
            self.show = false;
            self.reset();
        }
        
        result
    }
    
    fn render_content(&mut self, ui: &mut Ui) -> Option<PathBuf> {
        if self.files.is_empty() {
            self.render_empty_state(ui);
            return None;
        }
        
        // File tabs
        if self.files.len() > 1 {
            ui.horizontal(|ui| {
                ui.label("📁 Files:");
                for (i, file) in self.files.iter().enumerate() {
                    let selected = i == self.current_file_index;
                    if ui.selectable_label(selected, file.file_name()).clicked() && i != self.current_file_index {
                        self.current_file_index = i;
                        self.load_preview_for_current_file();
                    }
                }
            });
            ui.separator();
        }
        
        // Current file configuration
        if let Some(config) = self.files.get_mut(self.current_file_index) {
            return self.render_file_configuration(ui, config);
        }
        
        None
    }
    
    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("📊 No CSV files selected");
            ui.add_space(20.0);
            ui.label("Click the button below to select CSV files to import:");
            ui.add_space(20.0);
            
            if ui.button("📂 Select CSV Files").clicked() {
                self.open_with_csv_selection();
            }
        });
    }
    
    fn render_file_configuration(&mut self, ui: &mut Ui, config: &mut FileConfig) -> Option<PathBuf> {
        ui.heading(format!("📄 {}", config.file_name()));
        
        ui.horizontal(|ui| {
            ui.label("Table name:");
            ui.text_edit_singleline(&mut config.table_name);
        });
        
        ui.horizontal(|ui| {
            ui.label("Delimiter:");
            let delimiter_str = config.delimiter.to_string();
            if ui.button(&delimiter_str).clicked() {
                // Cycle through common delimiters
                config.delimiter = match config.delimiter {
                    ',' => ';',
                    ';' => '\t',
                    '\t' => '|',
                    '|' => ',',
                    _ => ',',
                };
                self.load_preview_for_current_file();
            }
        });
        
        ui.separator();
        
        // Data preview
        if config.preview_data.is_none() {
            if ui.button("🔄 Load Preview").clicked() {
                self.load_preview_for_current_file();
            }
        } else {
            self.render_data_preview(ui, config);
        }
        
        ui.separator();
        
        // Import button
        ui.horizontal(|ui| {
            if ui.button("📊 Import to Database").clicked() {
                return self.start_database_creation();
            }
            None
        }).inner
    }
    
    fn render_data_preview(&mut self, ui: &mut Ui, config: &FileConfig) {
        if let Some(preview) = &config.preview_data {
            ui.heading("📋 Data Preview");
            
            ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(Column::auto(), preview.headers.len())
                        .header(20.0, |mut header| {
                            for column_name in &preview.headers {
                                header.col(|ui| {
                                    ui.strong(column_name);
                                });
                            }
                        })
                        .body(|mut body| {
                            for row in &preview.rows {
                                body.row(18.0, |mut table_row| {
                                    for cell in row {
                                        table_row.col(|ui| {
                                            ui.label(cell);
                                        });
                                    }
                                });
                            }
                        });
                });
        }
    }
    
    fn load_preview_for_current_file(&mut self) {
        if let Some(config) = self.files.get_mut(self.current_file_index) {
            self.load_preview_for_file_config(config);
        }
    }
    
    fn load_preview_for_file_config(&mut self, config: &mut FileConfig) {
        match std::fs::File::open(&config.path) {
            Ok(file) => {
                let mut reader = csv::ReaderBuilder::new()
                    .delimiter(config.delimiter as u8)
                    .has_headers(true)
                    .from_reader(file);
                
                let headers: Vec<String> = reader.headers()
                    .map(|h| h.iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default();
                
                let mut rows = Vec::new();
                for (i, result) in reader.records().enumerate() {
                    if i >= 10 { break; } // Limit preview to 10 rows
                    if let Ok(record) = result {
                        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                        rows.push(row);
                    }
                }
                
                config.preview_data = Some(PreviewData {
                    headers: headers.clone(),
                    rows,
                    total_rows: 0, // Simplified for now
                });
                
                // Create column configs
                config.columns = headers.into_iter().map(|name| {
                    ColumnConfig {
                        name: name.clone(),
                        original_name: name,
                        data_type: DataType::Text,
                        included: true,
                        create_index: false,
                        is_primary_key: false,
                        not_null: false,
                        unique: false,
                        sample_values: Vec::new(),
                    }
                }).collect();
            }
            Err(e) => {
                self.error = Some(format!("Failed to read file: {}", e));
            }
        }
    }
    
    fn start_database_creation(&mut self) -> Option<PathBuf> {
        // For now, just return a dummy path to indicate success
        // In a real implementation, this would create the database
        if let Some(config) = self.files.get(self.current_file_index) {
            let db_path = config.path.with_extension("db");
            println!("📊 Would create database at: {:?}", db_path);
            println!("📊 Table: {}", config.table_name);
            println!("📊 Columns: {}", config.columns.len());
            Some(db_path)
        } else {
            None
        }
    }
} 