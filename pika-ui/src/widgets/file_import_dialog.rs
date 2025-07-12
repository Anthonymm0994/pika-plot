//! File import dialog for loading data files.

use pika_core::types::ImportOptions;
use egui::{Context, Window};
use std::path::PathBuf;

/// File import dialog with configuration options.
pub struct FileImportDialog {
    selected_paths: Vec<PathBuf>,
    options: ImportOptions,
    show: bool,
}

impl FileImportDialog {
    pub fn new() -> Self {
        Self {
            selected_paths: Vec::new(),
            options: ImportOptions::default(),
            show: true,
        }
    }
    
    /// Show the dialog and return Some((paths, options)) when confirmed.
    pub fn show(&mut self, ctx: &Context) -> Option<(Vec<PathBuf>, ImportOptions)> {
        let mut result = None;
        let mut keep_open = self.show;
        
        Window::new("Import Data")
            .open(&mut keep_open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // File selection
                    ui.horizontal(|ui| {
                        ui.label("Files:");
                        if ui.button("Browse...").clicked() {
                            if let Some(paths) = rfd::FileDialog::new()
                                .add_filter("Data files", &["csv", "parquet", "json"])
                                .add_filter("CSV", &["csv"])
                                .add_filter("Parquet", &["parquet"])
                                .add_filter("JSON", &["json", "jsonl"])
                                .pick_files()
                            {
                                self.selected_paths = paths;
                            }
                        }
                    });
                    
                    // Show selected files
                    if !self.selected_paths.is_empty() {
                        ui.separator();
                        ui.label("Selected files:");
                        for path in &self.selected_paths {
                            ui.horizontal(|ui| {
                                ui.label("📄");
                                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                            });
                        }
                    }
                    
                    ui.separator();
                    
                    // Import options
                    ui.heading("Import Options");
                    
                    // CSV specific options
                    ui.collapsing("CSV Options", |ui| {
                        ui.checkbox(&mut self.options.has_header, "First row is header");
                        
                        ui.horizontal(|ui| {
                            ui.label("Delimiter:");
                            ui.selectable_value(&mut self.options.delimiter, b',', "Comma (,)");
                            ui.selectable_value(&mut self.options.delimiter, b'\t', "Tab");
                            ui.selectable_value(&mut self.options.delimiter, b';', "Semicolon (;)");
                            ui.selectable_value(&mut self.options.delimiter, b'|', "Pipe (|)");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Sample rows:");
                            ui.add(egui::DragValue::new(&mut self.options.sample_size)
                                .speed(100)
                                .clamp_range(100..=100000));
                        });
                    });
                    
                    // General options
                    ui.checkbox(&mut self.options.infer_schema, "Infer data types");
                    ui.checkbox(&mut self.options.create_table, "Create new table");
                    
                    if self.options.create_table {
                        ui.horizontal(|ui| {
                            ui.label("Table name:");
                            ui.text_edit_singleline(&mut self.options.table_name);
                        });
                    }
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() && !self.selected_paths.is_empty() {
                            result = Some((self.selected_paths.clone(), self.options.clone()));
                            keep_open = false;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            keep_open = false;
                        }
                    });
                });
            });
        
        self.show = keep_open;
        
        if !keep_open && result.is_none() {
            // Dialog was closed without importing
            None
        } else {
            result
        }
    }
} 