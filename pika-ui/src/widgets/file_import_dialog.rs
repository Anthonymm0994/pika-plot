//! Enhanced file import dialog with comprehensive CSV configuration.

use pika_core::types::ImportOptions;
use pika_engine::enhanced_csv::{EnhancedCsvReader, CsvAnalyzer, CsvFileStats};
use egui::{Context, Ui, Color32, RichText, ScrollArea, Grid};
use std::path::PathBuf;
use std::collections::HashMap;
use arrow::datatypes::DataType;

/// Enhanced file import dialog with comprehensive CSV configuration.
pub struct FileImportDialog {
    selected_file: Option<PathBuf>,
    options: ImportOptions,
    
    // Analysis results
    csv_reader: Option<EnhancedCsvReader>,
    file_stats: Option<CsvFileStats>,
    headers: Vec<String>,
    column_types: Vec<DataType>,
    sample_data: Vec<Vec<String>>, // Changed from StringRecord to Vec<String>
    
    // UI state
    show_advanced: bool,
    preview_rows: usize,
    column_nullable: Vec<bool>,
    column_custom_types: HashMap<usize, DataType>,
    
    // Detection state
    auto_detect_delimiter: bool,
    auto_detect_headers: bool,
    
    // Preview state
    highlight_header_row: bool,
    show_type_detection: bool,
}

impl FileImportDialog {
    pub fn new() -> Self {
        Self {
            selected_file: None,
            options: ImportOptions::default(),
            csv_reader: None,
            file_stats: None,
            headers: Vec::new(),
            column_types: Vec::new(),
            sample_data: Vec::new(),
            show_advanced: true,
            preview_rows: 10,
            column_nullable: Vec::new(),
            column_custom_types: HashMap::new(),
            auto_detect_delimiter: true,
            auto_detect_headers: true,
            highlight_header_row: true,
            show_type_detection: true,
        }
    }
    
