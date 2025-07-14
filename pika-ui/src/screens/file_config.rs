//! File configuration screen for importing CSV files.
//! Matches Pebble's professional CSV import interface.

use crate::state::AppState;
use egui::{Color32, RichText, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use pika_core::types::{ColumnInfo, TableInfo};
use std::collections::HashMap;
use std::path::PathBuf;
use egui_plot::PlotPoint;

// Standalone helper function for column type inference
fn infer_column_type(rows: &[&Vec<String>], col_idx: usize) -> ColumnType {
    let mut all_integer = true;
    let mut all_float = true;
    let mut all_boolean = true;
    
    for row in rows {
        if let Some(value) = row.get(col_idx) {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Check if it's a boolean
            if !matches!(trimmed.to_lowercase().as_str(), "true" | "false" | "yes" | "no" | "0" | "1") {
                all_boolean = false;
            }
            
            // Check if it's an integer
            if trimmed.parse::<i64>().is_err() {
                all_integer = false;
            }
            
            // Check if it's a float
            if trimmed.parse::<f64>().is_err() {
                all_float = false;
            }
            
            if !all_integer && !all_float && !all_boolean {
                return ColumnType::Text;
            }
        }
    }
    
    if all_boolean {
        ColumnType::Boolean
    } else if all_integer {
        ColumnType::Integer
    } else if all_float {
        ColumnType::Real
    } else {
        ColumnType::Text
    }
}

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
    Boolean,
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
            ColumnType::Boolean => "Boolean",
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
            ColumnType::Boolean => "BOOLEAN",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ColumnType::Integer => "Integer",
            ColumnType::Text => "Text",
            ColumnType::Real => "Real",
            ColumnType::Blob => "Blob",
            ColumnType::Date => "Date",
            ColumnType::DateTime => "DateTime",
            ColumnType::Boolean => "Boolean",
        }
    }
}

