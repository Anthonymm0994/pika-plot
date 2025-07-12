//! Workspace implementation with dual-mode UI (notebook and canvas)

use pika_core::{
    types::{NodeId, Point2},
    events::{EventBus, AppEvent, CanvasEvent},
    node::Node,
    workspace::WorkspaceMode,
};
use crate::{
    canvas::{CanvasState, CanvasWidget},
    nodes::{TableNode, QueryNode, PlotNode, create_node},
};
use std::collections::HashMap;
use std::sync::Arc;
use egui::{Color32, Vec2, Stroke};

/// Main workspace that supports both notebook and canvas modes
pub struct Workspace {
    /// Current workspace mode
    mode: WorkspaceMode,
    
    /// Canvas state (used in canvas mode)
    canvas_state: CanvasState,
    
    /// Notebook cells (used in notebook mode)
    notebook_cells: Vec<NotebookCell>,
    active_cell: Option<usize>,
    
    /// Event bus for communication
    event_bus: EventBus,
    
    /// UI state
    show_mode_switcher: bool,
    transition_progress: f32,
}

/// A cell in notebook mode
pub struct NotebookCell {
    pub id: NodeId,
    pub node: Box<dyn Node>,
    pub collapsed: bool,
    pub execution_number: Option<usize>,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(event_bus: EventBus) -> Self {
        // Create canvas state with event sender
        let canvas_event_sender = event_bus.canvas_events_sender();
        let canvas_state = CanvasState::new(canvas_event_sender);
        
        Self {
            mode: WorkspaceMode::default(),
            canvas_state,
            notebook_cells: Vec::new(),
            active_cell: None,
            event_bus,
            show_mode_switcher: true,
            transition_progress: 0.0,
        }
    }
    
    /// Render the workspace
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Mode switcher
        if self.show_mode_switcher {
            self.render_mode_switcher(ui);
        }
        
