//! File configuration screen for importing CSV files.
//! Matches Pebble's professional CSV import interface.

use crate::state::AppState;
use egui::{Color32, RichText, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use pika_core::types::{ColumnInfo, TableInfo};
use std::collections::HashMap;
use std::path::PathBuf;

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

        // Use CentralPanel like Pebble does
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar
            ui.horizontal(|ui| {
                ui.heading("Create Database from CSV");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        state.view_mode = crate::state::ViewMode::Canvas;
                    }
                });
            });
            ui.separator();
            
            // Bottom panel for buttons
            egui::TopBottomPanel::bottom("bottom_buttons")
                .show_inside(ui, |ui| {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            state.view_mode = crate::state::ViewMode::Canvas;
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_proceed = !self.database_path.display().to_string().is_empty() 
                                && !self.files.is_empty() 
                                && self.files.iter().all(|f| !f.table_name.is_empty() && 
                                    f.columns.iter().any(|c| c.include));
                            
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
                            
                            if ui.add_enabled(can_proceed, create_button).clicked() {
                                result = Some(self.create_database());
                                state.view_mode = crate::state::ViewMode::Canvas;
                            }
                            
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("💡 Configure each file's import settings before creating the database")
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(150))
                            );
                        });
                    });
                    ui.add_space(5.0);
                });
            
            // Main content in central panel
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    // Database path section
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Database Path:");
                            ui.label(self.database_path.display().to_string());
                            
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                                    .set_title("Save database as...")
                                    .save_file()
                                {
                                    self.database_path = path;
                                }
                            }
                        });
                    });
                    
                    // Error display
                    if let Some(error) = &self.error_message.clone() {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("❌ {}", error));
                            if ui.small_button("✖").clicked() {
                                self.error_message = None;
                            }
                        });
                    }
                    
                    ui.separator();
                    
                    // Two-column layout with more vertical space
                    let available_height = ui.available_height();
                    ui.horizontal_top(|ui| {
                        // Left side - file configuration
                        ui.vertical(|ui| {
                            ui.set_width(500.0);
                            ui.set_height(available_height);
                            
                            if !self.files.is_empty() {
                                self.render_file_configuration(ui);
                            }
                        });
                        
                        ui.separator();
                        
                        // Right side - data preview
                        ui.vertical(|ui| {
                            ui.set_height(available_height);
                            ui.label(egui::RichText::new("Data Preview").size(16.0).strong());
                            ui.add_space(8.0);
                            
                            if !self.files.is_empty() {
                                self.render_data_preview(ui);
                            }
                        });
                    });
                });
        });

        result
    }

    fn render_database_config(&mut self, ui: &mut Ui) {
        // Section with card-like styling
        let section_frame = egui::Frame::none()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(70)))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::vec2(15.0, 15.0));
            
        section_frame.show(ui, |ui| {
            ui.heading(RichText::new("Database Configuration").color(Color32::from_gray(220)).size(16.0));
            ui.add_space(10.0);
            
            // Database path
            ui.horizontal(|ui| {
                ui.label(RichText::new("Database Path:").color(Color32::from_gray(180)).size(14.0));
                ui.add_space(10.0);
                
                let available_width = ui.available_width() - 100.0;
                
                // Display the path as a styled label in a frame
                let path_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(55, 55, 55))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(70)))
                    .rounding(egui::Rounding::same(2.0))
                    .inner_margin(egui::vec2(8.0, 4.0));
                    
                path_frame.show(ui, |ui| {
                    ui.set_max_width(available_width.min(400.0));
                    ui.label(RichText::new(self.database_path.to_string_lossy()).color(Color32::from_gray(200)));
                });
                    
                // Browse button with better styling
                let browse_button = egui::Button::new("Browse...")
                    .fill(Color32::from_rgb(60, 60, 60))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)));
                    
                if ui.add(browse_button).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                        .save_file()
                    {
                        self.database_path = path;
                    }
                }
            });
        });
    }

    fn render_file_configuration(&mut self, ui: &mut Ui) {
        self.render_configuration_panel(ui);
    }

    fn render_configuration_panel(&mut self, ui: &mut Ui) {
        if self.files.is_empty() {
            return;
        }

        // Main configuration panel with card styling
        let config_frame = egui::Frame::none()
            .fill(Color32::from_rgb(50, 50, 50))
            .stroke(egui::Stroke::new(1.0, Color32::from_gray(70)))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::vec2(15.0, 15.0));
        
        config_frame.show(ui, |ui| {
            // CSV File selector
            let mut needs_preview_load = false;
            ui.horizontal(|ui| {
                ui.label(RichText::new("CSV File:").color(Color32::from_gray(180)).size(14.0));
                ui.add_space(10.0);
                
                if self.current_file_index >= self.files.len() && !self.files.is_empty() {
                    self.current_file_index = 0;
                }
                
                if let Some(current_file) = self.files.get(self.current_file_index) {
                    let file_name = current_file.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    
                    egui::ComboBox::from_id_source("csv_file_selector")
                        .selected_text(RichText::new(file_name).color(Color32::from_gray(200)))
                        .show_ui(ui, |ui| {
                            ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(70, 70, 70);
                            for (idx, file) in self.files.iter().enumerate() {
                                let name = file.path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown");
                                
                                if ui.selectable_label(self.current_file_index == idx, name).clicked() {
                                    if self.current_file_index != idx {
                                        self.current_file_index = idx;
                                        needs_preview_load = true;
                                    }
                                }
                            }
                        });
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_files_button = egui::Button::new("Add Files...")
                        .fill(Color32::from_rgb(60, 60, 60))
                        .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)));
                        
                    if ui.add(add_files_button).clicked() {
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
            });
            
            ui.label(RichText::new(format!("Files to import: {} total, {} configured", 
                self.files.len(),
                self.files.iter().filter(|f| f.columns.iter().any(|c| c.include)).count()
            )).color(Color32::from_gray(150)).size(12.0));
            
            // Load preview if file selection changed
            if needs_preview_load && self.current_file_index < self.files.len() {
                let should_load = self.files.get(self.current_file_index)
                    .map(|f| f.preview_data.is_none())
                    .unwrap_or(false);
                    
                if should_load {
                    // Extract the data we need to avoid borrow checker issues
                    let (file_path, header_row, delimiter, sample_size) = {
                        let file = &self.files[self.current_file_index];
                        (file.path.clone(), file.header_row, file.delimiter.clone(), file.sample_size)
                    };
                    
                    // Load preview data
                    if let Ok(preview) = self.load_preview_data(&file_path, header_row, &delimiter, sample_size) {
                        self.preview_cache.insert(file_path.clone(), preview.clone());
                        if let Some(file) = self.files.get_mut(self.current_file_index) {
                            file.preview_data = Some(preview.clone());
                            
                            // Update columns if empty (inline to avoid borrow checker issues)
                            if file.columns.is_empty() {
                                let data_rows: Vec<&Vec<String>> = preview.rows.iter().collect();
                                
                                for (i, col_name) in preview.headers.iter().enumerate() {
                                    let inferred_type = infer_column_type(&data_rows, i);
                                    let is_id_column = col_name.to_lowercase() == "id";
                                    
                                    let config = ColumnConfig {
                                        name: col_name.clone(),
                                        data_type: inferred_type,
                                        include: true,
                                        is_primary_key: false,
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

            ui.add_space(15.0);

            // Get values before the mutable borrow
            let idx = self.current_file_index;
            let mut need_reload = false;
            
            // Track previous values for change detection
            let (_prev_header_row, _prev_delimiter) = if let Some(file) = self.files.get(self.current_file_index) {
                (file.header_row, file.delimiter.clone())
            } else {
                (1, Delimiter::Comma)
            };
            
            if let Some(file) = self.files.get_mut(self.current_file_index) {
                // Table name
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Table Name:").color(Color32::from_gray(180)).size(14.0));
                    ui.add_space(10.0);
                    let text_edit = egui::TextEdit::singleline(&mut file.table_name)
                        .text_color(Color32::from_gray(200))
                        .desired_width(200.0);
                    ui.add(text_edit);
                });

                ui.add_space(15.0);

                // Header Configuration
                let header_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(55, 55, 55))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::vec2(12.0, 12.0));
                    
                header_frame.show(ui, |ui| {
                    ui.heading(RichText::new("Header Configuration").color(Color32::from_gray(200)).size(14.0));
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Header Row:").color(Color32::from_gray(170)).size(13.0));
                        let response = ui.add(egui::DragValue::new(&mut file.header_row)
                            .range(1..=50)
                            .speed(0.5));
                        ui.label(RichText::new("(1-50)").color(Color32::from_gray(130)).size(12.0));
                        
                        // Check if header row changed
                        if response.changed() {
                            need_reload = true;
                        }
                    });
                    ui.label(RichText::new("The green highlighted row in the preview is your header")
                        .color(Color32::from_gray(140))
                        .size(11.0));
                });

                ui.add_space(10.0);

                // Sample size
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Sample Size:").color(Color32::from_gray(180)).size(14.0));
                    ui.add(egui::DragValue::new(&mut file.sample_size)
                        .range(10..=10000)
                        .speed(10));
                    ui.label(RichText::new("rows").color(Color32::from_gray(150)).size(13.0));
                    
                    let resample_button = egui::Button::new("↻ Resample")
                        .fill(Color32::from_rgb(60, 60, 60))
                        .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)));
                        
                    if ui.add(resample_button).clicked() {
                        need_reload = true;
                    }
                });

                ui.add_space(15.0);

                // Delimiter
                let delimiter_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(55, 55, 55))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::vec2(12.0, 12.0));
                    
                delimiter_frame.show(ui, |ui| {
                    ui.heading(RichText::new("Delimiter:").color(Color32::from_gray(200)).size(14.0));
                    ui.horizontal_wrapped(|ui| {
                        ui.push_id(format!("delimiter_{}", idx), |ui| {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
                            ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 70);
                            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 65, 65);
                            
                            ui.radio_value(&mut file.delimiter, Delimiter::Comma, "• Comma");
                            ui.radio_value(&mut file.delimiter, Delimiter::Tab, "• Tab");
                            ui.radio_value(&mut file.delimiter, Delimiter::Semicolon, "• Semicolon");
                            ui.radio_value(&mut file.delimiter, Delimiter::Pipe, "• Pipe");
                        });
                    });
                });

                ui.add_space(15.0);

                // Null Values
                let null_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(55, 55, 55))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::vec2(12.0, 12.0));
                    
                null_frame.show(ui, |ui| {
                    ui.heading(RichText::new("Null Values").color(Color32::from_gray(200)).size(14.0));
                    ui.label(RichText::new("Values to treat as NULL:").color(Color32::from_gray(170)).size(13.0));
                    ui.push_id(format!("null_values_{}", idx), |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
                        ui.checkbox(&mut file.null_values.empty_string, "[empty string]");
                        ui.checkbox(&mut file.null_values.null_text, "NULL");
                        ui.checkbox(&mut file.null_values.lowercase_null, "null");
                        ui.checkbox(&mut file.null_values.na, "N/A");
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
                ui.add_space(15.0);

                let column_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(55, 55, 55))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::vec2(12.0, 12.0));
                    
                column_frame.show(ui, |ui| {
                    ui.heading(RichText::new("Column Selection").color(Color32::from_gray(200)).size(14.0));
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        let select_all_button = egui::Button::new("Select All")
                            .fill(Color32::from_rgb(60, 60, 60))
                            .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)));
                            
                        if ui.add(select_all_button).clicked() {
                            for col in &mut file.columns {
                                col.include = true;
                            }
                        }
                        
                        let deselect_all_button = egui::Button::new("Deselect All")
                            .fill(Color32::from_rgb(60, 60, 60))
                            .stroke(egui::Stroke::new(1.0, Color32::from_gray(80)));
                            
                        if ui.add(deselect_all_button).clicked() {
                            for col in &mut file.columns {
                                col.include = false;
                            }
                        }
                        
                        ui.add_space(20.0);
                        ui.label(RichText::new(format!("{}/{} selected", 
                            file.columns.iter().filter(|c| c.include).count(),
                            file.columns.len()
                        )).color(Color32::from_gray(150)).size(13.0));
                    });
                });
            }

            // Column configuration table - rendered outside the file borrow
            let file_idx = self.current_file_index;
            if file_idx < self.files.len() {
                ui.add_space(10.0);
                self.render_column_table(ui, file_idx);
            }
        });
    }

    fn render_column_table(&mut self, ui: &mut Ui, file_idx: usize) {
        if let Some(file) = self.files.get_mut(file_idx) {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            
            // Column table with card styling
            let table_frame = egui::Frame::none()
                .fill(Color32::from_rgb(55, 55, 55))
                .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                .rounding(egui::Rounding::same(4.0))
                .inner_margin(egui::vec2(0.0, 0.0));
            
            table_frame.show(ui, |ui| {
                // Wrap table in unique ID scope
                ui.scope(|ui| {
                    ui.push_id(format!("column_table_{}", file_idx), |ui| {
                        // Custom table styling
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(50, 50, 50);
                        ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(65, 65, 65);
                        ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 70);
                        ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(80, 80, 80);
                        
                        let table = TableBuilder::new(ui)
                            .striped(true)  // Enable striping for alternating row colors
                            .resizable(false)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::exact(50.0)) // Include
                            .column(Column::auto().at_least(120.0)) // Column name
                            .column(Column::exact(110.0)) // Type
                            .column(Column::exact(40.0)) // PK
                            .column(Column::exact(80.0)) // Not Null
                            .column(Column::exact(70.0)) // Unique
                            .column(Column::exact(60.0)) // Index
                            .min_scrolled_height(200.0)
                            .max_scroll_height(400.0);

                        // Header
                        table.header(30.0, |mut header| {
                            header.col(|ui| {
                                ui.label(RichText::new("Include").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Column").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Type").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("PK").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Not Null").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Unique").color(Color32::from_gray(180)).size(12.0));
                            });
                            header.col(|ui| {
                                ui.label(RichText::new("Index").color(Color32::from_gray(180)).size(12.0));
                            });
                        }).body(|body| {
                            let row_height = text_height + 12.0;
                            let num_cols = file.columns.len();
                            
                            body.rows(row_height, num_cols, |mut row| {
                                let row_index = row.index();
                                
                                // Note: We can't set row background colors directly in egui_extras
                                // The striping will be handled by the table's default behavior
                                
                                row.col(|ui| {
                                    ui.push_id(row_index, |ui| {
                                        ui.checkbox(&mut file.columns[row_index].include, "");
                                    });
                                });
                                
                                row.col(|ui| {
                                    ui.label(RichText::new(&file.columns[row_index].name)
                                        .color(Color32::from_gray(200))
                                        .size(13.0));
                                });
                                
                                row.col(|ui| {
                                    // Style the combo box
                                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(60, 60, 60);
                                    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(70, 70, 70);
                                    
                                    egui::ComboBox::from_id_source(format!("type_combo_{}_{}", file_idx, row_index))
                                        .selected_text(RichText::new(file.columns[row_index].data_type.display_name())
                                            .color(Color32::from_gray(200))
                                            .size(12.0))
                                        .show_ui(ui, |ui| {
                                            ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(70, 70, 70);
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
            });
        }
    }

    fn render_data_preview(&mut self, ui: &mut Ui) {
        if let Some(file) = self.files.get(self.current_file_index) {
            if let Some(preview) = &file.preview_data {
                // Limit table height to leave room for buttons below
                let max_height = 400.0;
                let available_height = ui.available_height().min(max_height);
                
                // Preview table with better styling
                let preview_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(48, 48, 48))
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(70)))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::vec2(0.0, 0.0));
                
                preview_frame.show(ui, |ui| {
                    // Wrap table in unique ID scope
                    ui.scope(|ui| {
                        ui.push_id(format!("data_preview_table_{}", self.current_file_index), |ui| {
                            // Custom table styling
                            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45);
                            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 60);
                            ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(70, 70, 70);
                            
                            TableBuilder::new(ui)
                                .striped(true)  // Enable striping for alternating row colors
                                .resizable(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .columns(Column::auto().at_least(80.0), preview.headers.len())
                                .max_scroll_height(available_height)
                                .header(28.0, |mut header| {
                                    // Style header row
                                    for (idx, col_name) in preview.headers.iter().enumerate() {
                                        header.col(|ui| {
                                            // Highlight header row that matches the selected header_row
                                            let is_header_row = file.header_row == 1; // Since preview only shows data after header
                                            let text_color = if is_header_row {
                                                Color32::from_rgb(144, 238, 144) // Light green for header indicator
                                            } else {
                                                Color32::from_gray(220)
                                            };
                                            ui.label(RichText::new(col_name).strong().color(text_color).size(13.0));
                                        });
                                    }
                                })
                                .body(|mut body| {
                                    // preview.rows already contains only data rows (header is separate)
                                    for (row_idx, data_row) in preview.rows.iter().enumerate() {
                                        body.row(22.0, |mut row| {
                                            // Note: We use table striping instead of manual row coloring
                                            
                                            // Ensure we only render as many columns as we have headers
                                            // This prevents crashes when rows have extra delimiters
                                            let num_columns = preview.headers.len();
                                            for i in 0..num_columns {
                                                row.col(|ui| {
                                                    if let Some(cell) = data_row.get(i) {
                                                        let display_text = if cell.is_empty() {
                                                            "NULL"
                                                        } else {
                                                            cell
                                                        };
                                                        
                                                        let text_color = if cell.is_empty() {
                                                            Color32::from_gray(130) // Dimmed for NULL values
                                                        } else {
                                                            Color32::from_gray(200)
                                                        };
                                                        
                                                        ui.label(RichText::new(display_text)
                                                            .color(text_color)
                                                            .size(12.0));
                                                    } else {
                                                        // Empty cell if row has fewer columns than headers
                                                        ui.label(RichText::new("NULL")
                                                            .color(Color32::from_gray(130))
                                                            .size(12.0));
                                                    }
                                                });
                                            }
                                        });
                                    }
                                });
                        });
                    });
                });
                
                // Show row count info
                ui.add_space(5.0);
                ui.label(RichText::new(format!("Showing {} of {} rows", 
                    preview.rows.len().min(file.sample_size), 
                    preview.total_rows
                )).color(Color32::from_gray(150)).size(11.0));
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("No preview available")
                        .color(Color32::from_gray(150))
                        .size(14.0));
                });
            }
        }
    }

    fn load_preview_for_file(&mut self, file: &mut FileConfig) {
        // Check cache first
        if let Some(cached) = self.preview_cache.get(&file.path) {
            file.preview_data = Some(cached.clone());
            
            // Update columns if empty (inline to avoid borrow checker issues)
            if file.columns.is_empty() {
                let data_rows: Vec<&Vec<String>> = cached.rows.iter().collect();
                
                for (i, col_name) in cached.headers.iter().enumerate() {
                    let inferred_type = infer_column_type(&data_rows, i);
                    let is_id_column = col_name.to_lowercase() == "id";
                    
                    let config = ColumnConfig {
                        name: col_name.clone(),
                        data_type: inferred_type,
                        include: true,
                        is_primary_key: false,
                        not_null: is_id_column,
                        unique: is_id_column,
                        create_index: false,
                    };
                    
                    file.columns.push(config);
                }
            }
            return;
        }

        // Load preview data
        if let Ok(preview) = self.load_preview_data(&file.path, file.header_row, &file.delimiter, file.sample_size) {
            self.preview_cache.insert(file.path.clone(), preview.clone());
            file.preview_data = Some(preview.clone());
            
            // Update columns if empty (inline to avoid borrow checker issues)
            if file.columns.is_empty() {
                let data_rows: Vec<&Vec<String>> = preview.rows.iter().collect();
                
                for (i, col_name) in preview.headers.iter().enumerate() {
                    let inferred_type = infer_column_type(&data_rows, i);
                    let is_id_column = col_name.to_lowercase() == "id";
                    
                    let config = ColumnConfig {
                        name: col_name.clone(),
                        data_type: inferred_type,
                        include: true,
                        is_primary_key: false,
                        not_null: is_id_column,
                        unique: is_id_column,
                        create_index: false,
                    };
                    
                    file.columns.push(config);
                }
            }
        }
    }
    
    fn load_preview_data(&self, path: &PathBuf, header_row: usize, delimiter: &Delimiter, sample_size: usize) -> Result<PreviewData, String> {
        use std::fs::File;
        use std::io::{BufReader, BufRead};
        
        // Open file with buffered reader for efficient line-by-line reading
        let file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut lines_iter = reader.lines();
        
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut line_count = 0;
        let mut total_lines = 0;
        
        // Read only the lines we need
        for (idx, line_result) in lines_iter.by_ref().enumerate() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            line_count = idx + 1;
            
            // Get headers from specified row
            if line_count == header_row {
                headers = line
                    .split(delimiter.as_char())
                    .map(|s| s.trim().to_string())
                    .collect();
            }
            
            // Collect sample rows after header
            if line_count > header_row && rows.len() < sample_size {
                let cells: Vec<String> = line
                    .split(delimiter.as_char())
                    .map(|s| s.trim().to_string())
                    .collect();
                rows.push(cells);
            }
            
            // Stop reading once we have enough sample data
            if line_count >= header_row && rows.len() >= sample_size {
                // Count remaining lines for total_rows (quick scan)
                total_lines = line_count + lines_iter.count();
                break;
            }
        }
        
        // If we read all lines, use the line count
        if total_lines == 0 {
            total_lines = line_count;
        }
        
        if headers.is_empty() {
            return Err("No headers found at specified row".to_string());
        }

        // Store in cache for reuse
        let preview = PreviewData {
            headers,
            rows,
            total_rows: total_lines.saturating_sub(header_row),
        };

        Ok(preview)
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