//! Status bar panel for application status.

use egui::Ui;
use crate::state::AppState;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &AppState) {
        ui.horizontal(|ui| {
            // Left side - mode and selection info
            ui.label(format!("Mode: {:?}", state.view_mode));
            ui.separator();
            
            match state.selected_node {
                Some(node_id) => {
                    if let Some(canvas_node) = state.get_canvas_node(node_id) {
                        match &canvas_node.node_type {
                            crate::state::CanvasNodeType::Table { table_info } => {
                                ui.label(format!("Selected: Table '{}'", table_info.name));
                            }
                            crate::state::CanvasNodeType::Plot { plot_type } => {
                                ui.label(format!("Selected: {} Plot", plot_type));
                            }
                            /* Note and Shape nodes disabled
                            crate::state::CanvasNodeType::Note { .. } => {
                                ui.label("Selected: Note");
                            }
                            crate::state::CanvasNodeType::Shape { shape_type } => {
                                ui.label(format!("Selected: Shape ({:?})", shape_type));
                            }
                            */
                        }
                    }
                }
                None => {
                    ui.label("No selection");
                }
            }
            
            ui.separator();
            ui.label(format!("Tool: {:?}", state.tool_mode));
            
            // Right side - canvas info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Connections: {}", state.connections.len()));
                ui.separator();
                ui.label(format!("Nodes: {}", state.canvas_nodes.len()));
                ui.separator();
                ui.label(format!("Data Sources: {}", state.data_nodes.len()));
            });
        });
    }
} 