        // Main workspace area
        egui::CentralPanel::default().show_inside(ui, |ui| {
            match &self.mode {
                WorkspaceMode::Notebook { .. } => self.render_notebook_mode(ui),
                WorkspaceMode::Canvas { .. } => self.render_canvas_mode(ui),
            }
        });
    }
    
    /// Render mode switcher UI
    fn render_mode_switcher(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("mode_switcher").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Mode:");
                
                let notebook_selected = matches!(self.mode, WorkspaceMode::Notebook { .. });
                if ui.selectable_label(notebook_selected, "📓 Notebook").clicked() {
                    self.switch_to_notebook_mode();
                }
                
                ui.separator();
                
                let canvas_selected = matches!(self.mode, WorkspaceMode::Canvas { .. });
                if ui.selectable_label(canvas_selected, "🎨 Canvas").clicked() {
                    self.switch_to_canvas_mode();
                }
                
                ui.separator();
                
                // Add node buttons
                ui.label("Add:");
                if ui.button("📊 Table").clicked() {
                    self.add_node("TableNode");
                }
                if ui.button("🔍 Query").clicked() {
                    self.add_node("QueryNode");
                }
                if ui.button("📈 Plot").clicked() {
                    self.add_node("PlotNode");
                }
            });
        });
    }
    
    /// Render notebook mode
    fn render_notebook_mode(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Render cells
            let mut cell_to_delete = None;
            let mut cell_to_move = None;
            
            for (idx, cell) in self.notebook_cells.iter_mut().enumerate() {
                let is_active = self.active_cell == Some(idx);
                
                ui.push_id(idx, |ui| {
                    // Cell container
                    let response = ui.group(|ui| {
                        // Cell header
                        ui.horizontal(|ui| {
                            // Execution number
                            if let Some(num) = cell.execution_number {
                                ui.label(format!("[{}]", num));
                            } else {
                                ui.label("[ ]");
                            }
                            
                            // Cell type indicator
                            ui.label(cell.node.type_name());
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Cell controls
                                if ui.small_button("🗑").clicked() {
                                    cell_to_delete = Some(idx);
                                }
                                
                                if ui.small_button(if cell.collapsed { "▶" } else { "▼" }).clicked() {
                                    cell.collapsed = !cell.collapsed;
                                }
                                
                                if idx > 0 && ui.small_button("⬆").clicked() {
                                    cell_to_move = Some((idx, idx - 1));
                                }
                                
                                if idx < self.notebook_cells.len() - 1 && ui.small_button("⬇").clicked() {
                                    cell_to_move = Some((idx, idx + 1));
                                }
                            });
                        });
                        
                        ui.separator();
                        
                        // Cell content
                        if !cell.collapsed {
                            let ctx = pika_core::node::NodeContext {
                                frame_time: ui.ctx().frame_nr() as f32,
                                is_selected: is_active,
                                is_hovered: false,
                                scale_factor: ui.ctx().pixels_per_point(),
                            };
                            
                            cell.node.render(ui, &ctx);
                        }
                    });
                    
                    // Handle cell selection
                    if response.response.clicked() {
                        self.active_cell = Some(idx);
                    }
                    
                    // Visual feedback for active cell
                    if is_active {
                        ui.painter().rect_stroke(
                            response.response.rect.expand(2.0),
                            5.0,
                            Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                        );
                    }
                });
                
                ui.add_space(10.0);
            }
            
            // Handle cell operations
            if let Some(idx) = cell_to_delete {
                self.notebook_cells.remove(idx);
                if self.active_cell == Some(idx) {
                    self.active_cell = None;
                }
            }
            
            if let Some((from, to)) = cell_to_move {
                self.notebook_cells.swap(from, to);
                if self.active_cell == Some(from) {
                    self.active_cell = Some(to);
                } else if self.active_cell == Some(to) {
                    self.active_cell = Some(from);
                }
            }
            
            // Add new cell button at the end
            if ui.button("➕ Add Cell").clicked() {
                self.add_node("QueryNode");
            }
        });
    }
    
    /// Render canvas mode
    fn render_canvas_mode(&mut self, ui: &mut egui::Ui) {
        let canvas = CanvasWidget::new(&mut self.canvas_state);
        ui.add(canvas);
    }
    
    /// Switch to notebook mode
    fn switch_to_notebook_mode(&mut self) {
        // Convert canvas nodes to notebook cells if needed
        if matches!(self.mode, WorkspaceMode::Canvas { .. }) {
            self.notebook_cells.clear();
            
            // Get nodes from canvas and convert to cells
            let nodes: Vec<_> = self.canvas_state.nodes
                .drain()
                .map(|(id, node)| NotebookCell {
                    id,
                    node,
                    collapsed: false,
                    execution_number: None,
                })
                .collect();
            
            self.notebook_cells = nodes;
        }
        
        // Create the cells for WorkspaceMode
        let cells = self.notebook_cells.iter().map(|cell| {
            pika_core::nodes::NotebookCell {
                id: cell.id,
                cell_type: match cell.node.type_name() {
                    "TableNode" => pika_core::nodes::CellType::Table(pika_core::nodes::TableNodeData {
                        source_path: std::path::PathBuf::new(),
                        table_name: String::new(),
                        import_options: pika_core::types::ImportOptions::default(),
                        columns: None,
                        row_count: None,
                    }),
                    "QueryNode" => pika_core::nodes::CellType::Query(pika_core::nodes::QueryNodeData {
                        sql: String::new(),
                        input_tables: Vec::new(),
                        cached_result: None,
                    }),
                    "PlotNode" => pika_core::nodes::CellType::Plot(pika_core::nodes::PlotNodeData {
                        config: pika_core::plots::SimplePlotConfig::default(),
                        source_node: None,
                        cached_data: None,
                    }),
                    _ => pika_core::nodes::CellType::Query(pika_core::nodes::QueryNodeData {
                        sql: String::new(),
                        input_tables: Vec::new(),
                        cached_result: None,
                    }),
                },
                collapsed: cell.collapsed,
                execution_count: cell.execution_number,
            }
        }).collect();
        
        self.mode = WorkspaceMode::Notebook {
            cells,
            active_cell: self.active_cell,
        };
    }
    
    /// Switch to canvas mode
    fn switch_to_canvas_mode(&mut self) {
        // Convert notebook cells to canvas nodes if needed
        if matches!(self.mode, WorkspaceMode::Notebook { .. }) {
            self.canvas_state.nodes.clear();
            
            // Position nodes in a grid
            let mut x = 100.0;
            let mut y = 100.0;
            
            for cell in self.notebook_cells.drain(..) {
                self.canvas_state.nodes.insert(cell.id, cell.node);
                
                // Update position
                if let Some(node) = self.canvas_state.nodes.get_mut(&cell.id) {
                    node.set_position(Point2::new(x, y));
                }
                
                x += 400.0;
                if x > 1200.0 {
                    x = 100.0;
                    y += 300.0;
                }
            }
        }
        
        // Create CanvasNode objects for WorkspaceMode
        let nodes = self.canvas_state.nodes.iter().map(|(id, node)| {
            let node_type = match node.type_name() {
                "TableNode" => pika_core::nodes::NodeType::Table(pika_core::nodes::TableNodeData {
                    source_path: std::path::PathBuf::new(),
                    table_name: String::new(),
                    import_options: pika_core::types::ImportOptions::default(),
                    columns: None,
                    row_count: None,
                }),
                "QueryNode" => pika_core::nodes::NodeType::Query(pika_core::nodes::QueryNodeData {
                    sql: String::new(),
                    input_tables: Vec::new(),
                    cached_result: None,
                }),
                "PlotNode" => pika_core::nodes::NodeType::Plot(pika_core::nodes::PlotNodeData {
                    config: pika_core::plots::SimplePlotConfig::default(),
                    source_node: None,
                    cached_data: None,
                }),
                _ => pika_core::nodes::NodeType::Query(pika_core::nodes::QueryNodeData {
                    sql: String::new(),
                    input_tables: Vec::new(),
                    cached_result: None,
                }),
            };
            
            (*id, pika_core::nodes::CanvasNode {
                id: *id,
                position: node.position(),
                size: node.size(),
                node_type,
                selected: false,
            })
        }).collect();
        
        self.mode = WorkspaceMode::Canvas {
            nodes,
            connections: Vec::new(),
            camera: pika_core::types::Camera2D::default(),
        };
    }
    
    /// Add a new node
    fn add_node(&mut self, node_type: &str) {
        match &mut self.mode {
            WorkspaceMode::Notebook { .. } => {
                if let Some(node) = create_node(node_type, Point2::new(0.0, 0.0)) {
                    let id = node.id();
                    let cell = NotebookCell {
                        id,
                        node,
                        collapsed: false,
                        execution_number: None,
                    };
                    self.notebook_cells.push(cell);
                    self.active_cell = Some(self.notebook_cells.len() - 1);
                }
            }
            WorkspaceMode::Canvas { .. } => {
                // Add to canvas at center of viewport
                let center = self.canvas_state.camera.center;
                if let Some(node) = create_node(node_type, Point2::new(center.x, center.y)) {
                    let id = node.id();
                    self.canvas_state.add_node(node);
                    self.canvas_state.selected_nodes = vec![id];
                }
            }
        }
    }
} 