    /// Show the dialog and return Some((path, options)) when confirmed.
    pub fn show(&mut self, ctx: &Context) -> Option<(PathBuf, ImportOptions)> {
        let mut result = None;
        
        egui::Window::new("CSV Import Configuration")
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                if let Some(res) = self.show_content(ui) {
                    result = Some(res);
                }
            });
            
        result
    }
    
    fn show_content(&mut self, ui: &mut Ui) -> Option<(PathBuf, ImportOptions)> {
        ui.vertical(|ui| {
            // File selection section
            self.show_file_selection(ui);
            
            if self.selected_file.is_some() {
                ui.separator();
                
                // Configuration section
                self.show_configuration(ui);
                
                ui.separator();
                
                // Preview section
                self.show_preview(ui);
                
                ui.separator();
                
                // Column configuration section
                self.show_column_configuration(ui);
                
                ui.separator();
                
                // Action buttons
                return self.show_action_buttons(ui);
            }
            
            None
        }).inner
    }
    
    fn show_file_selection(&mut self, ui: &mut Ui) {
        ui.heading("📁 File Selection");
        
        ui.horizontal(|ui| {
            ui.label("Selected file:");
            if ui.button("Browse...").clicked() {
                if let Some(file) = rfd::FileDialog::new()
                    .add_filter("CSV Files", &["csv", "tsv", "txt"])
                    .pick_file()
                {
                    self.selected_file = Some(file.clone());
                    self.load_file_analysis();
                }
            }
        });
        
        if let Some(file) = &self.selected_file {
            ui.label(RichText::new(format!("📄 {}", file.display()))
                .color(Color32::from_rgb(100, 150, 255)));
            
            // Show file statistics
            if let Some(stats) = &self.file_stats {
                ui.horizontal(|ui| {
                    ui.label(format!("Size: {:.1} KB", stats.file_size as f64 / 1024.0));
                    ui.separator();
                    ui.label(format!("Est. rows: {}", stats.estimated_rows));
                    ui.separator();
                    ui.label(format!("Encoding: {}", stats.encoding));
                });
            }
        }
    }
    
    fn show_configuration(&mut self, ui: &mut Ui) {
        ui.heading("⚙️ Configuration");
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_advanced, "Show advanced options");
            ui.separator();
            ui.checkbox(&mut self.auto_detect_delimiter, "Auto-detect delimiter");
            ui.separator();
            ui.checkbox(&mut self.auto_detect_headers, "Auto-detect headers");
        });
        
        Grid::new("config_grid")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // Header row option
                ui.label("Has header row:");
                ui.checkbox(&mut self.options.has_header, "");
                ui.end_row();
                
                // Delimiter selection
                ui.label("Delimiter:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.options.delimiter, ',', "Comma (,)");
                    ui.selectable_value(&mut self.options.delimiter, ';', "Semicolon (;)");
                    ui.selectable_value(&mut self.options.delimiter, '\t', "Tab (\\t)");
                    ui.selectable_value(&mut self.options.delimiter, '|', "Pipe (|)");
                });
                ui.end_row();
                
                if self.show_advanced {
                    // Quote character
                    ui.label("Quote character:");
                    ui.horizontal(|ui| {
                        let mut quote_char = self.options.quote_char.unwrap_or('"');
                        ui.selectable_value(&mut quote_char, '"', "Double quote (\")");
                        ui.selectable_value(&mut quote_char, '\'', "Single quote (')");
                        self.options.quote_char = Some(quote_char);
                    });
                    ui.end_row();
                    
                    // Skip rows
                    ui.label("Skip rows:");
                    ui.add(egui::DragValue::new(&mut self.options.skip_rows)
                        .range(0..=100)
                        .suffix(" rows"));
                    ui.end_row();
                    
                    // Max rows
                    ui.label("Max rows:");
                    ui.horizontal(|ui| {
                        if let Some(ref mut max_rows) = self.options.max_rows {
                            ui.add(egui::DragValue::new(max_rows)
                                .range(100..=10000000)
                                .suffix(" rows"));
                            if ui.button("Remove limit").clicked() {
                                self.options.max_rows = None;
                            }
                        } else {
                            ui.label("Unlimited");
                            if ui.button("Set limit").clicked() {
                                self.options.max_rows = Some(10000);
                            }
                        }
                    });
                    ui.end_row();
                    
                    // Encoding
                    ui.label("Encoding:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.options.encoding, "UTF-8".to_string(), "UTF-8");
                        ui.selectable_value(&mut self.options.encoding, "ISO-8859-1".to_string(), "Latin-1");
                        ui.selectable_value(&mut self.options.encoding, "Windows-1252".to_string(), "Windows-1252");
                    });
                    ui.end_row();
                }
            });
        
        // Refresh button
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh Preview").clicked() {
                self.load_file_analysis();
            }
            ui.separator();
            ui.label(format!("Preview rows: {}", self.preview_rows));
            ui.add(egui::DragValue::new(&mut self.preview_rows)
                .range(5..=50)
                .suffix(" rows"));
        });
    }
    
    fn show_preview(&mut self, ui: &mut Ui) {
        ui.heading("👁️ Data Preview");
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.highlight_header_row, "Highlight header row");
            ui.separator();
            ui.checkbox(&mut self.show_type_detection, "Show type detection");
        });
        
        if !self.sample_data.is_empty() {
            ScrollArea::horizontal()
                .max_height(200.0)
                .show(ui, |ui| {
                    Grid::new("preview_grid")
                        .striped(true)
                        .spacing([5.0, 2.0])
                        .show(ui, |ui| {
                            // Show headers if enabled
                            if self.options.has_header && !self.headers.is_empty() {
                                ui.label("Row");
                                for (i, header) in self.headers.iter().enumerate() {
                                    let header_text = if self.highlight_header_row {
                                        RichText::new(header)
                                            .color(Color32::WHITE)
                                            .background_color(Color32::from_rgb(70, 130, 180))
                                    } else {
                                        RichText::new(header)
                                    };
                                    
                                    ui.label(header_text);
                                    
                                    if self.show_type_detection && i < self.column_types.len() {
                                        ui.label(RichText::new(format!("({})", self.format_data_type(&self.column_types[i])))
                                            .color(Color32::GRAY)
                                            .size(10.0));
                                    }
                                }
                                ui.end_row();
                            }
                            
                            // Show sample data
                            for (row_idx, record) in self.sample_data.iter().take(self.preview_rows).enumerate() {
                                ui.label(format!("{}", row_idx + 1));
                                
                                for (col_idx, field) in record.iter().enumerate() {
                                    let display_value = if field.is_empty() {
                                        RichText::new("(empty)")
                                            .color(Color32::GRAY)
                                            .italics()
                                    } else {
                                        RichText::new(field)
                                    };
                                    
                                    ui.label(display_value);
                                    
                                    // Show nullable indicator
                                    if col_idx < self.column_nullable.len() && self.column_nullable[col_idx] {
                                        ui.label(RichText::new("?")
                                            .color(Color32::YELLOW)
                                            .size(10.0));
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
        } else {
            ui.label("No preview data available");
        }
    }
    
    fn show_column_configuration(&mut self, ui: &mut Ui) {
        ui.heading("📊 Column Configuration");
        
        if !self.headers.is_empty() {
            ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    Grid::new("column_config_grid")
                        .num_columns(4)
                        .spacing([10.0, 5.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Headers
                            ui.label(RichText::new("Column").strong());
                            ui.label(RichText::new("Detected Type").strong());
                            ui.label(RichText::new("Custom Type").strong());
                            ui.label(RichText::new("Nullable").strong());
                            ui.end_row();
                            
                            // Column configurations
                            for (i, header) in self.headers.iter().enumerate() {
                                ui.label(header);
                                
                                // Detected type
                                let detected_type = self.column_types.get(i)
                                    .map(|t| self.format_data_type(t))
                                    .unwrap_or_else(|| "Unknown".to_string());
                                ui.label(RichText::new(detected_type).color(Color32::GRAY));
                                
                                // Custom type selector
                                let current_type = self.column_custom_types.get(&i)
                                    .unwrap_or_else(|| self.column_types.get(i).unwrap_or(&DataType::Utf8));
                                let mut selected_type = current_type.clone();
                                
                                egui::ComboBox::from_id_source(format!("type_{}", i))
                                    .selected_text(self.format_data_type(&selected_type))
                                    .show_ui(ui, |ui| {
                                        for data_type in &[DataType::Utf8, DataType::Int64, DataType::Float64, DataType::Boolean] {
                                            if ui.selectable_value(&mut selected_type, data_type.clone(), self.format_data_type(data_type)).clicked() {
                                                // Type was changed
                                            }
                                        }
                                    });
                                
                                // Update the custom type if it changed
                                if selected_type != *current_type {
                                    self.column_custom_types.insert(i, selected_type);
                                }
                                
                                // Nullable checkbox
                                if i >= self.column_nullable.len() {
                                    self.column_nullable.resize(self.headers.len(), true);
                                }
                                ui.checkbox(&mut self.column_nullable[i], "");
                                
                                ui.end_row();
                            }
                        });
                });
        } else {
            ui.label("No columns to configure");
        }
    }
    
    fn show_action_buttons(&mut self, ui: &mut Ui) -> Option<(PathBuf, ImportOptions)> {
        ui.horizontal(|ui| {
            if ui.button("✅ Import").clicked() {
                if let Some(file) = &self.selected_file {
                    return Some((file.clone(), self.options.clone()));
                }
            }
            
            if ui.button("❌ Cancel").clicked() {
                return None;
            }
            
            ui.separator();
            
            if ui.button("💾 Save Configuration").clicked() {
                // TODO: Implement configuration saving
            }
            
            if ui.button("📂 Load Configuration").clicked() {
                // TODO: Implement configuration loading
            }
            
            None
        }).inner
    }
    
    fn load_file_analysis(&mut self) {
        if let Some(file) = &self.selected_file {
            // Auto-detect delimiter if enabled
            if self.auto_detect_delimiter {
                if let Ok(delimiter) = CsvAnalyzer::detect_delimiter(file) {
                    self.options.delimiter = delimiter;
                }
            }
            
            // Auto-detect headers if enabled
            if self.auto_detect_headers {
                if let Ok(has_headers) = CsvAnalyzer::detect_headers(file, self.options.delimiter) {
                    self.options.has_header = has_headers;
                }
            }
            
            // Create CSV reader
            if let Ok(mut reader) = EnhancedCsvReader::new(file, self.options.clone()) {
                // Get file stats
                if let Ok(stats) = reader.file_stats() {
                    self.file_stats = Some(stats);
                }
                
                // Get headers
                if let Ok(headers) = reader.headers() {
                    self.headers = headers;
                    self.column_nullable.resize(self.headers.len(), true);
                }
                
                // Analyze column types
                if let Ok(types) = reader.analyze_column_types(1000) {
                    self.column_types = types;
                }
                
                // Get sample data and convert to Vec<Vec<String>>
                if let Ok(sample) = reader.sample_records(self.preview_rows) {
                    self.sample_data = sample.into_iter()
                        .map(|record| record.iter().map(|s| s.to_string()).collect())
                        .collect();
                }
                
                self.csv_reader = Some(reader);
            }
        }
    }
    
    fn format_data_type(&self, data_type: &DataType) -> String {
        match data_type {
            DataType::Utf8 => "Text".to_string(),
            DataType::Int64 => "Integer".to_string(),
            DataType::Float64 => "Float".to_string(),
            DataType::Boolean => "Boolean".to_string(),
            DataType::Date32 => "Date".to_string(),
            DataType::Timestamp(_, _) => "Timestamp".to_string(),
            _ => format!("{:?}", data_type),
        }
    }
} 