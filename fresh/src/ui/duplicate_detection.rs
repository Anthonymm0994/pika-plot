use egui::{Ui, Window, Button, Checkbox, ComboBox, TextEdit, Label, ScrollArea, CollapsingHeader, Grid, Response, Color32};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::core::{Database, DuplicateDetector, DuplicateDetectionConfig, DuplicateDetectionResult, DuplicateBlock};

/// UI state for duplicate detection dialog
pub struct DuplicateDetectionDialog {
    /// Whether the dialog is visible
    pub visible: bool,
    /// Selected group column
    pub selected_group_column: String,
    /// Available columns in the current table
    pub available_columns: Vec<String>,
    /// Columns to ignore during comparison
    pub ignore_columns: HashSet<String>,
    /// Block size for grouping rows
    pub block_size: usize,
    /// Whether to treat null values as equal
    pub null_equals_null: bool,
    /// Current detection result
    pub detection_result: Option<DuplicateDetectionResult>,
    /// Whether detection is in progress
    pub is_detecting: bool,
    /// Error message if detection failed
    pub error_message: Option<String>,
    /// Whether to show the results view
    pub show_results: bool,
    /// Output directory for clean Arrow files
    pub output_directory: PathBuf,
    /// Success message after creating clean file
    pub success_message: Option<String>,
    /// Whether to show success message
    pub show_success: bool,
}

impl Default for DuplicateDetectionDialog {
    fn default() -> Self {
        Self {
            visible: false,
            selected_group_column: String::new(),
            available_columns: Vec::new(),
            ignore_columns: HashSet::new(),
            block_size: 256,
            null_equals_null: true,
            detection_result: None,
            is_detecting: false,
            error_message: None,
            show_results: false,
            output_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            success_message: None,
            show_success: false,
        }
    }
}

