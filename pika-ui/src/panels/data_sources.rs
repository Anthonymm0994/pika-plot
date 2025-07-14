//! Data sources panel showing tables and views with metadata.

use egui::{Ui, ScrollArea, Color32, Button, TextEdit};
use crate::state::{AppState, CanvasNodeType};
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
                                    // Find the source data node to get table info
                                    if let Some(source_node) = state.data_nodes.iter().find(|n| &n.table_info.name == name) {
                                        // Create a new instance of this data source on the canvas
                                        let node_id = pika_core::types::NodeId(uuid::Uuid::new_v4());
                                        
                                        // Calculate position with offset to avoid stacking
                                        let existing_count = state.canvas_nodes.values()
                                            .filter(|n| matches!(&n.node_type, CanvasNodeType::Table { table_info } if &table_info.name == name))
                                            .count();
                                        let offset = (existing_count as f32) * 30.0;
                                        
                                        let canvas_node = crate::state::CanvasNode {
                                            id: node_id,
                                            position: egui::Vec2::new(200.0 + offset, 200.0 + offset),
                                            size: egui::Vec2::new(200.0, 150.0),
                                            node_type: crate::state::CanvasNodeType::Table { 
                                                table_info: source_node.table_info.clone() 
                                            },
                                        };
                                        state.canvas_nodes.insert(node_id, canvas_node);
                                    }
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