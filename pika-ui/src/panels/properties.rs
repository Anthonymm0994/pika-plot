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
        ui.heading("Properties");
        ui.separator();
        
        if let Some(selected_id) = state.selected_node {
            if let Some(node) = state.data_nodes.get(&selected_id) {
                // Node name
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.label(&node.name);
                });
                
                ui.separator();
                
                // Table info
                ui.label(egui::RichText::new("Table Information").strong());
                ui.indent("table_info", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Table:");
                        ui.label(&node.table_info.table_name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Rows:");
                        ui.label(format!("{}", node.table_info.row_count));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.label(humansize::format_size(
                            node.table_info.estimated_size,
                            humansize::DECIMAL
                        ));
                    });
                });
                
                ui.separator();
                
                // Columns
                ui.label(egui::RichText::new("Columns").strong());
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for column in &node.table_info.columns {
                            ui.horizontal(|ui| {
                                // Type icon
                                let (icon, color) = match column.data_type.as_str() {
                                    "INTEGER" | "BIGINT" => ("🔢", Color32::from_rgb(100, 150, 200)),
                                    "REAL" | "DOUBLE" => ("📊", Color32::from_rgb(150, 200, 100)),
                                    "VARCHAR" | "TEXT" => ("📝", Color32::from_rgb(200, 150, 100)),
                                    "BOOLEAN" => ("✓", Color32::from_rgb(150, 100, 200)),
                                    "DATE" | "TIMESTAMP" => ("📅", Color32::from_rgb(200, 100, 150)),
                                    _ => ("❓", Color32::from_gray(150)),
                                };
                                
                                ui.colored_label(color, icon);
                                ui.label(&column.name);
                                ui.weak(format!("({})", column.data_type));
                            });
                        }
                    });
                
                ui.separator();
                
                // Actions
                ui.label(egui::RichText::new("Actions").strong());
                
                if ui.button("🔍 Query Data").clicked() {
                    event_tx.send(AppEvent::ExecuteQuery {
                        id: selected_id,
                        sql: format!("SELECT * FROM {} LIMIT 100", node.table_info.table_name),
                        cache_key: None,
                    }).ok();
                }
                
                if ui.button("📊 Create Plot").clicked() {
                    // TODO: Open plot configuration dialog
                }
                
                if ui.button("🔗 Export").clicked() {
                    // TODO: Export functionality
                }
                
                // Last query result
                if let Some(result) = &node.last_query_result {
                    ui.separator();
                    ui.label(egui::RichText::new("Last Query Result").strong());
                    ui.indent("query_result", |ui| {
                        ui.label(format!("Rows: {}", result.row_count));
                        ui.label(format!("Execution time: {:.2}ms", result.execution_time_ms));
                    });
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.weak("Select a node to view properties");
            });
        }
    }
} 