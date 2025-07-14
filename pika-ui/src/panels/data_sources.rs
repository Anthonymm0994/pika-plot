//! Data sources panel showing tables and views with metadata.

use egui::{Ui, ScrollArea, Color32, Button, TextEdit};
use crate::state::AppState;
use pika_core::events::EventBus;
use std::sync::Arc;

pub struct DataSourcesPanel {
    search_query: String,
    selected_table: Option<String>,
}

impl DataSourcesPanel {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            selected_table: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, _event_bus: &Arc<EventBus>) {
        ui.heading("📂 Data Sources");
        
        // Header buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Import CSV...").clicked() {
                state.show_import_dialog = true;
            }
            if ui.button("➕ Open Database...").clicked() {
                // TODO: Open database dialog
                println!("Open database dialog");
            }
        });
        
        ui.separator();
        
        // Tables section
        ui.collapsing("▾ Tables", |ui| {
            // Search bar
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.add(TextEdit::singleline(&mut self.search_query)
                    .desired_width(ui.available_width() - 20.0)
                    .hint_text("Search tables..."));
            });
            
            ui.separator();
            
            // Table list
            ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for node in &state.data_nodes {
                        let name = &node.table_info.name;
                        
                        // Filter by search query
                        if !self.search_query.is_empty() && 
                           !name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                            continue;
                        }
                        
                        ui.horizontal(|ui| {
                            // Table name (clickable)
                            let selected = self.selected_table.as_ref() == Some(name);
                            if ui.selectable_label(selected, name).clicked() {
                                self.selected_table = Some(name.clone());
                            }
                            
                            // Green + button to add to canvas
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let button = Button::new("➕")
                                    .fill(Color32::from_rgb(0, 150, 0))
                                    .small();
                                if ui.add(button).on_hover_text("Add to canvas").clicked() {
                                    // Node already added when importing
                                    println!("Add {} to canvas", name);
                                }
                            });
                        });
                    }
                    
                    if state.data_nodes.is_empty() {
                        ui.label("No tables loaded");
                    }
                });
        });
        
        ui.separator();
        
        // Views section (placeholder)
        ui.collapsing("▾ Views", |ui| {
            ui.label("No views available");
        });
        
        ui.separator();
        
        // Info panel
        if let Some(selected_name) = &self.selected_table {
            if let Some(node) = state.data_nodes.iter().find(|n| &n.table_info.name == selected_name) {
                ui.separator();
                ui.label(format!("ℹ️ Selected: {}", node.table_info.name));
                
                if let Some(path) = &node.table_info.source_path {
                    ui.label(format!("Source: {}", path.display()));
                }
                
                ui.label(format!(
                    "Rows: {}", 
                    node.table_info.row_count.map_or("Unknown".to_string(), |n| n.to_string())
                ));
                ui.label(format!("Columns: {}", node.table_info.columns.len()));
                
                ui.separator();
                ui.label("▾ Column Details");
                
                ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for col in &node.table_info.columns {
                            ui.horizontal(|ui| {
                                ui.label("•");
                                ui.label(&col.name);
                                ui.label(format!("({})", col.data_type));
                                if col.nullable {
                                    ui.label("nullable");
                                }
                            });
                        }
                    });
            }
        }
    }
} 