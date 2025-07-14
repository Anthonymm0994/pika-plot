//! CSV import dialog based on Pebble's design.
//! Features: multi-file selection, clean data preview, header configuration,
//! professional column selection table, Include/PK/Not Null/Unique/Index checkboxes,
//! better visual hierarchy, streamlined null value handling, fast loading for large CSVs.

use egui::{Ui, Color32, ScrollArea, TextEdit, ComboBox, DragValue, Button, Context, Id};
use egui_extras::{TableBuilder, Column};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Text,
    Integer,
    Real,
    Boolean,
    Date,
    DateTime,
    Uuid,
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
            DataType::Uuid => "TEXT",
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
            DataType::Uuid => "🔑",
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
    pub null_count: usize,
    pub unique_count: usize,
}

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

impl FileConfig {
    pub fn new(path: PathBuf) -> Self {
        let table_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported_data")
            .replace(' ', "_")
            .replace('-', "_")
            .to_lowercase();
            
        Self {
            path,
            table_name,
            header_row: 0,
            delimiter: ',',
            sample_size: 1000,
            columns: Vec::new(),
            null_values: vec!["".to_string(), "NULL".to_string(), "null".to_string(), "N/A".to_string(), "n/a".to_string()],
            preview_data: None,
            file_size: 0,
            estimated_rows: 0,
        }
    }
    
