//! Properties panel for selected nodes.

use egui::{Ui, TextEdit, ScrollArea};
use crate::state::{AppState, CanvasNodeType};
use crate::panels::canvas_panel::AppEvent;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        ui.heading("Properties");
        
        ui.separator();
        
        if let Some(selected_node_id) = state.selected_node.clone() {
            if let Some(node) = state.get_canvas_node_mut(selected_node_id) {
                match &mut node.node_type {
                    CanvasNodeType::Table { table_info } => {
                        ui.label(format!("Table: {}", table_info.name));
                        ui.label(format!("Rows: {:?}", table_info.row_count));
                        ui.separator();
                        ui.label("Position:");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut node.position.x));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut node.position.y));
                        });
                        
                        ui.separator();
                        ui.label("Size:");
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add(egui::DragValue::new(&mut node.size.x).range(50.0..=800.0));
                            ui.label("Height:");
                            ui.add(egui::DragValue::new(&mut node.size.y).range(50.0..=600.0));
                        });
                        
                        ui.separator();
                        ui.label("Column Details:");
                        
                        ScrollArea::vertical()
                            .id_source(format!("properties_columns_{:?}", selected_node_id))
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for column in &table_info.columns {
                                    ui.horizontal(|ui| {
                                        ui.label(&column.name);
                                        ui.label(format!("({})", column.data_type));
                                        if column.nullable {
                                            ui.label("nullable");
                                        }
                                    });
                                }
                            });
                    }
                    CanvasNodeType::Plot { plot_type } => {
                        ui.label("📈 Plot");
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.text_edit_singleline(plot_type);
                        });
                        
                        ui.separator();
                        ui.label("Plot Configuration:");
                        ui.label("X Column: [Select]");
                        ui.label("Y Column: [Select]");
                        ui.label("Color By: [None]");
                        
                        // TODO: Add plot-specific configuration UI
                    }
                    CanvasNodeType::Note { content } => {
                        ui.label("📝 Note");
                        ui.separator();
                        
                        ui.label("Content:");
                        ui.add(TextEdit::multiline(content)
                            .desired_width(ui.available_width())
                            .desired_rows(10));
                    }
                    CanvasNodeType::Shape { shape_type } => {
                        ui.label("🔷 Shape");
                        ui.separator();
                        
                        ui.label(format!("Type: {:?}", shape_type));
                        
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.label(format!("({:.0}, {:.0})", node.position.x, node.position.y));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.label(format!("({:.0}, {:.0})", node.size.x, node.size.y));
                        });
                    }
                }
                
                ui.separator();
                
                // Common properties
                ui.collapsing("Transform", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut node.position.x).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut node.position.y).speed(1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(egui::DragValue::new(&mut node.size.x).speed(1.0).range(10.0..=1000.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(egui::DragValue::new(&mut node.size.y).speed(1.0).range(10.0..=1000.0));
                    });
                });
            }
        } else {
            ui.label("No node selected");
            ui.separator();
            ui.label("Select a node on the canvas to view its properties");
        }
        
        ui.separator();
        ui.collapsing("Canvas Info", |ui| {
            ui.label(format!("View Mode: {:?}", state.view_mode));
            ui.label(format!("Tool: {:?}", state.tool_mode));
            ui.label(format!("Zoom: {:.2}", state.canvas_state.zoom));
            ui.label(format!("Pan: ({:.1}, {:.1})", state.canvas_state.pan_offset.x, state.canvas_state.pan_offset.y));
            ui.label(format!("Total Nodes: {}", state.canvas_nodes.len()));
            ui.label(format!("Connections: {}", state.connections.len()));
        });
    }
} 