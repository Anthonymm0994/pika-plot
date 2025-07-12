//! Properties panel for selected nodes.

use crate::state::AppState;
use pika_core::events::AppEvent;
use tokio::sync::broadcast::Sender;
use egui::{Ui, ScrollArea, Color32};

/// Properties panel showing details of selected node.
pub struct PropertiesPanel {}

impl PropertiesPanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        if let Some(selected_id) = state.selected_node {
            // Find the selected node
            if let Some(node) = state.data_nodes.iter().find(|n| n.id == selected_id) {
                ui.heading("Node Properties");
                ui.separator();
                
                // Display node information
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.label(&node.table_info.name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label("Table");
                });
                
                ui.horizontal(|ui| {
                    ui.label("Rows:");
                    ui.label(format!("{}", node.table_info.row_count.map_or("Unknown".to_string(), |n| n.to_string())));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Columns:");
                    ui.label(format!("{}", node.table_info.columns.len()));
                });
                
                ui.separator();
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Preview").clicked() {
                        let _ = event_tx.send(AppEvent::ExecuteQuery {
                            node_id: selected_id,
                            sql: format!("SELECT * FROM {} LIMIT 100", node.table_info.name),
                        });
                    }
                    
                    if ui.button("Edit").clicked() {
                        // Would open edit dialog
                    }
                });
            }
        } else {
            ui.label("No node selected");
        }
    }
} 