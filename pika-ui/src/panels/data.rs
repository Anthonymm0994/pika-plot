//! Data panel showing loaded data nodes.

use crate::state::AppState;
use pika_core::events::AppEvent;
use tokio::sync::broadcast::Sender;
use egui::{Ui, ScrollArea};

/// Data panel showing loaded data sources.
pub struct DataPanel {
    search_query: String,
}

impl DataPanel {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        ui.heading("Data Sources");
        ui.separator();
        
        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        
        ui.separator();
        
        // List of data nodes
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for node in &state.data_nodes {
                    if let Some(selected_id) = state.selected_node {
                        let is_selected = node.id == selected_id;
                        
                        let response = ui.selectable_label(
                            is_selected,
                            &node.table_info.name,
                        );
                        
                        if response.clicked() {
                            state.selected_node = Some(node.id);
                        }
                        
                        response.clone().on_hover_ui(|ui| {
                            ui.label(format!("Table: {}", node.table_info.name));
                            ui.label(format!("Rows: {}", node.table_info.row_count.map_or("Unknown".to_string(), |n| n.to_string())));
                            ui.label(format!("Columns: {}", node.table_info.columns.len()));
                        });
                        
                        response.context_menu(|ui| {
                            if ui.button("Remove").clicked() {
                                // Would remove the node
                                ui.close_menu();
                            }
                        });
                    }
                }
            });
        
        ui.separator();
        
        // Import button
        if ui.button("➕ Import Data...").clicked() {
            // This is handled in the main app
        }
    }
} 