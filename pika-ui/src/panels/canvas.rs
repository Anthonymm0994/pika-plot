//! Canvas panel for node-based data visualization.

use crate::state::{AppState, NodeConnection, ConnectionType, CanvasNode, CanvasNodeType, ShapeType, ToolMode, NodeDataPreview};
use pika_core::types::NodeId;
use tokio::sync::broadcast::Sender;
use egui::{Context, Ui, Painter, Pos2, Vec2, Color32, Stroke, Rect, Response, Sense, FontId, menu};
use crate::panels::canvas_panel::AppEvent;
use egui_extras::{TableBuilder, Column};

const RESIZE_HANDLE_SIZE: f32 = 8.0;
const MIN_NODE_SIZE: f32 = 50.0;
const DEFAULT_TABLE_SIZE: Vec2 = Vec2::new(400.0, 300.0);
const DEFAULT_PLOT_SIZE: Vec2 = Vec2::new(350.0, 250.0);
const DEFAULT_SHAPE_SIZE: Vec2 = Vec2::new(150.0, 100.0);

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Canvas panel for node-based visualization.
pub struct CanvasPanel {
    /// Dragging state
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    
    /// Resizing state
    resizing_node: Option<(NodeId, ResizeHandle)>,
    resize_start_size: Vec2,
    resize_start_pos: Vec2,
    
    /// Connection creation state
    connecting_from: Option<NodeId>,
    
    /// Canvas transform
    pan_offset: Vec2,
    zoom: f32,
    
    /// Drawing state
    drawing_start: Option<Pos2>,
    current_stroke: Vec<Pos2>,
    
    /// Preview shape while drawing
    preview_shape: Option<NodeId>,
    
    /// Context menu position
    context_menu_pos: Option<Pos2>,
}

impl CanvasPanel {
    pub fn new(_ctx: std::sync::Arc<pika_core::events::EventBus>) -> Self {
        Self {
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            resizing_node: None,
            resize_start_size: Vec2::ZERO,
            resize_start_pos: Vec2::ZERO,
            connecting_from: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            drawing_start: None,
            current_stroke: Vec::new(),
            preview_shape: None,
            context_menu_pos: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            Sense::click_and_drag(),
        );
        
        // Draw the canvas background FIRST
        painter.rect_filled(
            response.rect,
            0.0,
            Color32::from_rgb(20, 20, 20),
        );
        
        // Update our internal state from the app state
        self.zoom = state.canvas_state.zoom;
        self.pan_offset = state.canvas_state.pan_offset;
        
        // Handle middle mouse pan
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan_offset += response.drag_delta();
            state.canvas_state.pan_offset = self.pan_offset;
        }
        
        // Handle zoom with scroll wheel
        response.ctx.input(|i| {
        if response.hovered() {
                if i.raw_scroll_delta.y != 0.0 {
                    let zoom_delta = 1.0 + i.raw_scroll_delta.y * 0.01;
                    self.zoom = (self.zoom * zoom_delta).clamp(0.1, 5.0);
                    // Zoom towards mouse position
                    state.canvas_state.zoom = self.zoom;
                }
            }
        });
        
        // Transform helpers
        let zoom = self.zoom;
        let pan_offset = self.pan_offset;
        let rect = response.rect;
        
        let from_screen = move |pos: Pos2| -> Pos2 {
            Pos2::new(
                (pos.x - rect.left() - pan_offset.x) / zoom,
                (pos.y - rect.top() - pan_offset.y) / zoom,
            )
        };
        
        let to_screen = move |pos: Pos2| -> Pos2 {
            Pos2::new(
                pos.x * zoom + pan_offset.x + rect.left(),
                pos.y * zoom + pan_offset.y + rect.top(),
            )
        };
        
        // Draw grid
        if state.canvas_state.show_grid {
        self.draw_grid(&painter, &response.rect);
        }
        
