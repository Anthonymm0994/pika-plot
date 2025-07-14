//! Simple CSV import dialog for Pika-Plot

use egui::{Context, Ui, Button, TextEdit, ComboBox};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SimpleCsvImportDialog {
    pub show: bool,
    selected_file: Option<PathBuf>,
    table_name: String,
    has_header: bool,
    delimiter: char,
    error_message: Option<String>,
}

impl Default for SimpleCsvImportDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleCsvImportDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            selected_file: None,
            table_name: String::new(),
            has_header: true,
            delimiter: ',',
            error_message: None,
        }
    }
    
    pub fn open(&mut self) {
        self.show = true;
        self.selected_file = None;
        self.table_name.clear();
        self.error_message = None;
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<PathBuf> {
        let mut result = None;
        
        if self.show {
            egui::Window::new("📊 Import CSV File")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    result = self.render_content(ui);
                });
        }
        
        result
    }
    
    fn render_content(&mut self, ui: &mut Ui) -> Option<PathBuf> {
        let mut result = None;
        
        ui.heading("Import CSV Data");
        ui.separator();
        
        // File selection
        ui.horizontal(|ui| {
            ui.label("File:");
            if ui.button("📂 Select CSV File").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CSV files", &["csv"])
                    .pick_file()
                {
                    self.selected_file = Some(path.clone());
                    if self.table_name.is_empty() {
                        self.table_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("imported_data")
                            .to_string();
                    }
                }
            }
        });
        
        if let Some(ref path) = self.selected_file {
            ui.label(format!("Selected: {}", path.display()));
        } else {
            ui.colored_label(egui::Color32::GRAY, "No file selected");
        }
        
        ui.separator();
        
        // Import options
        ui.horizontal(|ui| {
            ui.label("Table name:");
            ui.text_edit_singleline(&mut self.table_name);
        });
        
        ui.horizontal(|ui| {
            ui.label("Delimiter:");
            ComboBox::from_id_source("delimiter")
                .selected_text(match self.delimiter {
                    ',' => "Comma (,)",
                    ';' => "Semicolon (;)",
                    '\t' => "Tab",
                    '|' => "Pipe (|)",
                    _ => "Other",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.delimiter, ',', "Comma (,)");
                    ui.selectable_value(&mut self.delimiter, ';', "Semicolon (;)");
                    ui.selectable_value(&mut self.delimiter, '\t', "Tab");
                    ui.selectable_value(&mut self.delimiter, '|', "Pipe (|)");
                });
        });
        
        ui.checkbox(&mut self.has_header, "First row contains headers");
        
        ui.separator();
        
        // Error message
        if let Some(ref error) = self.error_message {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }
        
        // Buttons
        ui.horizontal(|ui| {
            if ui.button("✅ Import").clicked() {
                if let Some(ref path) = self.selected_file {
                    if self.table_name.trim().is_empty() {
                        self.error_message = Some("Please enter a table name".to_string());
                    } else {
                        // Simulate import process
                        println!("📊 Importing CSV file: {:?}", path);
                        println!("   Table name: {}", self.table_name);
                        println!("   Has header: {}", self.has_header);
                        println!("   Delimiter: {:?}", self.delimiter);
                        
                        // For now, just return the file path to indicate success
                        result = Some(path.clone());
                        self.show = false;
                    }
                } else {
                    self.error_message = Some("Please select a CSV file".to_string());
                }
            }
            
            if ui.button("❌ Cancel").clicked() {
                self.show = false;
            }
        });
        
        // Preview section (simplified)
        if self.selected_file.is_some() {
            ui.separator();
            ui.heading("Preview");
            ui.label("📊 CSV preview will be shown here");
            ui.label("(Preview functionality coming soon)");
        }
        
        result
    }
} 