#[derive(Debug, Clone)]
struct PreviewData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    total_rows: usize,
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
                ui.style_mut().spacing.item_spacing = Vec2::new(10.0, 10.0);
                
                // Wrap everything in a unique ID based on screen instance
                ui.push_id("file_config_main", |ui| {
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

                    // Main content
                    ui.columns(2, |columns| {
                        // Left column
                        columns[0].push_id("left_column", |ui| {
                            self.render_database_config(ui);
                            
                            if !self.files.is_empty() {
                                ui.add_space(10.0);
                                self.render_file_configuration(ui);
                            }
                        });

                        // Right column - data preview
                        columns[1].push_id("right_column", |ui| {
                            if !self.files.is_empty() {
                                ui.heading(RichText::new("Data Preview").color(Color32::from_gray(200)));
                                ui.separator();
                                self.render_data_preview(ui);
                            }
                        });
                    });

                    // Bottom actions
                    ui.add_space(20.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Cancel").size(16.0))
                            .on_hover_text("Cancel and return to canvas")
                            .clicked() 
                        {
                            state.view_mode = crate::state::ViewMode::Canvas;
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_proceed = !self.database_path.display().to_string().is_empty() 
                                && self.files.iter().any(|f| f.columns.iter().any(|c| c.include));
                            
                            ui.add_enabled_ui(can_proceed, |ui| {
                                if ui.button(RichText::new("Create Database →").color(Color32::from_rgb(100, 200, 100)).size(16.0))
                                    .on_hover_text("Create database with selected configuration")
                                    .clicked() 
                                {
                                    result = Some(self.create_database());
                                    state.view_mode = crate::state::ViewMode::Canvas;
                                }
                            });
                        });
                    });

                    if let Some(error) = &self.error_message {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), error);
                    }
                });
            });

        result
    }

    fn render_database_config(&mut self, ui: &mut Ui) {
        ui.heading(RichText::new("Database Configuration").color(Color32::from_gray(200)));
        ui.separator();
        
        // Database path
        ui.horizontal(|ui| {
            ui.label(RichText::new("Database Path:").color(Color32::from_gray(200)));
            ui.add_space(10.0);
            
            let available_width = ui.available_width() - 100.0;
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
    }

    fn render_file_configuration(&mut self, ui: &mut Ui) {
        self.render_configuration_panel(ui);
    }

    fn render_configuration_panel(&mut self, ui: &mut Ui) {
        if self.files.is_empty() {
            return;
        }

        // CSV File selector
        ui.horizontal(|ui| {
            ui.label(RichText::new("CSV File:").color(Color32::from_gray(200)));
            ui.add_space(10.0);
            
            if self.current_file_index >= self.files.len() && !self.files.is_empty() {
                self.current_file_index = 0;
            }
            
            if let Some(current_file) = self.files.get(self.current_file_index) {
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
                            
                            if ui.selectable_label(self.current_file_index == idx, name).clicked() {
                                self.current_file_index = idx;
                            }
                        }
                    });
                    
                    // Handle preview loading after file selection
                    if self.current_file_index < self.files.len() {
                        let needs_preview = self.files.get(self.current_file_index)
                            .map(|f| f.preview_data.is_none())
                            .unwrap_or(false);
                            
                        if needs_preview {
                            // Extract values we need with bounds checking
                            if let Some(file) = self.files.get(self.current_file_index) {
                                let file_path = file.path.clone();
                                let header_row = file.header_row;
                                let delimiter = file.delimiter.clone();
                                let sample_size = file.sample_size;
                                
                                // Check cache first
                                if let Some(cached) = self.preview_cache.get(&file_path).cloned() {
                                    if let Some(file) = self.files.get_mut(self.current_file_index) {
                                        file.preview_data = Some(cached.clone());
                                        // Update columns if empty
                                        if file.columns.is_empty() {
                                            let data_rows: Vec<&Vec<String>> = cached.rows.iter().collect();
                                            
                                            for (i, col_name) in cached.headers.iter().enumerate() {
                                                let inferred_type = infer_column_type(&data_rows, i);
                                                let is_id_column = col_name.to_lowercase() == "id";
                                                
                                                let config = ColumnConfig {
                                                    name: col_name.clone(),
                                                    data_type: inferred_type,
                                                    include: true,
                                                    is_primary_key: false, // Don't set primary key by default
                                                    not_null: is_id_column,
                                                    unique: is_id_column,
                                                    create_index: false,
                                                };
                                                
                                                file.columns.push(config);
                                            }
                                        }
                                    }
                                } else {
                                    // Load preview data
                                    if let Ok(preview) = self.load_preview_data(&file_path, header_row, &delimiter, sample_size) {
                                        self.preview_cache.insert(file_path.clone(), preview.clone());
                                        if let Some(file) = self.files.get_mut(self.current_file_index) {
                                            file.preview_data = Some(preview.clone());
                                            // Update columns if empty
                                            if file.columns.is_empty() {
                                                let data_rows: Vec<&Vec<String>> = preview.rows.iter().collect();
                                                
                                                for (i, col_name) in preview.headers.iter().enumerate() {
                                                    let inferred_type = infer_column_type(&data_rows, i);
                                                    let is_id_column = col_name.to_lowercase() == "id";
                                                    
                                                    let config = ColumnConfig {
                                                        name: col_name.clone(),
                                                        data_type: inferred_type,
                                                        include: true,
                                                        is_primary_key: false, // Don't set primary key by default
                                                        not_null: is_id_column,
                                                        unique: is_id_column,
                                                        create_index: false,
                                                    };
                                                    
                                                    file.columns.push(config);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
            }
        });

        // Add Files button  
        ui.horizontal(|ui| {
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
            ui.group(|ui| {
                ui.heading(RichText::new("Delimiter:").color(Color32::from_gray(200)));
                ui.horizontal_wrapped(|ui| {
                    ui.push_id(format!("delimiter_{}", idx), |ui| {
                        ui.radio_value(&mut file.delimiter, Delimiter::Comma, "• Comma");
                        ui.radio_value(&mut file.delimiter, Delimiter::Tab, "• Tab");
                        ui.radio_value(&mut file.delimiter, Delimiter::Semicolon, "• Semicolon");
                        ui.radio_value(&mut file.delimiter, Delimiter::Pipe, "• Pipe");
                    });
                });
            });

            ui.add_space(10.0);

            // Null Values
            ui.group(|ui| {
                ui.heading(RichText::new("Null Values").color(Color32::from_gray(200)));
                ui.label(RichText::new("Values to treat as NULL:").color(Color32::from_gray(180)));
                ui.push_id(format!("null_values_{}", idx), |ui| {
                    ui.checkbox(&mut file.null_values.empty_string, "☐ [empty string]");
                    ui.checkbox(&mut file.null_values.null_text, "☐ NULL");
                    ui.checkbox(&mut file.null_values.lowercase_null, "☐ null");
                    ui.checkbox(&mut file.null_values.na, "☐ N/A");
                });
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
        if file_idx < self.files.len() {
            self.render_column_table(ui, file_idx);
        }
    }

    fn render_column_table(&mut self, ui: &mut Ui, file_idx: usize) {
        if let Some(file) = self.files.get_mut(file_idx) {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            
            // Wrap table in unique ID scope
            ui.scope(|ui| {
                ui.push_id(format!("column_table_{}", file_idx), |ui| {
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
                                ui.push_id(row_index, |ui| {
                                    ui.checkbox(&mut file.columns[row_index].include, "");
                                });
                            });
                            
                            row.col(|ui| {
                                ui.label(&file.columns[row_index].name);
                            });
                            
                            row.col(|ui| {
                                egui::ComboBox::from_id_source(format!("type_combo_{}_{}", file_idx, row_index))
                                    .selected_text(file.columns[row_index].data_type.display_name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Text, "Text");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Integer, "Integer");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Real, "Real");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Blob, "Blob");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Date, "Date");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::DateTime, "DateTime");
                                        ui.selectable_value(&mut file.columns[row_index].data_type, ColumnType::Boolean, "Boolean");
                                    });
                            });
                            
                            row.col(|ui| {
                                ui.push_id(format!("pk_{}_{}", file_idx, row_index), |ui| {
                                    if ui.radio(file.columns[row_index].is_primary_key, "").clicked() {
                                        // First unset all other primary keys
                                        for (i, col) in file.columns.iter_mut().enumerate() {
                                            if i != row_index {
                                                col.is_primary_key = false;
                                            }
                                        }
                                        // Then toggle this one
                                        file.columns[row_index].is_primary_key = !file.columns[row_index].is_primary_key;
                                    }
                                });
                            });
                            
                            row.col(|ui| {
                                ui.push_id(format!("notnull_{}_{}", file_idx, row_index), |ui| {
                                    ui.checkbox(&mut file.columns[row_index].not_null, "");
                                });
                            });
                            
                            row.col(|ui| {
                                ui.push_id(format!("unique_{}_{}", file_idx, row_index), |ui| {
                                    ui.checkbox(&mut file.columns[row_index].unique, "");
                                });
                            });
                            
                            row.col(|ui| {
                                ui.push_id(format!("index_{}_{}", file_idx, row_index), |ui| {
                                    ui.checkbox(&mut file.columns[row_index].create_index, "");
                                });
                            });
                        });
                    });
                });
            });
        }
    }

    fn render_data_preview(&mut self, ui: &mut Ui) {
        if let Some(file) = self.files.get(self.current_file_index) {
            if let Some(preview) = &file.preview_data {
                let available_height = ui.available_height();
                let header_row = file.header_row;
                
                // Wrap table in unique ID scope
                ui.scope(|ui| {
                    ui.push_id(format!("data_preview_table_{}", self.current_file_index), |ui| {
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
                                for (row_idx, data_row) in preview.rows.iter().enumerate() {
                                    let is_header_row = row_idx == 0 && header_row == 1;
                                    
                                    body.row(18.0, |mut row| {
                                        for cell in data_row.iter() {
                                            row.col(|ui| {
                                                if is_header_row {
                                                    ui.label(RichText::new(cell).strong().color(Color32::from_rgb(120, 200, 255)));
                                                } else {
                                                    ui.label(cell);
                                                }
                                            });
                                        }
                                    });
                                }
                            });
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

        // Load preview data
        if let Ok(preview) = self.load_preview_data(&file.path, file.header_row, &file.delimiter, file.sample_size) {
            self.preview_cache.insert(file.path.clone(), preview.clone());
            file.preview_data = Some(preview);
            self.update_columns_from_preview(file);
        }
    }
    
    fn load_preview_data(&self, path: &PathBuf, header_row: usize, delimiter: &Delimiter, sample_size: usize) -> Result<PreviewData, String> {
        // Load CSV and parse preview
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
            
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err("File is empty".to_string());
        }

        let mut headers = Vec::new();
        let mut rows = Vec::new();

        // Get headers from specified row
        if header_row > 0 && header_row <= lines.len() {
            headers = lines[header_row - 1]
                .split(delimiter.as_char())
                .map(|s| s.trim().to_string())
                .collect();
        }

        // Parse rows for preview, starting AFTER the header row
        let start_row = header_row; // Start after header
        let end_row = (start_row + sample_size).min(lines.len());
        
        for idx in start_row..end_row {
            if idx < lines.len() {
                let cells: Vec<String> = lines[idx]
                    .split(delimiter.as_char())
                    .map(|s| s.trim().to_string())
                    .collect();
                rows.push(cells);
            }
        }

        // Store in cache for reuse
        let preview = PreviewData {
            headers,
            rows,
            total_rows: lines.len() - header_row,
        };
        
        Ok(preview)
    }

    fn update_columns_from_preview(&mut self, file: &mut FileConfig) {
        if let Some(preview) = &file.preview_data {
            file.columns.clear();
            
            // Infer types from data rows (all rows are now data rows, no header)
            let data_rows: Vec<&Vec<String>> = preview.rows.iter().collect();
            
            for (i, col_name) in preview.headers.iter().enumerate() {
                let inferred_type = infer_column_type(&data_rows, i);
                let is_id_column = col_name.to_lowercase() == "id";
                
                let config = ColumnConfig {
                    name: col_name.clone(),
                    data_type: inferred_type,
                    include: true,
                    is_primary_key: false, // Don't set primary key by default
                    not_null: is_id_column,
                    unique: is_id_column,
                    create_index: false,
                };

                file.columns.push(config);
            }
        }
    }

    fn infer_column_type(&self, rows: &[&Vec<String>], col_idx: usize) -> ColumnType {
        infer_column_type(rows, col_idx)
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