        // Draw connections
        for connection in &state.connections {
            if let (Some(from_node), Some(to_node)) = (
                state.get_canvas_node(connection.from),
                state.get_canvas_node(connection.to),
            ) {
                // Calculate connection points
                let from_pos = to_screen(Pos2::new(from_node.position.x + from_node.size.x, 
                                                   from_node.position.y + from_node.size.y / 2.0));
                let to_pos = to_screen(Pos2::new(to_node.position.x, 
                                                to_node.position.y + to_node.size.y / 2.0));
                
                // Draw bezier curve with color based on connection type
                let color = match connection.connection_type {
                    ConnectionType::DataFlow => Color32::from_rgb(100, 150, 250),
                    ConnectionType::Transform => Color32::from_rgb(250, 150, 100),
                    ConnectionType::Join => Color32::from_rgb(150, 250, 100),
                };
                
                let control1 = from_pos + Vec2::new(50.0, 0.0);
                let control2 = to_pos - Vec2::new(50.0, 0.0);
                
                let points = self.bezier_points(from_pos, control1, control2, to_pos, 20);
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(2.0, color),
                ));
            }
        }
        
        // Draw temporary connection line while creating
        if let Some(from_id) = self.connecting_from {
            if let Some(from_node) = state.get_canvas_node(from_id) {
                let from_pos = to_screen(Pos2::new(from_node.position.x + from_node.size.x,
                                                   from_node.position.y + from_node.size.y / 2.0));
                let to_pos = response.hover_pos().unwrap_or(from_pos);
                
                painter.line_segment(
                    [from_pos, to_pos],
                    Stroke::new(2.0, Color32::from_gray(150)),
                );
            }
        }
        
        // Tool-specific handling BEFORE drawing nodes so preview shapes appear immediately
        match state.tool_mode {
            ToolMode::Select => self.handle_select_tool(&response, state, from_screen, to_screen, event_tx),
            ToolMode::Pan => {
                // Pan is handled above with middle mouse, so nothing extra needed here
            },
            ToolMode::Rectangle => self.handle_shape_tool(&response, state, from_screen, ShapeType::Rectangle),
            ToolMode::Circle => self.handle_shape_tool(&response, state, from_screen, ShapeType::Circle),
            ToolMode::Line => self.handle_line_tool(&response, state, from_screen),
            ToolMode::Draw => self.handle_draw_tool(&response, state, from_screen, &painter, to_screen),
            ToolMode::Text => self.handle_text_tool(&response, state, from_screen),
        }
        
        // Draw canvas nodes AFTER tool handling so preview shapes are visible
        let nodes: Vec<_> = state.canvas_nodes.keys().cloned().collect();
        for node_id in nodes {
            if let Some(node) = state.get_canvas_node(node_id) {
                let is_selected = state.selected_node == Some(node_id);
                self.draw_node(&painter, node, to_screen, is_selected, state);
                
                // Draw resize handles for selected nodes
                if is_selected && state.tool_mode == ToolMode::Select {
                    self.draw_resize_handles(&painter, node, to_screen);
                }
            }
        }
        
        // Handle right-click context menu
        if response.clicked_by(egui::PointerButton::Secondary) {
            if let Some(pos) = response.interact_pointer_pos() {
                self.context_menu_pos = Some(from_screen(pos));
            }
        }
        
        // Show context menu if position is set
        if let Some(menu_pos) = self.context_menu_pos {
            let screen_pos = to_screen(menu_pos);
            
            // Check if we right-clicked on a node
            let clicked_node = state.canvas_nodes.values().find(|node| {
                let node_rect = Rect::from_min_size(
                    Pos2::new(node.position.x, node.position.y),
                    node.size,
                );
                node_rect.contains(menu_pos)
            }).map(|n| n.id);
            
            // Show appropriate context menu
            ui.allocate_ui_at_rect(
                Rect::from_min_size(screen_pos, Vec2::splat(1.0)),
                |ui| {
                    menu::bar(ui, |ui| {
                        ui.menu_button("", |ui| {
                            if let Some(node_id) = clicked_node {
                                self.show_node_context_menu(ui, state, node_id);
            } else {
                                self.show_canvas_context_menu(ui, state, menu_pos);
                            }
                            
                            // Close menu after any action
                            if ui.button("Cancel").clicked() {
                                self.context_menu_pos = None;
                                ui.close_menu();
                            }
                        });
                    });
                }
            );
        }

        // Handle keyboard input for query editing
        if let Some(selected_id) = state.selected_node {
            if let Some(node) = state.get_canvas_node(selected_id) {
                if let CanvasNodeType::Table { .. } = &node.node_type {
                    // Handle text input for query
                    if response.has_focus() {
                        ui.ctx().input(|i| {
                            for event in &i.events {
                                if let egui::Event::Text(text) = event {
                                    if let Some(query) = state.node_queries.get_mut(&selected_id) {
                                        query.push_str(text);
                                    }
                                }
                                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                                    if let Some(query) = state.node_queries.get_mut(&selected_id) {
                                        match key {
                                            egui::Key::Backspace => {
                                                query.pop();
                                            }
                                            egui::Key::Enter => {
                                                if modifiers.ctrl || modifiers.command {
                                                    // Execute query on Ctrl+Enter
                                                    state.execute_node_query(selected_id);
                                                } else {
                                                    query.push('\n');
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }
    
    fn handle_select_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2, to_screen: impl Fn(Pos2) -> Pos2, event_tx: &Sender<AppEvent>) {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen(pos);
                
                // First check if clicking on a resize handle of selected node
                if let Some(selected_id) = state.selected_node {
                    if let Some(node) = state.get_canvas_node(selected_id) {
                        if let Some(handle) = self.get_resize_handle_at_pos(node, canvas_pos, to_screen) {
                            self.resizing_node = Some((selected_id, handle));
                            self.resize_start_size = node.size;
                            self.resize_start_pos = node.position;
                            return;
                        }
                    }
                }
                
                // Then check if clicking on a node
                for (id, node) in &state.canvas_nodes {
                    let node_rect = Rect::from_min_size(
                        Pos2::new(node.position.x, node.position.y),
                        node.size,
                    );
                    
                    if node_rect.contains(canvas_pos) {
                        self.dragging_node = Some(*id);
                        self.drag_offset = canvas_pos.to_vec2() - node.position;
                        state.selected_node = Some(*id);
                        let _ = event_tx.send(AppEvent::NodeSelected(*id));
                        break;
                    }
                }
                
                // If clicking on empty space, deselect
                if self.dragging_node.is_none() && self.resizing_node.is_none() {
                    state.selected_node = None;
                }
            }
        }
        
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen(pos);
                
                // Handle node resizing
                if let Some((node_id, handle)) = self.resizing_node {
                    if let Some(node) = state.get_canvas_node_mut(node_id) {
                        match handle {
                            ResizeHandle::TopLeft => {
                                let new_size = self.resize_start_size - Vec2::new(canvas_pos.x - self.resize_start_pos.x, canvas_pos.y - self.resize_start_pos.y);
                                node.size = Vec2::new(new_size.x.max(MIN_NODE_SIZE), new_size.y.max(MIN_NODE_SIZE));
                                node.position = Vec2::new(
                                    self.resize_start_pos.x + (self.resize_start_size.x - node.size.x),
                                    self.resize_start_pos.y + (self.resize_start_size.y - node.size.y)
                                );
                            }
                            ResizeHandle::TopRight => {
                                let new_width = canvas_pos.x - node.position.x;
                                let new_height = self.resize_start_size.y - (canvas_pos.y - self.resize_start_pos.y);
                                node.size = Vec2::new(new_width.max(MIN_NODE_SIZE), new_height.max(MIN_NODE_SIZE));
                                node.position.y = self.resize_start_pos.y + (self.resize_start_size.y - node.size.y);
                            }
                            ResizeHandle::BottomLeft => {
                                let new_width = self.resize_start_size.x - (canvas_pos.x - self.resize_start_pos.x);
                                let new_height = canvas_pos.y - node.position.y;
                                node.size = Vec2::new(new_width.max(MIN_NODE_SIZE), new_height.max(MIN_NODE_SIZE));
                                node.position.x = self.resize_start_pos.x + (self.resize_start_size.x - node.size.x);
                            }
                            ResizeHandle::BottomRight => {
                                let new_size = canvas_pos.to_vec2() - node.position;
                                node.size = Vec2::new(new_size.x.max(MIN_NODE_SIZE), new_size.y.max(MIN_NODE_SIZE));
                            }
                        }
                    }
                }
                // Handle node dragging
                else if let Some(node_id) = self.dragging_node {
                    if let Some(node) = state.get_canvas_node_mut(node_id) {
                        node.position = canvas_pos.to_vec2() - self.drag_offset;
                        let _ = event_tx.send(AppEvent::NodeMoved { 
                            id: node_id, 
                            position: node.position 
                        });
                    }
                }
            }
        }
        
        if response.drag_stopped() {
            self.dragging_node = None;
            self.resizing_node = None;
        }
        
        // Double-click to create connections
        if response.double_clicked() {
            if let Some(node_id) = state.selected_node {
                if let Some(node) = state.get_canvas_node(node_id) {
                    if let CanvasNodeType::Table { .. } = &node.node_type {
                        self.connecting_from = Some(node_id);
                    }
                }
            }
        }
    }
    
    fn handle_shape_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2, shape_type: ShapeType) {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.drawing_start = Some(from_screen(pos));
                // Create preview shape immediately
                let id = NodeId::new();
                self.preview_shape = Some(id);
                
                // Add initial shape at mouse position
                let start_pos = from_screen(pos);
                let canvas_node = CanvasNode {
                    id,
                    position: start_pos.to_vec2(),
                    size: Vec2::new(1.0, 1.0), // Start with minimal size
                    node_type: CanvasNodeType::Shape { shape_type },
                };
                state.canvas_nodes.insert(id, canvas_node);
            }
        }
        
        // Update shape during drag (including hover)
        if let (Some(start_pos), Some(shape_id)) = (self.drawing_start, self.preview_shape) {
            if let Some(pos) = response.hover_pos() {
                let end_pos = from_screen(pos);
                let size = (end_pos - start_pos).abs();
                
                // Always update the shape, even for small sizes
                let canvas_node = CanvasNode {
                    id: shape_id,
                    position: start_pos.to_vec2().min(end_pos.to_vec2()),
                    size: size.max(Vec2::new(1.0, 1.0)), // Ensure minimum size
                    node_type: CanvasNodeType::Shape { shape_type },
                };
                
                // Update the preview shape
                state.canvas_nodes.insert(shape_id, canvas_node);
            }
        }
        
        if response.drag_stopped() {
            // Finalize the shape or remove if too small
            if let Some(shape_id) = self.preview_shape {
                if let Some(node) = state.get_canvas_node(shape_id) {
                    if node.size.x < 5.0 || node.size.y < 5.0 {
                        // Remove shape if too small
                        state.canvas_nodes.remove(&shape_id);
                    }
                }
            }
            
            self.drawing_start = None;
            self.preview_shape = None;
        }
    }
    
    fn handle_line_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2) {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.drawing_start = Some(from_screen(pos));
            }
        }
        
        if response.drag_stopped() {
            if let Some(start_pos) = self.drawing_start {
                if let Some(pos) = response.interact_pointer_pos() {
                    let end_pos = from_screen(pos);
                    let id = NodeId::new();
                    let canvas_node = CanvasNode {
                        id,
                        position: start_pos.to_vec2(),
                        size: Vec2::new(1.0, 1.0), // Lines don't really have size
                        node_type: CanvasNodeType::Shape { 
                            shape_type: ShapeType::Line { end: (end_pos - start_pos) }
                        },
                    };
                    state.canvas_nodes.insert(id, canvas_node);
                }
            }
            self.drawing_start = None;
        }
    }
    
    fn handle_draw_tool(&mut self, response: &Response, _state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2, painter: &Painter, to_screen: impl Fn(Pos2) -> Pos2) {
        if response.drag_started() {
            self.current_stroke.clear();
            if let Some(pos) = response.interact_pointer_pos() {
                self.current_stroke.push(from_screen(pos));
            }
        }
        
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.current_stroke.push(from_screen(pos));
            }
        }
        
        // Draw current stroke
        if !self.current_stroke.is_empty() {
            let screen_points: Vec<_> = self.current_stroke.iter()
                .map(|&p| to_screen(p))
                .collect();
            painter.add(egui::Shape::line(
                screen_points,
                Stroke::new(2.0, Color32::from_gray(100)),
            ));
        }
        
        if response.drag_stopped() && !self.current_stroke.is_empty() {
            // TODO: Store the stroke as a permanent drawing
            self.current_stroke.clear();
        }
    }
    
    fn handle_text_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2) {
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen(pos);
                let id = NodeId::new();
                let canvas_node = CanvasNode {
                    id,
                    position: canvas_pos.to_vec2(),
                    size: Vec2::new(100.0, 30.0),
                    node_type: CanvasNodeType::Note { content: "New note".to_string() },
                };
                state.canvas_nodes.insert(id, canvas_node);
                state.selected_node = Some(id);
            }
        }
    }
    
    fn show_node_context_menu(&mut self, ui: &mut Ui, state: &mut AppState, node_id: NodeId) {
        if let Some(node) = state.get_canvas_node(node_id) {
            match &node.node_type {
                CanvasNodeType::Table { .. } => {
                    ui.label("Query (Ctrl+Enter to execute):");
                    if let Some(query) = state.node_queries.get_mut(&node_id) {
                        ui.add(egui::TextEdit::multiline(query)
                            .code_editor()
                            .desired_width(250.0)
                            .desired_rows(3));
                    }
                    
                    if ui.button("▶ Execute Query").clicked() {
                        state.execute_node_query(node_id);
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    ui.label("Create Plot:");
                    ui.separator();
                    
                    // Basic plots
                    if ui.button("📊 Histogram").clicked() {
                        self.create_plot_from_table(state, node_id, "Histogram");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("📈 Line Plot").clicked() {
                        self.create_plot_from_table(state, node_id, "Line");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("📉 Scatter Plot").clicked() {
                        self.create_plot_from_table(state, node_id, "Scatter");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("📊 Bar Chart").clicked() {
                        self.create_plot_from_table(state, node_id, "Bar");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    // Statistical plots
                    if ui.button("📦 Box Plot").clicked() {
                        self.create_plot_from_table(state, node_id, "BoxPlot");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("🎻 Violin Plot").clicked() {
                        self.create_plot_from_table(state, node_id, "Violin");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("🔥 Heatmap").clicked() {
                        self.create_plot_from_table(state, node_id, "Heatmap");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("🔗 Correlation").clicked() {
                        self.create_plot_from_table(state, node_id, "Correlation");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    // Advanced plots
                    if ui.button("⏱ Time Series").clicked() {
                        self.create_plot_from_table(state, node_id, "TimeSeries");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("🕸 Radar Chart").clicked() {
                        self.create_plot_from_table(state, node_id, "Radar");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                }
                _ => {
                    if ui.button("Delete").clicked() {
                        state.canvas_nodes.remove(&node_id);
                        state.connections.retain(|c| c.from != node_id && c.to != node_id);
                        if state.selected_node == Some(node_id) {
                            state.selected_node = None;
                        }
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                }
            }
        }
    }
    
    fn show_canvas_context_menu(&mut self, ui: &mut Ui, state: &mut AppState, pos: Pos2) {
        if ui.button("Add Note").clicked() {
            let id = NodeId::new();
            let canvas_node = CanvasNode {
                id,
                position: pos.to_vec2(),
                size: Vec2::new(200.0, 150.0),
                node_type: CanvasNodeType::Note { content: "New note".to_string() },
            };
            state.canvas_nodes.insert(id, canvas_node);
            self.context_menu_pos = None;
            ui.close_menu();
        }
        
        ui.separator();
        
        ui.menu_button("Add Shape", |ui| {
            if ui.button("Rectangle").clicked() {
                let id = NodeId::new();
                let canvas_node = CanvasNode {
                    id,
                    position: pos.to_vec2(),
                    size: DEFAULT_SHAPE_SIZE,
                    node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle },
                };
                state.canvas_nodes.insert(id, canvas_node);
                self.context_menu_pos = None;
                ui.close_menu();
            }
            if ui.button("Circle").clicked() {
                let id = NodeId::new();
                let canvas_node = CanvasNode {
                    id,
                    position: pos.to_vec2(),
                    size: Vec2::new(120.0, 120.0),
                    node_type: CanvasNodeType::Shape { shape_type: ShapeType::Circle },
                };
                state.canvas_nodes.insert(id, canvas_node);
                self.context_menu_pos = None;
                ui.close_menu();
            }
        });
        
        if !state.data_nodes.is_empty() {
            ui.separator();
            ui.menu_button("Add Data Source", |ui| {
                // Collect data nodes info to avoid borrow checker issues
                let data_nodes_info: Vec<_> = state.data_nodes.iter()
                    .map(|node| node.table_info.clone())
                    .collect();
                
                for table_info in data_nodes_info {
                    if ui.button(&table_info.name).clicked() {
                        let node_id = NodeId::new();
                        let canvas_node = CanvasNode {
                            id: node_id,
                            position: pos.to_vec2(),
                            size: DEFAULT_TABLE_SIZE,
                            node_type: CanvasNodeType::Table { 
                                table_info: table_info.clone()
                            },
                        };
                        state.canvas_nodes.insert(node_id, canvas_node);
                        state.load_data_preview(node_id);
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                }
            });
        }
    }
    
    fn create_plot_from_table(&self, state: &mut AppState, table_id: NodeId, plot_type: &str) {
        if let Some(table_node) = state.get_canvas_node(table_id) {
            let plot_id = NodeId::new();
            
            // Count existing plots connected to this table to offset position
            let existing_plots = state.connections.iter()
                .filter(|conn| conn.from == table_id)
                .count();
            let offset_y = (existing_plots as f32) * 30.0;
            
            let plot_node = CanvasNode {
                id: plot_id,
                position: table_node.position + Vec2::new(450.0, offset_y),
                size: DEFAULT_PLOT_SIZE,
                node_type: CanvasNodeType::Plot { plot_type: plot_type.to_string() },
            };
            state.canvas_nodes.insert(plot_id, plot_node);
            
            // Create connection
            state.add_connection(table_id, plot_id, ConnectionType::DataFlow);
        }
    }
    
    fn draw_node(&self, painter: &Painter, node: &CanvasNode, to_screen: impl Fn(Pos2) -> Pos2, selected: bool, state: &AppState) {
        let rect = Rect::from_min_size(
            to_screen(Pos2::new(node.position.x, node.position.y)),
            node.size
        );
        
        match &node.node_type {
            CanvasNodeType::Table { table_info } => {
                // Draw table node with data preview
                painter.rect_filled(
                    rect,
                    5.0,
                    if selected { Color32::from_rgb(50, 50, 70) } else { Color32::from_rgb(35, 35, 50) },
                );
                painter.rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(100, 150, 250) } else { Color32::from_gray(80) }),
                );
                
                // Title bar
                let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 30.0));
                painter.rect_filled(title_rect, 5.0, Color32::from_rgb(45, 45, 65));
                
                painter.text(
                    title_rect.min + Vec2::new(10.0, 8.0),
                    egui::Align2::LEFT_TOP,
                    &table_info.name,
                    FontId::proportional(14.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Query editor area
                let query_height = 60.0;
                let query_rect = Rect::from_min_size(
                    rect.min + Vec2::new(5.0, 35.0),
                    Vec2::new(rect.width() - 10.0, query_height),
                );
                
                painter.rect_filled(query_rect, 2.0, Color32::from_rgb(25, 25, 35));
                
                // Draw query text
                if let Some(query) = state.node_queries.get(&node.id) {
                    painter.text(
                        query_rect.min + Vec2::new(5.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        query,
                        FontId::monospace(12.0 * self.zoom),
                        Color32::from_gray(200),
                    );
                    
                    // Show cursor if selected
                    if selected {
                        let cursor_pos = query_rect.min + Vec2::new(5.0 + query.len() as f32 * 7.0, 5.0);
                        painter.line_segment(
                            [cursor_pos, cursor_pos + Vec2::new(0.0, 14.0)],
                            Stroke::new(1.0, Color32::WHITE),
                        );
                    }
                } else {
                    painter.text(
                        query_rect.min + Vec2::new(5.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        "SELECT * FROM table LIMIT 10",
                        FontId::monospace(12.0 * self.zoom),
                        Color32::from_gray(120),
                    );
                }
                
                // Hint text
                painter.text(
                    query_rect.max - Vec2::new(5.0, 5.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "Ctrl+Enter to run",
                    FontId::proportional(10.0 * self.zoom),
                    Color32::from_gray(100),
                );
                
                // Data preview area (below query)
                let data_rect = Rect::from_min_size(
                    rect.min + Vec2::new(5.0, 100.0),
                    Vec2::new(rect.width() - 10.0, rect.height() - 105.0),
                );
                
                painter.rect_filled(data_rect, 2.0, Color32::from_rgb(20, 20, 30));
                
                // Get or create preview data
                if let Some(preview) = state.node_data.get(&node.id) {
                    // Draw headers
                    if let Some(headers) = &preview.headers {
                        let header_y = data_rect.min.y + 5.0;
                        let col_width = data_rect.width() / headers.len().max(1) as f32;
                        
                        // Header background
                        painter.rect_filled(
                            Rect::from_min_size(
                                data_rect.min,
                                Vec2::new(data_rect.width(), 25.0)
                            ),
                            0.0,
                            Color32::from_rgb(30, 30, 45)
                        );
                        
                        for (i, header) in headers.iter().enumerate() {
                            painter.text(
                                Pos2::new(data_rect.min.x + i as f32 * col_width + 5.0, header_y),
                                egui::Align2::LEFT_TOP,
                                header,
                                FontId::proportional(12.0 * self.zoom),
                                Color32::from_gray(220),
                            );
                        }
                        
                        // Draw rows
                        if let Some(rows) = &preview.rows {
                            let row_height = 20.0 * self.zoom;
                            let max_rows = ((data_rect.height() - 30.0) / row_height) as usize;
                            
                            for (row_idx, row) in rows.iter().take(max_rows).enumerate() {
                                let row_y = header_y + 25.0 + row_idx as f32 * row_height;
                                
                                // Alternating row colors
                                if row_idx % 2 == 1 {
                                    painter.rect_filled(
                                        Rect::from_min_size(
                                            Pos2::new(data_rect.min.x, row_y - 3.0),
                                            Vec2::new(data_rect.width(), row_height)
                                        ),
                                        0.0,
                                        Color32::from_rgb(28, 28, 38)
                                    );
                                }
                                
                                for (col_idx, cell) in row.iter().enumerate() {
                                    painter.text(
                                        Pos2::new(data_rect.min.x + col_idx as f32 * col_width + 5.0, row_y),
                                        egui::Align2::LEFT_TOP,
                                        cell,
                                        FontId::proportional(11.0 * self.zoom),
                                        Color32::from_gray(180),
                                    );
                                }
                            }
                            
                            // Show row count
                            painter.text(
                                data_rect.min + Vec2::new(5.0, data_rect.height() - 20.0),
                                egui::Align2::LEFT_TOP,
                                &format!("Showing {} of {} rows", rows.len().min(max_rows), rows.len()),
                                FontId::proportional(10.0 * self.zoom),
                                Color32::from_gray(120),
                            );
                        }
                    }
                } else {
                    // No data yet - show placeholder
                    painter.text(
                        data_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Loading data...",
                        FontId::proportional(12.0 * self.zoom),
                        Color32::from_gray(100),
                    );
                }
                
                // Query area (only if selected)
                if selected {
                    // Query editing would go here if needed
                }
            }
            CanvasNodeType::Plot { plot_type } => {
                // Draw plot node
                painter.rect_filled(
                    rect,
                    5.0,
                    if selected { Color32::from_rgb(80, 60, 60) } else { Color32::from_rgb(60, 40, 40) },
                );
                painter.rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(250, 150, 100) } else { Color32::from_gray(100) }),
                );
                
                // Title bar
                let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 25.0));
                painter.rect_filled(title_rect, 5.0, Color32::from_rgb(70, 50, 50));
                
                painter.text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("📊 {}", plot_type),
                    FontId::proportional(14.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Plot preview area
                let plot_rect = Rect::from_min_size(
                    rect.min + Vec2::new(5.0, 30.0),
                    Vec2::new(rect.width() - 10.0, rect.height() - 35.0),
                );
                painter.rect_filled(plot_rect, 2.0, Color32::from_rgb(40, 30, 30));
                
                // Check if connected to a data source
                let has_data = state.connections.iter().any(|conn| conn.to == node.id);
                
                if has_data {
                    // Draw placeholder visualization
                    match plot_type.as_str() {
                        "Histogram" => {
                            // Draw simple bars
                            let bar_width = plot_rect.width() / 5.0;
                            for i in 0..5 {
                                let height = (i as f32 + 1.0) * 0.15 * plot_rect.height();
                                let bar_rect = Rect::from_min_size(
                                    plot_rect.min + Vec2::new(i as f32 * bar_width + 2.0, plot_rect.height() - height),
                                    Vec2::new(bar_width - 4.0, height),
                                );
                                painter.rect_filled(bar_rect, 2.0, Color32::from_rgb(100, 150, 200));
                            }
                        }
                        "Line" | "Scatter" => {
                            // Draw simple line
                            painter.line_segment(
                                [
                                    plot_rect.min + Vec2::new(10.0, plot_rect.height() - 10.0),
                                    plot_rect.max - Vec2::new(10.0, 10.0),
                                ],
                                Stroke::new(2.0, Color32::from_rgb(100, 200, 150)),
                            );
                        }
                        _ => {
                            painter.text(
                                plot_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "📊",
                                FontId::proportional(32.0 * self.zoom),
                                Color32::from_gray(150),
                            );
                        }
                    }
                } else {
                    painter.text(
                        plot_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "No data",
                        FontId::proportional(12.0 * self.zoom),
                        Color32::from_gray(150),
                    );
                }
            }
            CanvasNodeType::Note { content } => {
                // Draw note node
                painter.rect_filled(
                    rect,
                    5.0,
                    if selected { Color32::from_rgb(80, 80, 60) } else { Color32::from_rgb(60, 60, 40) },
                );
                painter.rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(250, 250, 100) } else { Color32::from_gray(100) }),
                );
                
        painter.text(
                    rect.min + Vec2::new(10.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    content,
                    FontId::proportional(12.0 * self.zoom),
                    Color32::WHITE,
                );
            }
            CanvasNodeType::Shape { shape_type } => {
                // Draw shapes
                let shape_color = if self.preview_shape == Some(node.id) {
                    Color32::from_gray(100) // Lighter color for preview
                } else {
                    Color32::from_gray(150)
                };
                let stroke = Stroke::new(2.0, shape_color);
                
                match shape_type {
                    ShapeType::Rectangle => {
                        painter.rect_stroke(rect, 0.0, stroke);
                    }
                    ShapeType::Circle => {
                        painter.circle_stroke(rect.center(), rect.width().min(rect.height()) / 2.0, stroke);
                    }
                    ShapeType::Line { end } => {
                        let start = to_screen(Pos2::new(node.position.x, node.position.y));
                        let end = to_screen(Pos2::new(node.position.x + end.x, node.position.y + end.y));
                        painter.line_segment([start, end], stroke);
                    }
                    ShapeType::Arrow { end } => {
                        let start = to_screen(Pos2::new(node.position.x, node.position.y));
                        let end = to_screen(Pos2::new(node.position.x + end.x, node.position.y + end.y));
                        painter.arrow(start, end - start, stroke);
                    }
                }
                
                // Draw selection box for shapes
                if selected {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::from_rgb(100, 150, 250)),
                    );
                }
            }
        }
    }
    
    fn draw_grid(&self, painter: &Painter, rect: &Rect) {
        let grid_size = 20.0 * self.zoom;
        let grid_color = Color32::from_gray(30);
        
        // Calculate grid bounds
        let left = rect.left() - (rect.left() - self.pan_offset.x).rem_euclid(grid_size);
        let top = rect.top() - (rect.top() - self.pan_offset.y).rem_euclid(grid_size);
        
        // Draw vertical lines
        let mut x = left;
            while x < rect.right() {
                painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, grid_color),
                );
                x += grid_size;
            }
            
        // Draw horizontal lines
        let mut y = top;
            while y < rect.bottom() {
                painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, grid_color),
                );
                y += grid_size;
        }
    }
    
    fn bezier_points(&self, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, segments: usize) -> Vec<Pos2> {
        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            
            let x = mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x;
            let y = mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y;
            
            points.push(Pos2::new(x, y));
        }
        points
    }
    
    fn get_resize_handle_at_pos(&self, node: &CanvasNode, pos: Pos2, to_screen: impl Fn(Pos2) -> Pos2) -> Option<ResizeHandle> {
        let handles = [
            (ResizeHandle::TopLeft, Pos2::new(node.position.x, node.position.y)),
            (ResizeHandle::TopRight, Pos2::new(node.position.x + node.size.x, node.position.y)),
            (ResizeHandle::BottomLeft, Pos2::new(node.position.x, node.position.y + node.size.y)),
            (ResizeHandle::BottomRight, Pos2::new(node.position.x + node.size.x, node.position.y + node.size.y)),
        ];
        
        for (handle, handle_pos) in handles {
            let dist = (pos.to_vec2() - handle_pos.to_vec2()).length();
            if dist <= RESIZE_HANDLE_SIZE {
                return Some(handle);
            }
        }
        None
    }
    
    fn draw_resize_handles(&self, painter: &Painter, node: &CanvasNode, to_screen: impl Fn(Pos2) -> Pos2) {
        let handles = [
            Pos2::new(node.position.x, node.position.y),
            Pos2::new(node.position.x + node.size.x, node.position.y),
            Pos2::new(node.position.x, node.position.y + node.size.y),
            Pos2::new(node.position.x + node.size.x, node.position.y + node.size.y),
        ];
        
        for handle_pos in handles {
            let screen_pos = to_screen(handle_pos);
            painter.circle_filled(
                screen_pos,
                RESIZE_HANDLE_SIZE / 2.0,
                Color32::from_rgb(100, 150, 250),
            );
            painter.circle_stroke(
                screen_pos,
                RESIZE_HANDLE_SIZE / 2.0,
                Stroke::new(1.0, Color32::WHITE),
            );
        }
    }
}