impl DuplicateDetectionDialog {
    /// Show the duplicate detection dialog
    pub fn show(&mut self, ctx: &egui::Context, db: &Arc<Database>) {
        if !self.visible {
            return;
        }

        let mut visible = self.visible;
        Window::new("Detect Duplicate Row Blocks")
            .open(&mut visible)
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                self.render_dialog(ui, db);
            });
        self.visible = visible;
    }

    /// Render the main dialog content
    fn render_dialog(&mut self, ui: &mut Ui, db: &Arc<Database>) {
        // Configuration section
        ui.heading("Configuration");
        ui.separator();

        // Group column selection
        ui.horizontal(|ui| {
            ui.label("Group by column:");
            ComboBox::from_id_source("group_column")
                .selected_text(&self.selected_group_column)
                .show_ui(ui, |ui| {
                    for column in &self.available_columns {
                        ui.selectable_value(&mut self.selected_group_column, column.clone(), column);
                    }
                });
        });

        // Block size configuration
        ui.horizontal(|ui| {
            ui.label("Block size:");
            ui.add(TextEdit::singleline(&mut self.block_size.to_string())
                .desired_width(80.0));
            if let Ok(size) = self.block_size.to_string().parse::<usize>() {
                self.block_size = size.max(1).min(1000);
            }
        });

        // Null handling
        ui.checkbox(&mut self.null_equals_null, "Treat null values as equal");

        ui.separator();

        // Column ignore selection
        ui.heading("Columns to Ignore");
        ui.label("Select columns to exclude from comparison (e.g., timestamps, IDs):");

        ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                Grid::new("ignore_columns_grid").show(ui, |ui| {
                                            for column in &self.available_columns {
                            if column != &self.selected_group_column {
                                let mut is_ignored = self.ignore_columns.contains(column);
                                if ui.checkbox(&mut is_ignored, column).clicked() {
                                    if is_ignored {
                                        self.ignore_columns.insert(column.clone());
                                    } else {
                                        self.ignore_columns.remove(column);
                                    }
                                }
                            }
                        }
                });
            });

        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Detect Duplicates").clicked() {
                self.run_detection(db);
            }

            if let Some(_result) = &self.detection_result {
                if ui.button("Show Results").clicked() {
                    self.show_results = true;
                }
            }
        });

        // Status and error messages
        if self.is_detecting {
            ui.label("🔍 Detecting duplicates...");
        }

        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }

        // Results section
        if let Some(result) = &self.detection_result {
            ui.separator();
            ui.heading("Detection Results");
            
            ui.label(format!("Total duplicate blocks: {}", result.total_duplicates));
            ui.label(format!("Total duplicate rows: {}", result.total_duplicate_rows));
            ui.label(format!("Groups processed: {}", result.stats.groups_processed));
            ui.label(format!("Blocks analyzed: {}", result.stats.blocks_analyzed));
            ui.label(format!("Unique blocks found: {}", result.stats.unique_blocks));

            if result.total_duplicates > 0 {
                ui.separator();
                ui.heading("Actions");
                
                // Output directory selection
                ui.horizontal(|ui| {
                    ui.label("Output directory:");
                    ui.label(&self.output_directory.display().to_string());
                    if ui.button("📁 Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.output_directory)
                            .pick_folder() {
                            self.output_directory = path;
                        }
                    }
                });

                // Action buttons
                let result_clone = result.clone();
                let should_remove = ui.button("🗑️ Remove from Database").clicked();
                let should_create = ui.button("💾 Create Clean Arrow File").clicked();
                
                if should_remove {
                    self.remove_duplicates(db, &result_clone);
                }
                
                if should_create {
                    self.create_clean_arrow_file(db, &result_clone);
                }
                
                ui.label("💾 Creates a new Arrow file with duplicates removed");
                ui.label("🗑️ Removes duplicates from the current database");
            }
        }

        // Success message
        if self.show_success {
            if let Some(success_msg) = &self.success_message {
                ui.separator();
                ui.colored_label(Color32::GREEN, success_msg);
                if ui.button("OK").clicked() {
                    self.show_success = false;
                    self.success_message = None;
                }
            }
        }
    }

    /// Run the duplicate detection
    fn run_detection(&mut self, db: &Arc<Database>) {
        if self.selected_group_column.is_empty() {
            self.error_message = Some("Please select a group column".to_string());
            return;
        }

        self.is_detecting = true;
        self.error_message = None;

        // Get the database reference
        let db_guard = db;
        
        // Get the first table (assuming single table for now)
        let tables = db_guard.get_tables().unwrap_or_default();
        if tables.is_empty() {
            self.error_message = Some("No tables available".to_string());
            self.is_detecting = false;
            return;
        }

        let table_name = &tables[0].name;
        
        // Load the table data using the non-mutable version
        match db_guard.get_table_arrow_batch(table_name) {
            Ok(batch) => {
                // Create detector configuration
                let config = DuplicateDetectionConfig {
                    group_column: self.selected_group_column.clone(),
                    ignore_columns: self.ignore_columns.clone(),
                    block_size: self.block_size,
                    null_equals_null: self.null_equals_null,
                };

                let detector = DuplicateDetector::new(config);
                
                // Run detection
                match detector.detect_duplicates(&batch) {
                    Ok(result) => {
                        self.detection_result = Some(result);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Detection failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load table: {}", e));
            }
        }

        self.is_detecting = false;
    }

    /// Remove duplicates from the database
    fn remove_duplicates(&mut self, db: &Arc<Database>, result: &DuplicateDetectionResult) {
        let tables = db.get_tables().unwrap_or_default();
        if tables.is_empty() {
            self.error_message = Some("No tables available".to_string());
            return;
        }

        let table_name = &tables[0].name;
        
        // Create detector configuration
        let config = DuplicateDetectionConfig {
            group_column: self.selected_group_column.clone(),
            ignore_columns: self.ignore_columns.clone(),
            block_size: self.block_size,
            null_equals_null: self.null_equals_null,
        };

        let detector = DuplicateDetector::new(config);
        
        // Note: Since we can't modify the Arc<Database> directly,
        // we'll just show a message that this operation isn't supported
        // in the current implementation
        self.error_message = Some("Remove from database not supported in current implementation. Use 'Create Clean Arrow File' instead.".to_string());
    }

    /// Create a new Arrow file with duplicates removed
    fn create_clean_arrow_file(&mut self, db: &Arc<Database>, result: &DuplicateDetectionResult) {
        let tables = db.get_tables().unwrap_or_default();
        if tables.is_empty() {
            self.error_message = Some("No tables available".to_string());
            return;
        }

        let table_name = &tables[0].name;
        
        // Create detector configuration
        let config = DuplicateDetectionConfig {
            group_column: self.selected_group_column.clone(),
            ignore_columns: self.ignore_columns.clone(),
            block_size: self.block_size,
            null_equals_null: self.null_equals_null,
        };

        let detector = DuplicateDetector::new(config);
        
        // Load the table as Arrow batch using the non-mutable version
        match db.get_table_arrow_batch(table_name) {
            Ok(batch) => {
                // Create clean Arrow file
                match detector.create_clean_arrow_file_with_path(
                    &batch,
                    result,
                    &self.output_directory,
                    table_name,
                ) {
                    Ok((output_path, kept_rows)) => {
                        self.success_message = Some(format!(
                            "✅ Created clean Arrow file: {}\nKept {} rows, removed {} duplicate rows",
                            output_path.display(),
                            kept_rows,
                            batch.num_rows() - kept_rows
                        ));
                        self.show_success = true;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to create clean Arrow file: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load table: {}", e));
            }
        }
    }

    /// Update available columns from the database
    pub fn update_available_columns(&mut self, db: &Arc<Database>) {
        let db_guard = db;
        let tables = db_guard.get_tables().unwrap_or_default();
        
        if let Some(table) = tables.first() {
            self.available_columns = table.columns.iter()
                .map(|col| col.name.clone())
                .collect();
        }
    }
}

/// Results viewer for duplicate blocks
pub struct DuplicateResultsViewer {
    pub visible: bool,
    pub result: Option<DuplicateDetectionResult>,
}

impl Default for DuplicateResultsViewer {
    fn default() -> Self {
        Self {
            visible: false,
            result: None,
        }
    }
}

impl DuplicateResultsViewer {
    /// Show the results viewer
    pub fn show(&mut self, ctx: &egui::Context, db: &Arc<Database>) {
        if !self.visible || self.result.is_none() {
            return;
        }

        let result = self.result.as_ref().unwrap().clone();
        let mut visible = self.visible;

        Window::new("Duplicate Block Results")
            .open(&mut visible)
            .resizable(true)
            .default_size([800.0, 600.0])
            .show(ctx, |ui| {
                self.render_results(ui, &result, db);
            });
        
        self.visible = visible;
    }

    /// Render the results view
    fn render_results(&mut self, ui: &mut Ui, result: &DuplicateDetectionResult, db: &Arc<Database>) {
        ui.heading(format!("Found {} Duplicate Blocks", result.total_duplicates));
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            for (block_idx, block) in result.duplicate_blocks.iter().enumerate() {
                CollapsingHeader::new(format!(
                    "Block {} (Group: {}, {} occurrences)",
                    block_idx + 1,
                    block.group_id,
                    block.row_indices.len()
                ))
                .default_open(false)
                .show(ui, |ui| {
                    self.render_block_details(ui, block, db);
                });
            }
        });
    }

    /// Render details for a specific block
    fn render_block_details(&mut self, ui: &mut Ui, block: &DuplicateBlock, db: &Arc<Database>) {
        ui.label(format!("Block Hash: {:x}", block.block_hash));
        ui.label(format!("Group ID: {}", block.group_id));
        ui.label(format!("Block Size: {}", block.block_size));
        ui.label(format!("Occurrences: {}", block.row_indices.len()));

        // Show row indices for each occurrence
        for (occurrence_idx, row_indices) in block.row_indices.iter().enumerate() {
            CollapsingHeader::new(format!("Occurrence {}", occurrence_idx + 1))
                .show(ui, |ui| {
                    ui.label(format!("Row indices: {:?}", row_indices));
                    
                    // Show sample data for this occurrence
                    let tables = db.get_tables().unwrap_or_default();
                    if let Some(table) = tables.first() {
                        ui.label(format!("Table: {}", table.name));
                    }
                });
        }
    }
} 