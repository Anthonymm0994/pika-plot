//! Data panel showing loaded data nodes.

use crate::state::{AppState, DataNode};
use crate::panels::canvas_panel::AppEvent;
use tokio::sync::broadcast::Sender;
use egui::{Ui, ScrollArea};
use egui::Response;
use pika_core::types::NodeId;

/// Data panel showing loaded data sources.
pub struct DataPanel {
    search_query: String,
    dragging_table: Option<NodeId>,
}

impl DataPanel {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            dragging_table: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, _event_tx: &Sender<AppEvent>) {
        ui.heading("Data Sources");
        
        // Import button
        if ui.button("Import CSV").clicked() {
            // TODO: Open file dialog
        }
        
        ui.separator();
        
        // Search
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_query);
        });
        
        ui.separator();
        
        // Data sources list
        ScrollArea::vertical().show(ui, |ui| {
            let mut to_remove = None;
            
            // List tables
            for node in &state.data_nodes {
                let matches_search = self.search_query.is_empty() 
                    || node.table_info.name.to_lowercase().contains(&self.search_query.to_lowercase());
                
                if matches_search {
                    ui.horizontal(|ui| {
                        let selected = state.selected_node == Some(node.id);
                        
                        // Selection indicator
                        if selected {
                            ui.label("▶");
                        } else {
                            ui.label(" ");
                        }
                        
                        // Table name (clickable)
                        let response = ui.selectable_label(selected, &node.table_info.name);
                        
                        if response.clicked() {
                            state.selected_node = Some(node.id);
                        }
                        
                        // Context menu
                        response.context_menu(|ui| {
                            if ui.button("Remove").clicked() {
                                to_remove = Some(node.id);
                                ui.close_menu();
                            }
                            if ui.button("Duplicate").clicked() {
                                // TODO: Implement duplicate
                                ui.close_menu();
                            }
                        });
                        
                        // Drag support - simple approach without memory drag
                        if response.drag_started() {
                            self.dragging_table = Some(node.id);
                        }
                    });
                    
                    // Show table info when selected
                    if state.selected_node == Some(node.id) {
                        ui.indent("table_info", |ui| {
                            ui.label(format!("Rows: {}", node.table_info.row_count.unwrap_or(0)));
                            ui.label(format!("Columns: {}", node.table_info.columns.len()));
                            if let Some(path) = &node.table_info.source_path {
                                ui.label(format!("Path: {}", path.display()));
                            }
                        });
                    }
                }
            }
            
            // Remove marked node outside the iteration
            if let Some(id) = to_remove {
                state.remove_data_node(id);
            }
        });
    }

    fn render_draggable_node(&self, ui: &mut Ui, node: &DataNode) {
        // Simple drag visualization
        let response = ui.group(|ui| {
            ui.label(&node.table_info.name);
        }).response;
        
        // Handle drag without memory.start_drag
        if response.drag_started() && self.dragging_table == Some(node.id) {
            // Visual feedback handled by egui
        }
    }
} 

pub fn show_data_sources(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Data Sources");
    for node in &state.data_nodes {
        let response = ui.label(&node.table_info.name);
        response.drag_started().then(|| {
            // This part of the original code was problematic for memory drag.
            // It was trying to start a drag from a label response.
            // The new_drag_data function is not directly available on Response.
            // This block is kept as is, but it might not work as intended
            // for memory drag without a more complex setup.
        });
    }
} 