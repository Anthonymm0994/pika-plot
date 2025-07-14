//! File configuration screen for importing CSV files.
//! Matches Pebble's professional CSV import interface.

use crate::state::AppState;
use egui::{Color32, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use pika_core::types::{ColumnInfo, TableInfo};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileConfigScreen {
    /// List of files being configured
    files: Vec<FileConfig>,
    /// Currently selected file index
    current_file_index: usize,
    /// Database path
    database_path: PathBuf,
    /// UI state
    error_message: Option<String>,
    /// Preview data cache
    preview_cache: HashMap<PathBuf, PreviewData>,
    /// Whether we need to show the file picker on initialization
    show_file_picker_on_init: bool,
    instance_id: egui::Id,
}

#[derive(Debug, Clone)]
struct FileConfig {
    path: PathBuf,
    table_name: String,
    header_row: usize,
    sample_size: usize,
    delimiter: Delimiter,
    null_values: NullValues,
    columns: Vec<ColumnConfig>,
    preview_data: Option<PreviewData>,
}

#[derive(Debug, Clone, PartialEq)]
enum Delimiter {
    Comma,
    Tab,
    Semicolon,
    Pipe,
}

impl Delimiter {
    fn as_char(&self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Semicolon => ';',
            Delimiter::Pipe => '|',
        }
    }
}

#[derive(Debug, Clone)]
struct NullValues {
    empty_string: bool,
    null_text: bool,
    lowercase_null: bool,
    na: bool,
}

impl Default for NullValues {
    fn default() -> Self {
        Self {
            empty_string: true,
            null_text: true,
            lowercase_null: true,
            na: true,
        }
    }
}

#[derive(Debug, Clone)]
struct ColumnConfig {
    name: String,
    data_type: ColumnType,
    include: bool,
    is_primary_key: bool,
    not_null: bool,
    unique: bool,
    create_index: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Integer,
    Text,
    Real,
    Blob,
    Date,
    DateTime,
}

impl ColumnType {
    fn as_str(&self) -> &'static str {
        match self {
            ColumnType::Integer => "Integer",
            ColumnType::Text => "Text",
            ColumnType::Real => "Real",
            ColumnType::Blob => "Blob",
            ColumnType::Date => "Date",
            ColumnType::DateTime => "DateTime",
        }
    }
    
    fn to_sql(&self) -> &'static str {
        match self {
            ColumnType::Integer => "INTEGER",
            ColumnType::Text => "TEXT",
            ColumnType::Real => "REAL",
            ColumnType::Blob => "BLOB",
            ColumnType::Date => "DATE",
            ColumnType::DateTime => "DATETIME",
        }
    }
}

