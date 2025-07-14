//! Properties panel for selected nodes.

use egui::{Ui, ScrollArea, Color32};
use crate::state::AppState;
use pika_core::events::AppEvent;
use tokio::sync::broadcast;

pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, _event_tx: &broadcast::Sender<AppEvent>) {
        ui.heading("Properties");
        
        ui.separator();
        
        if let Some(selected_node_id) = &state.selected_node {
            if let Some(node) = state.get_data_node(*selected_node_id) {
                ui.label(format!("Selected Node: {}", selected_node_id));
                
                // Find the selected node
                ui.label(&node.table_info.name);
                ui.separator();
                
                ui.label("📊 Table Information:");
                ui.label(format!("Source: {}", 
                    node.table_info.source_path.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                ));
                ui.label(format!("Rows: {}", 
                    node.table_info.row_count.map_or("Unknown".to_string(), |n| n.to_string())
                ));
                ui.label(format!("Columns: {}", node.table_info.columns.len()));
                
                ui.separator();
                ui.label("📋 Column Details:");
                for column in &node.table_info.columns {
                    ui.horizontal(|ui| {
                        ui.label(&column.name);
                        ui.label(format!("({})", column.data_type));
                        if column.nullable {
                            ui.label("nullable");
                        }
                    });
                }
            }
        } else {
            ui.label("No node selected");
            ui.separator();
            ui.label("Select a node on the canvas to view its properties");
        }
        
        ui.separator();
        ui.heading("Canvas Info");
        ui.label(format!("View Mode: {:?}", state.view_mode));
        ui.label(format!("Zoom: {:.2}", state.zoom));
        ui.label(format!("Pan: ({:.1}, {:.1})", state.pan.0, state.pan.1));
        ui.label(format!("Total Nodes: {}", state.data_nodes.len()));
    }
} 