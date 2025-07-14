//! Canvas panel for node-based data visualization.

use crate::state::{AppState, NodeConnection, ConnectionType, CanvasNode, CanvasNodeType, ShapeType, ToolMode};
use pika_core::types::NodeId;
use tokio::sync::broadcast::Sender;
use egui::{Context, Ui, Painter, Pos2, Vec2, Color32, Stroke, Rect, Response, Sense, FontId, menu};
use crate::panels::canvas_panel::AppEvent;
use egui_extras::{TableBuilder, Column};

/// Canvas panel for node-based visualization.
pub struct CanvasPanel {
    /// Dragging state
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    
    /// Connection creation state
    connecting_from: Option<NodeId>,
    
    /// Canvas transform
    pan_offset: Vec2,
    zoom: f32,
    
    /// Drawing state
    drawing_start: Option<Pos2>,
    current_stroke: Vec<Pos2>,
    
    /// Context menu position
    context_menu_pos: Option<Pos2>,
}

impl CanvasPanel {
    pub fn new(_ctx: std::sync::Arc<pika_core::events::EventBus>) -> Self {
        Self {
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            connecting_from: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            drawing_start: None,
            current_stroke: Vec::new(),
            context_menu_pos: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            Sense::click_and_drag(),
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
        
        // Draw canvas nodes
        let nodes: Vec<_> = state.canvas_nodes.keys().cloned().collect();
        for node_id in nodes {
            if let Some(node) = state.get_canvas_node(node_id) {
                self.draw_node(&painter, node, to_screen, state.selected_node == Some(node_id));
            }
        }
        
        // Tool-specific handling
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
    }
    
    fn handle_select_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2, _to_screen: impl Fn(Pos2) -> Pos2, event_tx: &Sender<AppEvent>) {
        if response.clicked() {
            let click_pos = response.hover_pos().unwrap();
            let canvas_pos = from_screen(click_pos);
            
            // Check if we clicked on a node
            let mut _clicked_node = None;
            for (id, node) in &state.canvas_nodes {
                let node_rect = Rect::from_min_size(
                    Pos2::new(node.position.x, node.position.y),
                    node.size
                );
                if node_rect.contains(canvas_pos) {
                    _clicked_node = Some(*id);
                state.selected_node = Some(*id);
                    let _ = event_tx.send(AppEvent::NodeSelected(*id));
                    break;
                }
            }
            
            // Clear selection if clicked on empty space
            if response.clicked() && self.dragging_node.is_none() {
                state.selected_node = None;
            }
        }
        
        // Handle node dragging
        if response.dragged() && self.dragging_node.is_some() {
            if let Some(pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen(pos);
                if let Some(node_id) = self.dragging_node {
                    if let Some(node) = state.get_canvas_node_mut(node_id) {
                        node.position = (canvas_pos + self.drag_offset).to_vec2();
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
        }
    }
    
    fn handle_shape_tool(&mut self, response: &Response, state: &mut AppState, from_screen: impl Fn(Pos2) -> Pos2, shape_type: ShapeType) {
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.drawing_start = Some(from_screen(pos));
            }
        }
        
        if response.drag_stopped() {
            if let Some(start_pos) = self.drawing_start {
                if let Some(pos) = response.interact_pointer_pos() {
                    let end_pos = from_screen(pos);
                    let size = (end_pos - start_pos).abs();
                    
                    if size.x > 5.0 && size.y > 5.0 {
                        let id = NodeId::new();
                        let canvas_node = CanvasNode {
                            id,
                            position: start_pos.to_vec2().min(end_pos.to_vec2()),
                            size,
                            node_type: CanvasNodeType::Shape { shape_type },
                        };
                        state.canvas_nodes.insert(id, canvas_node);
                    }
                }
            }
            self.drawing_start = None;
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
                    ui.label("Create Plot:");
                    ui.separator();
                    
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
                    if ui.button("🥧 Pie Chart").clicked() {
                        self.create_plot_from_table(state, node_id, "Pie");
                        self.context_menu_pos = None;
                        ui.close_menu();
                    }
                    if ui.button("🔥 Heatmap").clicked() {
                        self.create_plot_from_table(state, node_id, "Heatmap");
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
                size: Vec2::new(150.0, 100.0),
                node_type: CanvasNodeType::Note { content: "New note".to_string() },
            };
            state.canvas_nodes.insert(id, canvas_node);
            self.context_menu_pos = None;
            ui.close_menu();
        }
    }
    
    fn create_plot_from_table(&self, state: &mut AppState, table_id: NodeId, plot_type: &str) {
        if let Some(table_node) = state.get_canvas_node(table_id) {
            let plot_id = NodeId::new();
            let plot_node = CanvasNode {
                id: plot_id,
                position: table_node.position + Vec2::new(250.0, 0.0),
                size: Vec2::new(200.0, 150.0),
                node_type: CanvasNodeType::Plot { plot_type: plot_type.to_string() },
            };
            state.canvas_nodes.insert(plot_id, plot_node);
            
            // Create connection
            state.add_connection(table_id, plot_id, ConnectionType::DataFlow);
        }
    }
    
    fn draw_node(&self, painter: &Painter, node: &CanvasNode, to_screen: impl Fn(Pos2) -> Pos2, selected: bool) {
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
                    if selected { Color32::from_rgb(60, 60, 80) } else { Color32::from_rgb(40, 40, 60) },
                );
                painter.rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(100, 150, 250) } else { Color32::from_gray(100) }),
                );
                
                // Title
                painter.text(
                    rect.min + Vec2::new(10.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    &table_info.name,
                    FontId::proportional(14.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Mini data preview (placeholder)
                let preview_rect = Rect::from_min_size(
                    rect.min + Vec2::new(5.0, 30.0 * self.zoom),
                    Vec2::new(rect.width() - 10.0, rect.height() - 35.0 * self.zoom),
                );
                painter.rect_filled(preview_rect, 2.0, Color32::from_rgb(30, 30, 40));
                
                // Show column headers
                if !table_info.columns.is_empty() {
                    let text = table_info.columns.iter()
                        .take(3)
                        .map(|c| &c.name[..c.name.len().min(8)])
                        .collect::<Vec<_>>()
                        .join(" | ");
                    painter.text(
                        preview_rect.min + Vec2::new(5.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        text,
                        FontId::monospace(10.0 * self.zoom),
                        Color32::from_gray(200),
                    );
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
                
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("📊 {}", plot_type),
                    FontId::proportional(16.0 * self.zoom),
                    Color32::WHITE,
                );
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
                let stroke = Stroke::new(2.0, if selected { Color32::from_rgb(150, 150, 250) } else { Color32::from_gray(150) });
                
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
}