#[derive(Debug, Clone)]
struct PreviewData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl FileConfigScreen {
    pub fn new() -> Self {
        let default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("pika_data.db");
            
        Self {
            files: Vec::new(),
            current_file_index: 0,
            database_path: default_path,
            error_message: None,
            preview_cache: HashMap::new(),
            show_file_picker_on_init: true,
            instance_id: egui::Id::new("file_config_instance").with(std::time::SystemTime::now()),
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        let table_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("table")
            .to_string();

        let mut file_config = FileConfig {
            path: path.clone(),
            table_name,
            header_row: 1,
            sample_size: 1000,
            delimiter: Delimiter::Comma,
            null_values: NullValues::default(),
            columns: Vec::new(),
            preview_data: None,
        };

        // Load preview data
        self.load_preview_for_file(&mut file_config);
        self.files.push(file_config);
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState) -> Option<Vec<TableInfo>> {
        let mut result = None;

        // Show file picker immediately on first show if no files loaded
        if self.show_file_picker_on_init && self.files.is_empty() {
            self.show_file_picker_on_init = false;
            if let Some(paths) = rfd::FileDialog::new()
                .add_filter("CSV files", &["csv", "tsv", "txt"])
                .pick_files()
            {
                for path in paths {
                    self.add_file(path);
                }
            }
            // If no files selected, go back to canvas
            if self.files.is_empty() {
                state.view_mode = crate::state::ViewMode::Canvas;
                return None;
            }
        }

        // Dark theme window
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(30, 30, 30))
            .inner_margin(egui::vec2(20.0, 20.0));

        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                ui.push_id(self.instance_id.with("file_config_screen"), |ui| {
                    // Title bar
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new("Create Database from CSV").color(Color32::WHITE).size(18.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("✕").size(20.0)).clicked() {
                                state.view_mode = crate::state::ViewMode::Canvas;
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Database path
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Database Path:").color(Color32::from_gray(200)));
                        ui.add_space(10.0);
                        
                        let available_width = ui.available_width() - 100.0; // Leave space for Browse button
                        let path_display = self.database_path.to_string_lossy().to_string();
                        ui.add(egui::TextEdit::singleline(&mut path_display.clone())
                            .desired_width(available_width.min(400.0))
                            .interactive(false)
                            .text_color(Color32::from_gray(180)));
                        
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                                .save_file()
                            {
                                self.database_path = path;
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // Use available space for the main content
                    let available_size = ui.available_size();
                    let left_width = (available_size.x * 0.45).min(500.0).max(350.0);
                    let right_width = available_size.x - left_width - 10.0;

                    ui.horizontal_top(|ui| {
                        // Left column - Configuration
                        ui.allocate_ui(egui::vec2(left_width, available_size.y - 50.0), |ui| {
                            ui.vertical(|ui| {
                                self.render_configuration_panel(ui);
                            });
                        });

                        ui.add_space(10.0);

                        // Right column - Preview
                        ui.allocate_ui(egui::vec2(right_width, available_size.y - 50.0), |ui| {
                            ui.group(|ui| {
                                ui.heading(RichText::new("Data Preview").color(Color32::from_gray(200)));
                                ui.separator();
                                self.render_data_preview(ui);
                            });
                        });
                    });

                    ui.add_space(10.0);

                    // Bottom buttons
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Cancel").size(14.0)).clicked() {
                            state.view_mode = crate::state::ViewMode::Canvas;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let enabled = !self.files.is_empty() && self.files.iter().any(|f| 
                                f.columns.iter().any(|c| c.include)
                            );
                            
                            let button = egui::Button::new(
                                RichText::new(format!("✓ Create Database with {} Tables", self.files.len()))
                                    .size(14.0)
                                    .color(Color32::WHITE)
                            )
                            .fill(if enabled { Color32::from_rgb(34, 139, 34) } else { Color32::from_gray(60) });

                            if ui.add_enabled(enabled, button).clicked() {
                                result = Some(self.create_database());
                                state.view_mode = crate::state::ViewMode::Canvas;
                            }
                        });
                    });

                    // Error display
                    if let Some(error) = &self.error_message {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), error);
                    }
                });
            });

        result
    }

    fn render_configuration_panel(&mut self, ui: &mut Ui) {
        if self.files.is_empty() {
            return;
        }

        // CSV File selector
        ui.horizontal(|ui| {
            ui.label(RichText::new("CSV File:").color(Color32::from_gray(200)));
            ui.add_space(10.0);
            
            let current_file = &self.files[self.current_file_index];
            let file_name = current_file.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            egui::ComboBox::from_id_source("csv_file_selector")
                .selected_text(file_name)
                .show_ui(ui, |ui| {
                    for (idx, file) in self.files.iter().enumerate() {
                        let name = file.path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        if ui.selectable_value(&mut self.current_file_index, idx, name).clicked() {
                            // File selection changed
                        }
                    }
                });
            
            ui.add_space(10.0);
            if ui.button("Add Files...").clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("CSV files", &["csv", "tsv", "txt"])
                    .pick_files()
                {
                    for path in paths {
                        self.add_file(path);
                    }
                }
            }
        });

        ui.label(RichText::new(format!("Files to import: {} total, {} configured", 
            self.files.len(),
            self.files.iter().filter(|f| f.columns.iter().any(|c| c.include)).count()
        )).color(Color32::from_gray(150)).small());

        ui.add_space(10.0);

        // Get values before the mutable borrow
        let idx = self.current_file_index;
        let mut need_reload = false;
        
        // Track previous values for change detection
        let (prev_header_row, prev_delimiter) = if let Some(file) = self.files.get(self.current_file_index) {
            (file.header_row, file.delimiter.clone())
        } else {
            (1, Delimiter::Comma)
        };
        
        if let Some(file) = self.files.get_mut(self.current_file_index) {
            // Table name
            ui.horizontal(|ui| {
                ui.label(RichText::new("Table Name:").color(Color32::from_gray(200)));
                ui.add_space(10.0);
                ui.text_edit_singleline(&mut file.table_name);
            });

            ui.add_space(10.0);

            // Header Configuration
            ui.group(|ui| {
                ui.heading(RichText::new("Header Configuration").color(Color32::from_gray(200)));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Header Row:").color(Color32::from_gray(180)));
                    let response = ui.add(egui::DragValue::new(&mut file.header_row)
                        .clamp_range(1..=50));
                    ui.label(RichText::new("(1-50)").color(Color32::from_gray(128)));
                    
                    // Check if header row changed
                    if response.changed() {
                        need_reload = true;
                    }
                });
                ui.label(RichText::new("The green highlighted row in the preview is your header")
                    .color(Color32::from_gray(150))
                    .small());
            });

            ui.add_space(10.0);

            // Sample size
            ui.horizontal(|ui| {
                ui.label(RichText::new("Sample Size:").color(Color32::from_gray(200)));
                ui.add(egui::DragValue::new(&mut file.sample_size)
                    .clamp_range(10..=10000)
                    .speed(10));
                ui.label("rows");
                if ui.button("↻ Resample").clicked() {
                    need_reload = true;
                }
            });

            ui.add_space(10.0);

            // Delimiter
            ui.label(RichText::new("Delimiter:").color(Color32::from_gray(200)));
            ui.horizontal(|ui| {
                let mut delimiter_changed = false;
                
                if ui.radio_value(&mut file.delimiter, Delimiter::Comma, "⚪ Comma").changed() {
                    delimiter_changed = true;
                }
                if ui.radio_value(&mut file.delimiter, Delimiter::Tab, "⚪ Tab").changed() {
                    delimiter_changed = true;
                }
                if ui.radio_value(&mut file.delimiter, Delimiter::Semicolon, "⚪ Semicolon").changed() {
                    delimiter_changed = true;
                }
                if ui.radio_value(&mut file.delimiter, Delimiter::Pipe, "⚪ Pipe").changed() {
                    delimiter_changed = true;
                }
                
                // Check if delimiter changed
                if delimiter_changed && file.delimiter != prev_delimiter {
                    need_reload = true;
                }
            });

            ui.add_space(10.0);

            // Null Values
            ui.group(|ui| {
                ui.heading(RichText::new("Null Values").color(Color32::from_gray(200)));
                ui.label(RichText::new("Values to treat as NULL:").color(Color32::from_gray(180)));
                ui.checkbox(&mut file.null_values.empty_string, "☐ [empty string]");
                ui.checkbox(&mut file.null_values.null_text, "☐ NULL");
                ui.checkbox(&mut file.null_values.lowercase_null, "☐ null");
                ui.checkbox(&mut file.null_values.na, "☐ N/A");
            });
        }

        // Handle reload outside of the mutable borrow
        if need_reload {
            if let Some(file) = self.files.get(idx) {
                // Clear cache for this file to force fresh reload
                self.preview_cache.remove(&file.path);
            }
            if let Some(mut file_clone) = self.files.get(idx).cloned() {
                self.load_preview_for_file(&mut file_clone);
                self.files[idx] = file_clone;
            }
        }

        // Column Selection section
        if let Some(file) = self.files.get_mut(self.current_file_index) {
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.heading(RichText::new("Column Selection").color(Color32::from_gray(200)));
                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() {
                        for col in &mut file.columns {
                            col.include = true;
                        }
                    }
                    if ui.button("Deselect All").clicked() {
                        for col in &mut file.columns {
                            col.include = false;
                        }
                    }
                    
                    ui.add_space(20.0);
                    ui.label(format!("{}/{} selected", 
                        file.columns.iter().filter(|c| c.include).count(),
                        file.columns.len()
                    ));
                });
            });
        }

        // Column configuration table - rendered outside the file borrow
        let file_idx = self.current_file_index;
        ui.push_id(self.instance_id.with("column_table_section"), |ui| {
            self.render_column_table(ui, file_idx);
        });
    }

    fn render_column_table(&mut self, ui: &mut Ui, file_idx: usize) {
        if let Some(file) = self.files.get_mut(file_idx) {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(40.0)) // Include
                .column(Column::auto()) // Column name
                .column(Column::exact(100.0)) // Type
                .column(Column::exact(35.0)) // PK
                .column(Column::exact(70.0)) // Not Null
                .column(Column::exact(60.0)) // Unique
                .column(Column::exact(50.0)) // Index
                .min_scrolled_height(200.0)
                .max_scroll_height(300.0);

            table.body(|body| {
                let row_height = text_height + 8.0;
                let num_cols = file.columns.len();
                
                body.rows(row_height, num_cols, |mut row| {
                    let row_index = row.index();
                    
                    row.col(|ui| {
                        ui.checkbox(&mut file.columns[row_index].include, "");
                    });
                    
                    row.col(|ui| {
                        ui.label(&file.columns[row_index].name);
                    });
                    
                    row.col(|ui| {
                        let current_type = file.columns[row_index].data_type;
                        egui::ComboBox::from_id_source(format!("type_{}_{}", file_idx, row_index))
                            .selected_text(current_type.as_str())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Integer, "Integer");
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Text, "Text");
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Real, "Real");
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Blob, "Blob");
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Date, "Date");
                                ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::DateTime, "DateTime");
                            });
                    });
                    
                    row.col(|ui| {
                        let was_pk = file.columns[row_index].is_primary_key;
                        if ui.checkbox(&mut file.columns[row_index].is_primary_key, "").changed() {
                            if file.columns[row_index].is_primary_key && !was_pk {
                                // Unset other PKs - only one column can be primary key
                                for (idx, col) in file.columns.iter_mut().enumerate() {
                                    if idx != row_index {
                                        col.is_primary_key = false;
                                    }
                                }
                            }
                        }
                    });
                    
                    row.col(|ui| {
                        ui.checkbox(&mut file.columns[row_index].not_null, "");
                    });
                    
                    row.col(|ui| {
                        ui.checkbox(&mut file.columns[row_index].unique, "");
                    });
                    
                    row.col(|ui| {
                        ui.checkbox(&mut file.columns[row_index].create_index, "");
                    });
                });
            });
        }
    }

    fn render_data_preview(&mut self, ui: &mut Ui) {
        if let Some(file) = self.files.get(self.current_file_index) {
            if let Some(preview) = &file.preview_data {
                let available_height = ui.available_height();
                
                ui.push_id(self.instance_id.with("data_preview_section"), |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(Column::auto(), preview.headers.len())
                        .max_scroll_height(available_height)
                        .header(20.0, |mut header| {
                            for col_name in &preview.headers {
                                header.col(|ui| {
                                    ui.label(RichText::new(col_name).strong());
                                });
                            }
                        })
                        .body(|mut body| {
                            for (idx, data_row) in preview.rows.iter().enumerate() {
                                body.row(18.0, |mut row| {
                                    for cell in data_row.iter() {
                                        row.col(|ui| {
                                            ui.label(cell);
                                        });
                                    }
                                });
                            }
                        });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No preview available");
                });
            }
        }
    }

    fn load_preview_for_file(&mut self, file: &mut FileConfig) {
        // Check cache first
        if let Some(cached) = self.preview_cache.get(&file.path) {
            file.preview_data = Some(cached.clone());
            self.update_columns_from_preview(file);
            return;
        }

        // Load CSV and parse preview
        match std::fs::read_to_string(&file.path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if lines.is_empty() {
                    self.error_message = Some("File is empty".to_string());
                    return;
                }

                let mut headers = Vec::new();
                let mut rows = Vec::new();

                // Get headers from specified row
                if file.header_row > 0 && file.header_row <= lines.len() {
                    headers = lines[file.header_row - 1]
                        .split(file.delimiter.as_char())
                        .map(|s| s.trim().to_string())
                        .collect();
                }

                // Parse rows for preview, starting AFTER the header row
                let start_row = file.header_row; // Start after header
                let end_row = (start_row + file.sample_size).min(lines.len());
                
                for idx in start_row..end_row {
                    if idx < lines.len() {
                        let cells: Vec<String> = lines[idx]
                            .split(file.delimiter.as_char())
                            .map(|s| s.trim().to_string())
                            .collect();
                        rows.push(cells);
                    }
                }

                let preview_data = PreviewData {
                    headers: headers.clone(),
                    rows,
                };

                // Cache the preview
                self.preview_cache.insert(file.path.clone(), preview_data.clone());
                file.preview_data = Some(preview_data);
                self.update_columns_from_preview(file);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to read file: {}", e));
            }
        }
    }

    fn update_columns_from_preview(&mut self, file: &mut FileConfig) {
        if let Some(preview) = &file.preview_data {
            file.columns.clear();
            
            // Infer types from data rows (all rows are now data rows, no header)
            let data_rows: Vec<&Vec<String>> = preview.rows.iter().collect();
            
            for (i, header) in preview.headers.iter().enumerate() {
                let column_type = self.infer_column_type(&data_rows, i);
                let is_id_column = header.to_lowercase() == "id";
                
                file.columns.push(ColumnConfig {
                    name: header.clone(),
                    data_type: column_type,
                    include: true,
                    is_primary_key: is_id_column, // Set id column as primary key
                    not_null: false,
                    unique: false,
                    create_index: false,
                });
            }
        }
    }

    fn infer_column_type(&self, rows: &[&Vec<String>], col_idx: usize) -> ColumnType {
        let mut all_integer = true;
        let mut all_real = true;
        let mut has_values = false;

        for row in rows {
            if let Some(value) = row.get(col_idx) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    has_values = true;
                    
                    // Check if it's an integer
                    if trimmed.parse::<i64>().is_err() {
                        all_integer = false;
                    }
                    
                    // Check if it's a real number
                    if trimmed.parse::<f64>().is_err() {
                        all_real = false;
                    }
                }
            }
        }

        if !has_values || (!all_integer && !all_real) {
            ColumnType::Text
        } else if all_integer {
            ColumnType::Integer
        } else {
            ColumnType::Real
        }
    }

    fn create_database(&mut self) -> Vec<TableInfo> {
        let mut table_infos = Vec::new();

        // For each file, create a TableInfo
        for file in &self.files {
            let columns: Vec<ColumnInfo> = file.columns.iter()
                .filter(|col| col.include)
                .map(|col| ColumnInfo {
                    name: col.name.clone(),
                    data_type: col.data_type.to_sql().to_string(),
                    nullable: !col.not_null && !col.is_primary_key,
                })
                .collect();

            if !columns.is_empty() {
                // Create preview data from the loaded data
                let preview_data = file.preview_data.as_ref().map(|p| {
                    pika_core::types::TablePreview {
                        rows: p.rows
                            .iter()
                            .take(25) // Take first 25 rows for preview
                            .cloned()
                            .collect(),
                        current_page: 0,
                        rows_per_page: 25,
                    }
                });
                
                let table_info = TableInfo {
                    name: file.table_name.clone(),
                    source_path: Some(file.path.clone()),
                    row_count: file.preview_data.as_ref().map(|p| p.rows.len()),
                    columns,
                    preview_data,
                };
                
                table_infos.push(table_info);
            }
        }

        // Here you would normally create the actual database and import the data
        // For now, we just return the table information
        
        table_infos
    }
}

impl Default for FileConfigScreen {
    fn default() -> Self {
        Self::new()
    }
} 