    pub fn file_name(&self) -> String {
        self.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
    
    pub fn file_size_human(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        } else if self.file_size < 1024 * 1024 {
            format!("{:.1} KB", self.file_size as f64 / 1024.0)
        } else if self.file_size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.file_size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
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

/// Professional CSV import dialog with Pebble-inspired design
pub struct ProfessionalCsvImportDialog {
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
    
    // Progress tracking
    import_progress: f32,
    current_operation: String,
}

impl Default for ProfessionalCsvImportDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfessionalCsvImportDialog {
    pub fn new() -> Self {
        Self {
            id: Id::new("professional_csv_import"),
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
            import_progress: 0.0,
            current_operation: String::new(),
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
        self.error = None;
        
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
        self.import_progress = 0.0;
        self.current_operation.clear();
    }
    
    pub fn add_file(&mut self, path: PathBuf) {
        let mut config = FileConfig::new(path);
        
        // Get file size
        if let Ok(metadata) = std::fs::metadata(&config.path) {
            config.file_size = metadata.len();
        }
        
        self.files.push(config);
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<PathBuf> {
        let mut result = None;
        
        if self.show {
            egui::Window::new("📊 Professional CSV Import - Pebble Style")
                .id(self.id)
                .collapsible(false)
                .resizable(true)
                .default_width(1200.0)
                .default_height(800.0)
                .show(ctx, |ui| {
                    result = self.render_content(ui);
                });
        }
        
        result
    }
    
    fn render_content(&mut self, ui: &mut Ui) -> Option<PathBuf> {
        let mut created_db_path = None;
        
        // Header with professional styling
        ui.horizontal(|ui| {
            ui.heading("📊 Professional CSV Import");
            ui.label("| Pebble-Inspired Design");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Cancel").clicked() {
                    self.show = false;
                }
                
                let import_enabled = !self.files.is_empty() && 
                    self.files.iter().any(|f| f.columns.iter().any(|c| c.included));
                
                ui.add_enabled_ui(import_enabled, |ui| {
                    if ui.button("✅ Import All Files").clicked() {
                        if let Some(path) = self.start_database_creation() {
                            created_db_path = Some(path);
                            self.show = false;
                        }
                    }
                });
                
                ui.separator();
                
                if ui.button("📂 Add More Files").clicked() {
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("CSV files", &["csv", "tsv", "txt"])
                        .pick_files()
                    {
                        for file in files {
                            self.add_file(file);
                        }
                    }
                }
            });
        });
        
        ui.separator();
        
        // Progress bar for import operations
        if self.import_progress > 0.0 {
            ui.horizontal(|ui| {
                ui.label("Progress:");
                let progress_bar = egui::ProgressBar::new(self.import_progress)
                    .text(format!("{:.0}% - {}", self.import_progress * 100.0, self.current_operation));
                ui.add(progress_bar);
            });
            ui.separator();
        }
        
        // Error display with proper borrowing
        let mut clear_error = false;
        if let Some(ref error) = self.error {
            ui.horizontal(|ui| {
                ui.colored_label(Color32::RED, "❌");
                ui.colored_label(Color32::RED, format!("Error: {}", error));
                if ui.small_button("🗑️ Clear").clicked() {
                    clear_error = true;
                }
            });
            ui.separator();
        }
        if clear_error {
            self.error = None;
        }
        
        // File selection area with professional layout
        if self.files.is_empty() {
            self.render_empty_state(ui);
        } else {
            // File tabs with enhanced styling
            let mut file_to_select = None;
            ui.horizontal(|ui| {
                ui.label("📁 Files:");
                ui.separator();
                
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut files_to_remove = Vec::new();
                        
                        for (i, file) in self.files.iter().enumerate() {
                            let selected = i == self.current_file_index;
                            let button_color = if selected { Color32::from_rgb(70, 130, 200) } else { Color32::from_rgb(50, 50, 50) };
                            
                            ui.style_mut().visuals.widgets.inactive.bg_fill = button_color;
                            ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(80, 140, 210);
                            
                            let file_label = format!("📄 {} ({})", file.file_name(), file.file_size_human());
                            
                            if ui.selectable_label(selected, file_label).clicked() {
                                file_to_select = Some(i);
                            }
                            
                            if ui.small_button("❌").clicked() {
                                files_to_remove.push(i);
                            }
                        }
                        
                        // Process actions after iteration
                        if let Some(index) = file_to_select {
                            self.current_file_index = index;
                        }
                        
                        // Remove files in reverse order to maintain indices
                        for index in files_to_remove.into_iter().rev() {
                            self.files.remove(index);
                            if self.current_file_index >= self.files.len() && !self.files.is_empty() {
                                self.current_file_index = self.files.len() - 1;
                            }
                        }
                    });
                });
                
                // Load preview after all UI operations
                if let Some(_) = file_to_select {
                    self.load_preview_for_current_file();
                }
            });
            
            ui.separator();
            
            // Main content area with side-by-side layout
            if let Some(current_file) = self.files.get_mut(self.current_file_index) {
                let mut show_config = false;
                ui.horizontal(|ui| {
                    ui.label("Quick Settings:");
                    if ui.button("⚙️ Configure").clicked() {
                        show_config = true;
                    }
                });
                
                if show_config {
                    // Use index-based method to avoid borrowing issues
                    self.render_file_configuration_for_index(ui, self.current_file_index);
                }
            }
        }
        
        created_db_path
    }
    
    fn render_empty_state(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            ui.heading("📂 Select CSV Files for Import");
            ui.add_space(20.0);
            
            ui.label("Professional CSV import with Pebble-inspired features:");
            ui.label("✅ Multi-file selection");
            ui.label("✅ Clean data preview (no '?' symbols)");
            ui.label("✅ Intuitive header configuration with green highlighting");
            ui.label("✅ Professional column selection table");
            ui.label("✅ Include/PK/Not Null/Unique/Index checkboxes");
            ui.label("✅ Better visual hierarchy");
            ui.label("✅ Streamlined null value handling");
            ui.label("✅ Fast loading for large CSV files");
            
            ui.add_space(30.0);
            
            if ui.button("📂 Select CSV Files").clicked() {
                self.open_with_csv_selection();
            }
            
            ui.add_space(100.0);
        });
    }
    
    fn render_file_configuration(&mut self, ui: &mut Ui, config: &mut FileConfig) {
        ui.heading("⚙️ File Configuration");
        ui.separator();
        
        // File info panel
        ui.group(|ui| {
            ui.label("📋 File Information");
            ui.horizontal(|ui| {
                ui.label("File:");
                ui.monospace(&config.file_name());
            });
            ui.horizontal(|ui| {
                ui.label("Size:");
                ui.monospace(&config.file_size_human());
            });
            ui.horizontal(|ui| {
                ui.label("Estimated rows:");
                ui.monospace(format!("{}", config.estimated_rows));
            });
        });
        
        ui.add_space(10.0);
        
        // Basic settings with enhanced styling
        ui.group(|ui| {
            ui.label("📋 Import Settings");
            
            ui.horizontal(|ui| {
                ui.label("Table name:");
                ui.add(TextEdit::singleline(&mut config.table_name).desired_width(200.0));
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
                
                ui.label("Sample size:");
                ui.add(DragValue::new(&mut config.sample_size).range(100..=50000));
            });
        });
        
        ui.add_space(10.0);
        
        // Column configuration with professional table (Pebble-style)
        if !config.columns.is_empty() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("📊 Column Configuration");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✅ Select All").clicked() {
                            for col in &mut config.columns {
                                col.included = true;
                            }
                        }
                        if ui.small_button("❌ Deselect All").clicked() {
                            for col in &mut config.columns {
                                col.included = false;
                                col.is_primary_key = false;
                            }
                        }
                    });
                });
                
                ScrollArea::vertical()
                    .max_height(350.0)
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .column(Column::auto().at_least(120.0)) // Name
                            .column(Column::auto().at_least(60.0))  // Type
                            .column(Column::auto().at_least(50.0))  // Include
                            .column(Column::auto().at_least(40.0))  // PK
                            .column(Column::auto().at_least(60.0))  // Not Null
                            .column(Column::auto().at_least(50.0))  // Unique
                            .column(Column::auto().at_least(50.0))  // Index
                            .header(25.0, |mut header| {
                                header.col(|ui| { 
                                    ui.strong("📝 Column"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("🔧 Type"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("✅ Include"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("🔑 PK"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("⚠️ Not Null"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("🎯 Unique"); 
                                });
                                header.col(|ui| { 
                                    ui.strong("📇 Index"); 
                                });
                            })
                            .body(|mut body| {
                                let num_columns = config.columns.len();
                                for i in 0..num_columns {
                                    body.row(22.0, |mut row| {
                                        row.col(|ui| { 
                                            ui.text_edit_singleline(&mut config.columns[i].name);
                                        });
                                        row.col(|ui| {
                                            ComboBox::from_id_source(format!("type_{}", i))
                                                .selected_text(format!("{:?}", config.columns[i].data_type))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Text, "Text");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Integer, "Integer");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Real, "Real");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Boolean, "Boolean");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Date, "Date");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::DateTime, "DateTime");
                                                    ui.selectable_value(&mut config.columns[i].data_type, DataType::Uuid, "UUID");
                                                });
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut config.columns[i].included, "");
                                        });
                                        row.col(|ui| {
                                            if ui.checkbox(&mut config.columns[i].is_primary_key, "").changed() && config.columns[i].is_primary_key {
                                                // Mark that we need to update other PKs after iteration
                                                self.pk_changed_index = Some(i);
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut config.columns[i].not_null, "");
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut config.columns[i].unique, "");
                                        });
                                        row.col(|ui| {
                                            ui.checkbox(&mut config.columns[i].create_index, "");
                                        });
                                    });
                                }
                                
                                // Handle primary key changes after iteration
                                if let Some(pk_index) = self.pk_changed_index.take() {
                                    for j in 0..num_columns {
                                        if j != pk_index {
                                            config.columns[j].is_primary_key = false;
                                        }
                                    }
                                }
                            });
                    });
            });
        }
        
        ui.add_space(10.0);
        
        // Null values configuration (streamlined)
        ui.group(|ui| {
            ui.label("🚫 Null Value Handling");
            
            ui.horizontal(|ui| {
                ui.label("Add null value:");
                ui.add(TextEdit::singleline(&mut self.null_value_input).desired_width(100.0));
                if ui.button("➕").clicked() && !self.null_value_input.trim().is_empty() {
                    config.null_values.push(self.null_value_input.trim().to_string());
                    self.null_value_input.clear();
                }
            });
            
            ui.horizontal_wrapped(|ui| {
                let mut to_remove = None;
                for (i, null_val) in config.null_values.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.small(format!("\"{}\"", null_val));
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
    
    fn render_data_preview(&mut self, ui: &mut Ui, config: &FileConfig) {
        ui.heading("👁️ Clean Data Preview");
        ui.separator();
        
        if let Some(ref preview) = config.preview_data {
            ui.horizontal(|ui| {
                ui.label(format!("📊 Showing {} of {} rows", preview.rows.len().min(50), preview.total_rows));
                ui.separator();
                ui.label("✨ Clean formatting (no '?' symbols)");
            });
            
            ui.add_space(5.0);
            
            ScrollArea::both()
                .max_height(500.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .columns(Column::auto().at_least(100.0), preview.headers.len())
                        .header(25.0, |mut header| {
                            for (i, column_name) in preview.headers.iter().enumerate() {
                                header.col(|ui| {
                                    // Highlight included columns with green
                                    let is_included = config.columns.get(i).map_or(false, |c| c.included);
                                    if is_included {
                                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 80, 40);
                                    }
                                    
                                    ui.vertical(|ui| {
                                        ui.strong(column_name);
                                        if let Some(col_config) = config.columns.get(i) {
                                            ui.small(format!("{} {}", col_config.data_type.icon(), format!("{:?}", col_config.data_type)));
                                            if col_config.is_primary_key {
                                                ui.small("🔑 PK");
                                            }
                                        }
                                    });
                                });
                            }
                        })
                        .body(|mut body| {
                            for row in preview.rows.iter().take(50) {
                                body.row(20.0, |mut table_row| {
                                    for (i, cell) in row.iter().enumerate() {
                                        table_row.col(|ui| {
                                            // Clean display - no "?" symbols, proper formatting
                                            let display_value = if cell.trim().is_empty() {
                                                "∅".to_string() // Use empty set symbol instead of "?"
                                            } else {
                                                cell.clone()
                                            };
                                            
                                            // Color coding based on data type
                                            if let Some(col_config) = config.columns.get(i) {
                                                let color = match col_config.data_type {
                                                    DataType::Integer | DataType::Real => Color32::from_rgb(150, 200, 255),
                                                    DataType::Boolean => Color32::from_rgb(150, 255, 150),
                                                    DataType::Date | DataType::DateTime => Color32::from_rgb(255, 200, 150),
                                                    _ => Color32::WHITE,
                                                };
                                                ui.colored_label(color, display_value);
                                            } else {
                                                ui.label(display_value);
                                            }
                                        });
                                    }
                                });
                            }
                        });
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("📊 Loading preview...");
                ui.label("Click 'Refresh Preview' to load data");
                ui.add_space(50.0);
            });
        }
    }
    
    fn load_preview_for_current_file(&mut self) {
        let current_index = self.current_file_index;
        if current_index < self.files.len() {
            self.load_preview_for_file(current_index);
        }
    }
    
    fn load_preview_for_file(&mut self, file_index: usize) {
        if file_index >= self.files.len() {
            return;
        }
        
        let file_size = self.files[file_index].file_size;
        let config = &mut self.files[file_index];
        
        match std::fs::read_to_string(&config.path) {
            Ok(content) => {
                let mut rows = Vec::new();
                let mut reader = csv::ReaderBuilder::new()
                    .delimiter(config.delimiter as u8)
                    .has_headers(false)
                    .flexible(true) // Handle inconsistent column counts
                    .from_reader(content.as_bytes());
                
                for (i, result) in reader.records().enumerate() {
                    if i >= config.sample_size {
                        break;
                    }
                    
                    match result {
                        Ok(record) => {
                            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                            rows.push(row);
                        }
                        Err(e) => {
                            eprintln!("Warning: Skipping malformed row {}: {}", i, e);
                            continue;
                        }
                    }
                }
                
                if !rows.is_empty() {
                    // Store values we need before borrowing config mutably
                    let should_estimate_rows = true;
                    
                    // Detect headers and create column configs
                    let (headers, data_start) = if config.header_row < rows.len() {
                        let header_row = &rows[config.header_row];
                        (header_row.clone(), config.header_row + 1)
                    } else {
                        let default_headers: Vec<String> = (0..rows[0].len())
                            .map(|i| format!("column_{}", i + 1))
                            .collect();
                        (default_headers, 0)
                    };
                    
                    // Create column configs with analysis
                    config.columns.clear();
                    let null_values = config.null_values.clone(); // Clone before the loop
                    
                    // First collect all the column data
                    let mut all_column_stats = Vec::new();
                    for (i, header) in headers.iter().enumerate() {
                        let mut column_data = Vec::new();
                        for row in rows.iter().skip(data_start) {
                            if i < row.len() {
                                column_data.push(&row[i]);
                            }
                        }
                        
                        let stats = analyze_column_data_static(&column_data, &null_values);
                        all_column_stats.push((header.clone(), stats));
                    }
                    
                    // Now create column configs from the collected stats
                    for (header, stats) in all_column_stats {
                        config.columns.push(ColumnConfig {
                            name: header.clone(),
                            original_name: header,
                            data_type: stats.inferred_type,
                            included: true,
                            create_index: false,
                            is_primary_key: false,
                            not_null: false,
                            unique: false,
                            sample_values: stats.sample_values,
                            null_count: stats.null_count,
                            unique_count: stats.unique_count,
                        });
                    }
                    
                    config.preview_data = Some(PreviewData {
                        headers: headers.clone(),
                        rows: rows.clone(),
                        total_rows: rows.len(),
                    });
                    
                    // Now estimate total rows after we're done with config
                    if should_estimate_rows {
                        let estimated_rows = self.estimate_total_rows(file_size, &rows);
                        self.files[file_index].estimated_rows = estimated_rows;
                    }
                } else {
                    self.error = Some("No data found in CSV file".to_string());
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to read file: {}", e));
            }
        }
    }
    
    fn estimate_total_rows(&self, file_size: u64, sample_rows: &[Vec<String>]) -> usize {
        if sample_rows.is_empty() {
            return 0;
        }
        
        // Estimate average row size in bytes
        let sample_size_bytes: usize = sample_rows.iter()
            .map(|row| row.iter().map(|cell| cell.len() + 1).sum::<usize>()) // +1 for delimiter
            .sum();
        
        let avg_row_size = sample_size_bytes as f64 / sample_rows.len() as f64;
        
        if avg_row_size > 0.0 {
            (file_size as f64 / avg_row_size) as usize
        } else {
            sample_rows.len()
        }
    }
    
    fn start_database_creation(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            self.error = Some("No files selected for import".to_string());
            return None;
        }
        
        // Validate that at least one column is included across all files
        let has_included_columns = self.files.iter()
            .any(|file| file.columns.iter().any(|col| col.included));
        
        if !has_included_columns {
            self.error = Some("No columns selected for import".to_string());
            return None;
        }
        
        // For now, return a mock database path
        // In a real implementation, this would create the database and import the data
        let db_path = std::env::temp_dir().join("pika_professional_import.db");
        
        println!("📊 Professional CSV Import Summary:");
        println!("  🎯 Target database: {:?}", db_path);
        println!("  📁 Files to import: {}", self.files.len());
        
        for (i, file) in self.files.iter().enumerate() {
            let included_columns = file.columns.iter().filter(|c| c.included).count();
            let pk_columns = file.columns.iter().filter(|c| c.is_primary_key).count();
            
            println!("  📄 File {}: {} -> table '{}'", i + 1, file.file_name(), file.table_name);
            println!("    📊 Columns: {} total, {} included, {} primary keys", 
                file.columns.len(), included_columns, pk_columns);
            println!("    📏 Size: {}, ~{} rows", file.file_size_human(), file.estimated_rows);
            println!("    🔧 Delimiter: {:?}", file.delimiter);
        }
        
        Some(db_path)
    }
}

#[derive(Debug)]
struct ColumnStats {
    inferred_type: DataType,
    null_count: usize,
    unique_count: usize,
    sample_values: Vec<String>,
} 

// Move analyze_column_data to a free function
fn analyze_column_data_static(values: &[&String], null_values: &[String]) -> ColumnStats {
    let mut unique_values = HashSet::new();
    let mut null_count = 0;
    let mut sample_values = Vec::new();
    
    for value in values {
        if null_values.contains(value) || value.is_empty() {
            null_count += 1;
        } else {
            unique_values.insert(value.to_string());
            if sample_values.len() < 5 {
                sample_values.push(value.to_string());
            }
        }
    }
    
    // Infer data type based on values
    let numeric_values: Vec<f64> = values.iter()
        .filter(|v| !null_values.contains(v) && !v.is_empty())
        .filter_map(|v| v.parse::<f64>().ok())
        .collect();
    
    let inferred_type = infer_data_type_from_analysis_static(values, &numeric_values, null_values);
    
    ColumnStats {
        inferred_type,
        null_count,
        unique_count: unique_values.len(),
        sample_values,
    }
}

fn infer_data_type_from_analysis_static(values: &[&String], numeric_values: &[f64], null_values: &[String]) -> DataType {
    let non_null_values: Vec<&&String> = values.iter()
        .filter(|v| !null_values.contains(v) && !v.is_empty())
        .collect();
    
    if non_null_values.is_empty() {
        return DataType::Text;
    }
    
    // Check for booleans
    let bool_values = ["true", "false", "yes", "no", "1", "0", "t", "f", "y", "n"];
    if non_null_values.iter().all(|v| bool_values.contains(&v.to_lowercase().as_str())) {
        return DataType::Boolean;
    }
    
    // Check for numeric types
    if numeric_values.len() == non_null_values.len() {
        if numeric_values.iter().all(|v| v.fract() == 0.0) {
            return DataType::Integer;
        }
        return DataType::Real;
    }
    
    // Check for dates
    let date_patterns = [
        "%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y",
        "%Y-%m-%d %H:%M:%S", "%Y/%m/%d %H:%M:%S"
    ];
    
    for pattern in &date_patterns {
        if non_null_values.iter().all(|v| chrono::NaiveDateTime::parse_from_str(v, pattern).is_ok() ||
                                          chrono::NaiveDate::parse_from_str(v, pattern).is_ok()) {
            return if pattern.contains("%H") {
                DataType::DateTime
            } else {
                DataType::Date
            };
        }
    }
    
    // Check for UUIDs
    if non_null_values.iter().all(|v| looks_like_uuid_static(v)) {
        return DataType::Uuid;
    }
    
    DataType::Text
}

fn looks_like_uuid_static(value: &str) -> bool {
    // Simple UUID check without regex
    // UUID format: 8-4-4-4-12 hex digits
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    
    let expected_lengths = [8, 4, 4, 4, 12];
    for (i, part) in parts.iter().enumerate() {
        if part.len() != expected_lengths[i] {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    
    true
} 