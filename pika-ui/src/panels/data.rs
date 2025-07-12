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
                for (id, node) in state.data_nodes.iter() {
                    let selected = state.selected_node == Some(*id);
                    
                    let response = ui.selectable_label(
                        selected,
                        format!("📊 {}", node.name)
                    );
                    
                    if response.clicked() {
                        state.selected_node = Some(*id);
                    }
                    
                    // Show node info on hover
                    response.on_hover_ui(|ui| {
                        ui.label(format!("Table: {}", node.table_info.table_name));
                        ui.label(format!("Rows: {}", node.table_info.row_count));
                        ui.label(format!("Columns: {}", node.table_info.columns.len()));
                    });
                    
                    // Context menu
                    response.context_menu(|ui| {
                        if ui.button("Query...").clicked() {
                            // TODO: Open query dialog
                            ui.close_menu();
                        }
                        
                        if ui.button("Remove").clicked() {
                            // TODO: Remove node
                            ui.close_menu();
                        }
                    });
                }
            });
        
        ui.separator();
        
        // Import button
        if ui.button("➕ Import Data...").clicked() {
            // This is handled in the main app
        }
    }
} 