//! Canvas panel for node-based data visualization.

use crate::{
    panels::canvas_panel::AppEvent,
    state::{AppState, CanvasNode, CanvasNodeType, ShapeType, ToolMode, ConnectionType},
};

use egui::{Ui, Painter, Pos2, Vec2, Color32, Stroke, Rect, Response, Sense, FontId, Shape, Area};

use std::collections::{HashMap, HashSet};
use pika_core::NodeId;
use tokio::sync::broadcast::Sender;

// Constants for canvas rendering
const MIN_NODE_SIZE: f32 = 50.0;
const RESIZE_HANDLE_SIZE: f32 = 8.0;
const CONNECTION_LINE_WIDTH: f32 = 2.0;
const CONNECTION_ARROW_SIZE: f32 = 10.0;
const GRID_SIZE: f32 = 20.0;
const VISIBLE_MARGIN: f32 = 100.0;
const CELL_SIZE: f32 = 100.0;
const DEFAULT_PLOT_SIZE: Vec2 = Vec2::new(450.0, 350.0);
const DEFAULT_TABLE_SIZE: Vec2 = Vec2::new(600.0, 400.0);
const DEFAULT_SHAPE_SIZE: Vec2 = Vec2::new(150.0, 100.0);

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Spatial index cell for fast hit testing
struct SpatialCell {
    nodes: Vec<NodeId>,
}

// Button rectangles for table nodes
struct TableButtonRects {
    previous: Rect,
    next: Rect,
    execute: Rect,
    create_plot: Rect,
    export_page: Rect,
    export_all: Rect,
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
    
    /// Performance optimizations
    visible_rect: Rect,
    dirty_nodes: HashSet<NodeId>,
    spatial_index: HashMap<(i32, i32), SpatialCell>,
    cached_grid: Option<Vec<Shape>>,
    frame_count: u64,
    last_interaction_frame: u64,
    
    /// Table button rectangles for click detection
    table_button_rects: HashMap<NodeId, TableButtonRects>,
    show_plot_menu_for_node: Option<NodeId>,
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
            visible_rect: Rect::ZERO,
            dirty_nodes: HashSet::new(),
            spatial_index: HashMap::new(),
            cached_grid: None,
            frame_count: 0,
            last_interaction_frame: 0,
            table_button_rects: HashMap::new(),
            show_plot_menu_for_node: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        self.frame_count += 1;
        
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
        
        // Track if we're interacting for performance optimization
        let is_interacting = response.dragged() || response.clicked() || 
                           response.double_clicked() || response.hovered();
        if is_interacting {
            self.last_interaction_frame = self.frame_count;
        }
        
        // Handle middle mouse pan
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan_offset += response.drag_delta();
            state.canvas_state.pan_offset = self.pan_offset;
            self.cached_grid = None; // Invalidate grid cache
        }
        
        // Handle zoom with scroll wheel
        response.ctx.input(|i| {
        if response.hovered() {
                if i.raw_scroll_delta.y != 0.0 {
                    let zoom_delta = 1.0 + i.raw_scroll_delta.y * 0.01;
                    self.zoom = (self.zoom * zoom_delta).clamp(0.1, 5.0);
                    // Zoom towards mouse position
                    state.canvas_state.zoom = self.zoom;
                    self.cached_grid = None; // Invalidate grid cache
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
        
        // Draw grid (cached for performance)
        if state.canvas_state.show_grid {
            if self.cached_grid.is_none() {
                self.cached_grid = Some(self.create_grid_shapes(&response.rect));
            }
            if let Some(grid_shapes) = &self.cached_grid {
                painter.extend(grid_shapes.clone());
            }
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
        
        // Handle connection creation
        if let Some(from_id) = self.connecting_from {
            // Draw temporary connection line
            if let Some(from_node) = state.get_canvas_node(from_id) {
                let from_pos = to_screen(Pos2::new(from_node.position.x + from_node.size.x,
                                                   from_node.position.y + from_node.size.y / 2.0));
                let mouse_pos = response.hover_pos().unwrap_or(from_pos);
                
                // Draw temporary line to mouse
                painter.line_segment(
                    [from_pos, mouse_pos],
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 250).linear_multiply(0.5)),
                );
                
                // Check if clicking on another node to complete connection
                if response.clicked() {
                    if let Some(to_node_id) = self.find_node_at_pos(state, from_screen(mouse_pos)) {
                        if to_node_id != from_id {
                            // Create connection
                            state.add_connection(from_id, to_node_id, ConnectionType::DataFlow);
                            ui.ctx().request_repaint();
                        }
                    }
                    // Cancel connection creation
                    self.connecting_from = None;
                }
                
                // Cancel on right click or escape
                if response.secondary_clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.connecting_from = None;
                }
            }
        }
        
        // Double-click to create connections
        if response.double_clicked() && self.connecting_from.is_none() {
            if let Some(node_id) = state.selected_node {
                if let Some(node) = state.get_canvas_node(node_id) {
                    if let CanvasNodeType::Table { .. } = &node.node_type {
                        self.connecting_from = Some(node_id);
                    }
                }
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
        
        // Update visible rect for frustum culling
        self.visible_rect = Rect::from_min_max(
            from_screen(response.rect.min),
            from_screen(response.rect.max)
        );
        
        // Clear button rects before drawing nodes
        self.table_button_rects.clear();
        
        // Temporary collection for button rects
        let mut new_button_rects = Vec::new();
        
        // Draw canvas nodes with frustum culling
        for (node_id, node) in &state.canvas_nodes {
            // Skip nodes outside visible area
            let node_rect = Rect::from_min_size(
                Pos2::new(node.position.x, node.position.y),
                node.size
            );
            
            if !self.visible_rect.intersects(node_rect) {
                continue; // Skip invisible nodes
            }
            
            let is_selected = state.selected_node == Some(*node_id);
            let button_rects = self.draw_node(&painter, node, to_screen, is_selected, state);
            
            // Collect button rects if any
            if let Some(rects) = button_rects {
                new_button_rects.push((*node_id, rects));
            }
            
            // Draw resize handles for selected nodes
            if is_selected && state.tool_mode == ToolMode::Select {
                self.draw_resize_handles(&painter, node, to_screen);
            }
        }
        
        // Update button rects after the loop
        for (node_id, rects) in new_button_rects {
            self.table_button_rects.insert(node_id, rects);
        }
        
        // Handle right-click context menu with improved hit testing
        if response.clicked_by(egui::PointerButton::Secondary) {
            if let Some(pos) = response.interact_pointer_pos() {
                self.context_menu_pos = Some(from_screen(pos));
            }
        }
        
        // Show context menu if position is set
        if let Some(menu_pos) = self.context_menu_pos {
            let screen_pos = to_screen(menu_pos);
            
            // Use spatial index for faster node hit testing
            let clicked_node = self.find_node_at_pos(state, menu_pos);
            
            // Create a fixed area for the context menu
            let menu_area = egui::Area::new(ui.id().with("context_menu"))
                .fixed_pos(screen_pos)
                .order(egui::Order::Foreground)
                .interactable(true);
                
            menu_area.show(ui.ctx(), |ui| {
                ui.set_min_width(150.0);
                
                // Check if we should show plot menu
                if let Some(plot_node_id) = self.show_plot_menu_for_node {
                    self.show_plot_creation_menu(ui, state, plot_node_id);
                    self.show_plot_menu_for_node = None;
                } else if let Some(node_id) = clicked_node {
                    self.show_node_context_menu(ui, state, node_id);
                } else {
                    self.show_canvas_context_menu(ui, state, menu_pos);
                }
            });
            
            // Close menu if we click elsewhere
            if response.clicked() && self.context_menu_pos.is_some() {
                self.context_menu_pos = None;
            }
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
        let canvas_pos = from_screen(response.hover_pos().unwrap_or(Pos2::ZERO));
        
        // Check for table button clicks first
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let click_pos = from_screen(pos);
                
                // Check each table node's buttons
                for (node_id, button_rects) in &self.table_button_rects {
                    // Convert button rects to screen space for comparison
                    let screen_pos = to_screen(click_pos);
                    
                    // Previous button
                    if button_rects.previous.contains(screen_pos) {
                        if let Some(preview) = state.node_data.get_mut(node_id) {
                            if preview.current_page > 0 {
                                preview.current_page -= 1;
                                state.execute_node_query_with_pagination(*node_id);
                            }
                        }
                        return;
                    }
                    
                    // Next button
                    if button_rects.next.contains(screen_pos) {
                        if let Some(preview) = state.node_data.get(node_id) {
                            let total_pages = preview.total_rows
                                .map(|total| ((total as f32) / preview.page_size as f32).ceil() as usize)
                                .unwrap_or(1);
                            if preview.current_page + 1 < total_pages {
                                if let Some(preview_mut) = state.node_data.get_mut(node_id) {
                                    preview_mut.current_page += 1;
                                    state.execute_node_query_with_pagination(*node_id);
                                }
                            }
                        }
                        return;
                    }
                    
                    // Execute button
                    if button_rects.execute.contains(screen_pos) {
                        state.execute_node_query_with_pagination(*node_id);
                        return;
                    }
                    
                    // Create Plot button
                    if button_rects.create_plot.contains(screen_pos) {
                        // Set a flag to show plot menu for this node
                        self.context_menu_pos = Some(screen_pos);
                        state.selected_node = Some(*node_id);
                        self.show_plot_menu_for_node = Some(*node_id);
                        return;
                    }
                    
                    // Export Page button
                    if button_rects.export_page.contains(screen_pos) {
                        // TODO: Implement export page
                        println!("Export page for node {:?}", node_id);
                        return;
                    }
                    
                    // Export All button
                    if button_rects.export_all.contains(screen_pos) {
                        // TODO: Implement export all
                        println!("Export all for node {:?}", node_id);
                        return;
                    }
                }
            }
        }
        
        // Rest of the select tool handling...
        // Check if clicking on resize handle
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
                
                // Then check if clicking on a node (optimized)
                if let Some(clicked_node) = self.find_node_at_pos(state, canvas_pos) {
                    self.dragging_node = Some(clicked_node);
                    if let Some(node) = state.get_canvas_node(clicked_node) {
                        self.drag_offset = canvas_pos.to_vec2() - node.position;
                    }
                    state.selected_node = Some(clicked_node);
                    let _ = event_tx.send(AppEvent::NodeSelected(clicked_node));
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
        // Get node type first to avoid borrow issues
        let node_type = state.get_canvas_node(node_id).map(|n| n.node_type.clone());
        
        if let Some(node_type) = node_type {
            match node_type {
                CanvasNodeType::Table { table_info: _ } => {
                    // Query editor
                    ui.label("Query Editor:");
                    let mut query = state.node_queries.get(&node_id).cloned().unwrap_or_default();
                    
                    let response = ui.text_edit_multiline(&mut query);
                    if response.changed() {
                        state.node_queries.insert(node_id, query.clone());
                    }
                    
                    ui.separator();
                    
                    // Add plot creation options
                    self.show_plot_creation_menu(ui, state, node_id);
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
    
    fn draw_node(&self, painter: &Painter, node: &CanvasNode, to_screen: impl Fn(Pos2) -> Pos2, selected: bool, state: &AppState) -> Option<TableButtonRects> {
        let rect = Rect::from_min_size(
            to_screen(Pos2::new(node.position.x, node.position.y)),
            node.size
        );
        
        match &node.node_type {
            CanvasNodeType::Table { table_info } => {
                // Draw table node with Pebble-style design
                painter.rect_filled(
                    rect,
                    8.0,
                    if selected { Color32::from_rgb(45, 45, 45) } else { Color32::from_rgb(35, 35, 35) },
                );
                painter.rect_stroke(
                    rect,
                    8.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(100, 150, 250) } else { Color32::from_gray(60) }),
                );
                
                // Title bar with table name
                let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 40.0));
                painter.rect_filled(
                    title_rect, 
                    egui::Rounding::same(8.0),
                    Color32::from_rgb(50, 50, 50)
                );
                
                painter.text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &table_info.name,
                    FontId::proportional(16.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // SQL Query display area
                let query_rect = Rect::from_min_size(
                    rect.min + Vec2::new(10.0, 50.0),
                    Vec2::new(rect.width() - 20.0, 40.0),
                );
                
                painter.rect_filled(query_rect, 4.0, Color32::from_rgb(25, 25, 25));
                painter.rect_stroke(query_rect, 4.0, Stroke::new(1.0, Color32::from_gray(60)));
                
                // Get query or use default
                let query = state.node_queries.get(&node.id)
                    .cloned()
                    .unwrap_or_else(|| format!("SELECT * FROM '{}'", table_info.name));
                
                painter.text(
                    query_rect.min + Vec2::new(10.0, 10.0),
                    egui::Align2::LEFT_TOP,
                    &format!("SQL Query:\n{}", query),
                    FontId::monospace(12.0 * self.zoom),
                    Color32::from_gray(200),
                );
                
                // Results info
                let row_count = table_info.row_count.unwrap_or(0);
                let showing_rows = 25; // Default page size
                let current_page = 1; // TODO: Track current page in state
                let total_pages = (row_count as f32 / showing_rows as f32).ceil() as usize;
                
                painter.text(
                    rect.min + Vec2::new(10.0, 100.0),
                    egui::Align2::LEFT_TOP,
                    &format!("Results: {} rows (showing {}-{})", 
                        row_count,
                        (current_page - 1) * showing_rows + 1,
                        (current_page * showing_rows).min(row_count)
                    ),
                    FontId::proportional(13.0 * self.zoom),
                    Color32::from_gray(180),
                );
                
                // Data table area
                let table_rect = Rect::from_min_size(
                    rect.min + Vec2::new(10.0, 125.0),
                    Vec2::new(rect.width() - 20.0, rect.height() - 180.0),
                );
                
                painter.rect_filled(table_rect, 4.0, Color32::from_rgb(20, 20, 20));
                painter.rect_stroke(table_rect, 4.0, Stroke::new(1.0, Color32::from_gray(60)));
                
                // Get or create preview data
                if let Some(preview) = state.node_data.get(&node.id) {
                    // Draw headers
                    if let Some(headers) = &preview.headers {
                        let header_height = 30.0;
                        let header_rect = Rect::from_min_size(
                            table_rect.min,
                            Vec2::new(table_rect.width(), header_height)
                        );
                        
                        // Header background
                        painter.rect_filled(header_rect, 0.0, Color32::from_rgb(30, 30, 30));
                        
                        let col_width = table_rect.width() / headers.len().max(1) as f32;
                        
                        for (i, header) in headers.iter().enumerate() {
                            let text_pos = Pos2::new(
                                table_rect.min.x + i as f32 * col_width + 10.0,
                                table_rect.min.y + 8.0
                            );
                            
                            painter.text(
                                text_pos,
                                egui::Align2::LEFT_TOP,
                                header,
                                FontId::proportional(13.0 * self.zoom),
                                Color32::from_gray(220),
                            );
                            
                            // Column separator
                            if i > 0 {
                                painter.line_segment(
                                    [
                                        Pos2::new(table_rect.min.x + i as f32 * col_width, table_rect.min.y),
                                        Pos2::new(table_rect.min.x + i as f32 * col_width, table_rect.min.y + header_height),
                                    ],
                                    Stroke::new(1.0, Color32::from_gray(50)),
                                );
                            }
                        }
                        
                        // Header separator
                        painter.line_segment(
                            [
                                Pos2::new(table_rect.min.x, table_rect.min.y + header_height),
                                Pos2::new(table_rect.max.x, table_rect.min.y + header_height),
                            ],
                            Stroke::new(1.0, Color32::from_gray(70)),
                        );
                        
                        // Draw rows
                        if let Some(rows) = &preview.rows {
                            let row_height = 25.0;
                            let data_start_y = table_rect.min.y + header_height;
                            let max_rows = ((table_rect.height() - header_height) / row_height) as usize;
                            
                            for (row_idx, row) in rows.iter().take(max_rows).enumerate() {
                                let row_y = data_start_y + row_idx as f32 * row_height;
                                
                                // Alternating row colors
                                if row_idx % 2 == 1 {
                                    painter.rect_filled(
                                        Rect::from_min_size(
                                            Pos2::new(table_rect.min.x, row_y),
                                            Vec2::new(table_rect.width(), row_height)
                                        ),
                                        0.0,
                                        Color32::from_rgb(25, 25, 25)
                                    );
                                }
                                
                                for (col_idx, cell) in row.iter().enumerate() {
                                    let text_pos = Pos2::new(
                                        table_rect.min.x + col_idx as f32 * col_width + 10.0,
                                        row_y + 5.0
                                    );
                                    
                                    painter.text(
                                        text_pos,
                                        egui::Align2::LEFT_TOP,
                                        cell,
                                        FontId::proportional(12.0 * self.zoom),
                                        Color32::from_gray(180),
                                    );
                                    
                                    // Column separator
                                    if col_idx > 0 {
                                        painter.line_segment(
                                            [
                                                Pos2::new(table_rect.min.x + col_idx as f32 * col_width, row_y),
                                                Pos2::new(table_rect.min.x + col_idx as f32 * col_width, row_y + row_height),
                                            ],
                                            Stroke::new(1.0, Color32::from_gray(40)),
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No data yet - show placeholder
                    painter.text(
                        table_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Click Execute to run query",
                        FontId::proportional(14.0 * self.zoom),
                        Color32::from_gray(120),
                    );
                }
                
                // Pagination controls at bottom
                let pagination_y = rect.max.y - 40.0;
                let button_width = 80.0;
                let button_height = 25.0;
                
                // Store button interactions for later processing
                let mut clicked_previous = false;
                let mut clicked_next = false;
                let mut clicked_execute = false;
                let mut clicked_create_plot = false;
                let mut clicked_export_page = false;
                let mut clicked_export_all = false;
                
                // Previous button
                let prev_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 10.0, pagination_y),
                    Vec2::new(button_width, button_height)
                );
                
                let prev_enabled = state.node_data.get(&node.id)
                    .map(|d| d.current_page > 0)
                    .unwrap_or(false);
                    
                painter.rect_filled(
                    prev_rect, 
                    4.0, 
                    if prev_enabled { Color32::from_rgb(40, 40, 40) } else { Color32::from_rgb(25, 25, 25) }
                );
                painter.rect_stroke(
                    prev_rect, 
                    4.0, 
                    Stroke::new(1.0, if prev_enabled { Color32::from_gray(80) } else { Color32::from_gray(40) })
                );
                painter.text(
                    prev_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Previous",
                    FontId::proportional(12.0 * self.zoom),
                    if prev_enabled { Color32::from_gray(200) } else { Color32::from_gray(100) },
                );
                
                // Page info
                let current_page = state.node_data.get(&node.id)
                    .map(|d| d.current_page + 1)
                    .unwrap_or(1);
                let total_pages = state.node_data.get(&node.id)
                    .and_then(|d| d.total_rows.map(|total| ((total as f32) / d.page_size as f32).ceil() as usize))
                    .unwrap_or(1);
                    
                painter.text(
                    Pos2::new(rect.min.x + 100.0, pagination_y + 5.0),
                    egui::Align2::LEFT_TOP,
                    &format!("Page {} of {}", current_page, total_pages.max(1)),
                    FontId::proportional(12.0 * self.zoom),
                    Color32::from_gray(180),
                );
                
                // Next button
                let next_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 200.0, pagination_y),
                    Vec2::new(button_width, button_height)
                );
                
                let next_enabled = state.node_data.get(&node.id)
                    .map(|_d| current_page < total_pages)
                    .unwrap_or(false);
                    
                painter.rect_filled(
                    next_rect, 
                    4.0, 
                    if next_enabled { Color32::from_rgb(40, 40, 40) } else { Color32::from_rgb(25, 25, 25) }
                );
                painter.rect_stroke(
                    next_rect, 
                    4.0, 
                    Stroke::new(1.0, if next_enabled { Color32::from_gray(80) } else { Color32::from_gray(40) })
                );
                painter.text(
                    next_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Next",
                    FontId::proportional(12.0 * self.zoom),
                    if next_enabled { Color32::from_gray(200) } else { Color32::from_gray(100) },
                );
                
                // Execute button (moved here from query area)
                let execute_rect = Rect::from_min_size(
                    Pos2::new(rect.center().x - 45.0, pagination_y),
                    Vec2::new(90.0, button_height)
                );
                painter.rect_filled(execute_rect, 4.0, Color32::from_rgb(50, 100, 50));
                painter.rect_stroke(execute_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(80, 150, 80)));
                painter.text(
                    execute_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Execute",
                    FontId::proportional(12.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Create Plot button  
                let create_plot_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - 310.0, pagination_y),
                    Vec2::new(100.0, button_height)
                );
                painter.rect_filled(create_plot_rect, 4.0, Color32::from_rgb(50, 50, 100));
                painter.rect_stroke(create_plot_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(100, 100, 150)));
                painter.text(
                    create_plot_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Create Plot",
                    FontId::proportional(12.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Export buttons on the right
                let export_page_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - 200.0, pagination_y),
                    Vec2::new(90.0, button_height)
                );
                painter.rect_filled(export_page_rect, 4.0, Color32::from_rgb(40, 40, 40));
                painter.rect_stroke(export_page_rect, 4.0, Stroke::new(1.0, Color32::from_gray(80)));
                painter.text(
                    export_page_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Export Page",
                    FontId::proportional(12.0 * self.zoom),
                    Color32::from_gray(200),
                );
                
                let export_all_rect = Rect::from_min_size(
                    Pos2::new(rect.max.x - 100.0, pagination_y),
                    Vec2::new(90.0, button_height)
                );
                painter.rect_filled(export_all_rect, 4.0, Color32::from_rgb(40, 40, 40));
                painter.rect_stroke(export_all_rect, 4.0, Stroke::new(1.0, Color32::from_gray(80)));
                painter.text(
                    export_all_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Export All",
                    FontId::proportional(12.0 * self.zoom),
                    Color32::from_gray(200),
                );
                
                // Check for button clicks (we need to handle this in the interaction logic)
                // For now, store the button rects for click detection later
                Some(TableButtonRects {
                    previous: prev_rect,
                    next: next_rect,
                    execute: execute_rect,
                    create_plot: create_plot_rect,
                    export_page: export_page_rect,
                    export_all: export_all_rect,
                })
            }
            CanvasNodeType::Plot { plot_type } => {
                // Draw plot node with professional styling
                painter.rect_filled(
                    rect,
                    8.0,
                    if selected { Color32::from_rgb(45, 45, 45) } else { Color32::from_rgb(35, 35, 35) },
                );
                painter.rect_stroke(
                    rect,
                    8.0,
                    Stroke::new(2.0, if selected { Color32::from_rgb(250, 150, 100) } else { Color32::from_gray(60) }),
                );
                
                // Title bar
                let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 40.0));
                painter.rect_filled(
                    title_rect, 
                    egui::Rounding::same(8.0),
                    Color32::from_rgb(50, 50, 50)
                );
                
                painter.text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("📊 {} Plot", plot_type),
                    FontId::proportional(16.0 * self.zoom),
                    Color32::WHITE,
                );
                
                // Plot area with margins for axes
                let plot_margin = 50.0;
                let plot_rect = Rect::from_min_size(
                    rect.min + Vec2::new(plot_margin, 50.0),
                    Vec2::new(rect.width() - plot_margin - 20.0, rect.height() - 100.0),
                );
                painter.rect_filled(plot_rect, 4.0, Color32::from_rgb(25, 25, 25));
                painter.rect_stroke(plot_rect, 4.0, Stroke::new(1.0, Color32::from_gray(50)));
                
                // Draw axes
                let axes_color = Color32::from_gray(150);
                let axes_stroke = Stroke::new(2.0, axes_color);
                
                // Y-axis
                painter.line_segment(
                    [plot_rect.left_bottom(), plot_rect.left_top()],
                    axes_stroke,
                );
                
                // X-axis
                painter.line_segment(
                    [plot_rect.left_bottom(), plot_rect.right_bottom()],
                    axes_stroke,
                );
                
                // Axis labels
                painter.text(
                    Pos2::new(plot_rect.center().x, plot_rect.max.y + 20.0),
                    egui::Align2::CENTER_TOP,
                    "X Axis",
                    FontId::proportional(12.0 * self.zoom),
                    axes_color,
                );
                
                // Y-axis label (rotated text would be ideal, but for now just place it)
                painter.text(
                    Pos2::new(plot_rect.min.x - 30.0, plot_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    "Y",
                    FontId::proportional(12.0 * self.zoom),
                    axes_color,
                );
                
                // Check if connected to a data source
                let has_data = state.connections.iter().any(|conn| conn.to == node.id);
                
                if has_data {
                    // Draw plot visualization based on type
                    match plot_type.as_str() {
                        "Histogram" => {
                            // Draw histogram bars with spacing
                            let num_bars = 8;
                            let bar_width = plot_rect.width() / (num_bars as f32 * 1.5);
                            let spacing = bar_width * 0.2;
                            
                            for i in 0..num_bars {
                                let height = ((i as f32 + 1.0) / num_bars as f32) * plot_rect.height() * 0.8;
                                let x = plot_rect.min.x + (i as f32 * (bar_width + spacing)) + spacing;
                                let bar_rect = Rect::from_min_size(
                                    Pos2::new(x, plot_rect.max.y - height),
                                    Vec2::new(bar_width, height),
                                );
                                painter.rect_filled(bar_rect, 2.0, Color32::from_rgb(100, 150, 200));
                                painter.rect_stroke(bar_rect, 2.0, Stroke::new(1.0, Color32::from_rgb(150, 200, 250)));
                            }
                            
                            // Draw grid lines
                            for i in 1..5 {
                                let y = plot_rect.min.y + (i as f32 / 5.0) * plot_rect.height();
                                painter.line_segment(
                                    [Pos2::new(plot_rect.min.x, y), Pos2::new(plot_rect.max.x, y)],
                                    Stroke::new(1.0, Color32::from_gray(40)),
                                );
                            }
                        }
                        "Line" => {
                            // Draw a sample line chart
                            let points = vec![
                                Pos2::new(plot_rect.min.x + 20.0, plot_rect.max.y - 30.0),
                                Pos2::new(plot_rect.min.x + plot_rect.width() * 0.25, plot_rect.min.y + plot_rect.height() * 0.3),
                                Pos2::new(plot_rect.center().x, plot_rect.min.y + plot_rect.height() * 0.4),
                                Pos2::new(plot_rect.min.x + plot_rect.width() * 0.75, plot_rect.min.y + plot_rect.height() * 0.2),
                                Pos2::new(plot_rect.max.x - 20.0, plot_rect.min.y + plot_rect.height() * 0.5),
                            ];
                            
                            // Draw line
                            for i in 0..points.len() - 1 {
                                painter.line_segment(
                                    [points[i], points[i + 1]],
                                    Stroke::new(2.0, Color32::from_rgb(100, 200, 150)),
                                );
                            }
                            
                            // Draw points
                            for point in &points {
                                painter.circle_filled(*point, 4.0, Color32::from_rgb(150, 250, 200));
                                painter.circle_stroke(*point, 4.0, Stroke::new(1.0, Color32::from_rgb(100, 200, 150)));
                            }
                        }
                        "Scatter" => {
                            // Draw sample scatter plot
                            let num_points = 20;
                            for i in 0..num_points {
                                let x = plot_rect.min.x + (i as f32 / num_points as f32) * plot_rect.width();
                                let y = plot_rect.min.y + ((i * 7 % 11) as f32 / 11.0) * plot_rect.height();
                                painter.circle_filled(Pos2::new(x, y), 3.0, Color32::from_rgb(200, 150, 100));
                                painter.circle_stroke(Pos2::new(x, y), 3.0, Stroke::new(1.0, Color32::from_rgb(250, 200, 150)));
                            }
                        }
                        _ => {
                            // Generic plot placeholder
                            painter.text(
                                plot_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "📊",
                                FontId::proportional(48.0 * self.zoom),
                                Color32::from_gray(100),
                            );
                        }
                    }
                    
                    // Draw tick marks and labels
                    for i in 0..5 {
                        // X-axis ticks
                        let x = plot_rect.min.x + (i as f32 / 4.0) * plot_rect.width();
                        painter.line_segment(
                            [Pos2::new(x, plot_rect.max.y), Pos2::new(x, plot_rect.max.y + 5.0)],
                            axes_stroke,
                        );
                        painter.text(
                            Pos2::new(x, plot_rect.max.y + 8.0),
                            egui::Align2::CENTER_TOP,
                            &format!("{}", i * 25),
                            FontId::proportional(10.0 * self.zoom),
                            axes_color,
                        );
                        
                        // Y-axis ticks
                        let y = plot_rect.max.y - (i as f32 / 4.0) * plot_rect.height();
                        painter.line_segment(
                            [Pos2::new(plot_rect.min.x - 5.0, y), Pos2::new(plot_rect.min.x, y)],
                            axes_stroke,
                        );
                        painter.text(
                            Pos2::new(plot_rect.min.x - 10.0, y),
                            egui::Align2::RIGHT_CENTER,
                            &format!("{}", i * 25),
                            FontId::proportional(10.0 * self.zoom),
                            axes_color,
                        );
                    }
                } else {
                    painter.text(
                        plot_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Connect to a data source",
                        FontId::proportional(14.0 * self.zoom),
                        Color32::from_gray(120),
                    );
                }
                None
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
                None
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
                None
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
            (ResizeHandle::TopLeft, node.position.to_pos2()),
            (ResizeHandle::TopRight, Pos2::new(node.position.x + node.size.x, node.position.y)),
            (ResizeHandle::BottomLeft, Pos2::new(node.position.x, node.position.y + node.size.y)),
            (ResizeHandle::BottomRight, (node.position + node.size).to_pos2()),
        ];
        
        for (handle_type, handle_pos) in handles {
            let screen_pos = to_screen(handle_pos);
            if (screen_pos - pos).length() <= RESIZE_HANDLE_SIZE {
                return Some(handle_type);
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
    
    // Performance optimization: Create grid shapes once and cache them
    fn create_grid_shapes(&self, rect: &Rect) -> Vec<Shape> {
        let mut shapes = Vec::new();
        let grid_size = 50.0 * self.zoom;
        let grid_color = Color32::from_gray(40);
        
        // Calculate grid bounds
        let start_x = ((rect.min.x - self.pan_offset.x) / grid_size).floor() * grid_size + self.pan_offset.x;
        let start_y = ((rect.min.y - self.pan_offset.y) / grid_size).floor() * grid_size + self.pan_offset.y;
        let end_x = rect.max.x;
        let end_y = rect.max.y;
        
        // Vertical lines
        let mut x = start_x;
        while x <= end_x {
            shapes.push(Shape::LineSegment {
                points: [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                stroke: Stroke::new(1.0, grid_color).into(),
            });
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = start_y;
        while y <= end_y {
            shapes.push(Shape::LineSegment {
                points: [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                stroke: Stroke::new(1.0, grid_color).into(),
            });
            y += grid_size;
        }
        
        shapes
    }
    
    // Performance optimization: Find node at position using spatial index
    fn find_node_at_pos(&self, state: &AppState, pos: Pos2) -> Option<NodeId> {
        // For now, use linear search, but this could be optimized with spatial indexing
        for (id, node) in &state.canvas_nodes {
            let node_rect = Rect::from_min_size(
                Pos2::new(node.position.x, node.position.y),
                node.size,
            );
            if node_rect.contains(pos) {
                return Some(*id);
            }
        }
        None
    }
    
    // Rebuild spatial index for fast hit testing
    fn rebuild_spatial_index(&mut self, state: &AppState) {
        self.spatial_index.clear();
        
        let cell_size = 100.0; // Size of spatial cells
        
        for (id, node) in &state.canvas_nodes {
            let min_cell_x = (node.position.x / cell_size).floor() as i32;
            let min_cell_y = (node.position.y / cell_size).floor() as i32;
            let max_cell_x = ((node.position.x + node.size.x) / cell_size).ceil() as i32;
            let max_cell_y = ((node.position.y + node.size.y) / cell_size).ceil() as i32;
            
            for cell_x in min_cell_x..=max_cell_x {
                for cell_y in min_cell_y..=max_cell_y {
                    self.spatial_index
                        .entry((cell_x, cell_y))
                        .or_insert_with(|| SpatialCell { nodes: Vec::new() })
                        .nodes
                        .push(*id);
                }
            }
        }
    }

    fn show_plot_creation_menu(&mut self, ui: &mut Ui, state: &mut AppState, node_id: NodeId) {
        ui.label("Create Plot:");
        
        // Common plot types
        if ui.button("📊 Histogram").clicked() {
            self.create_plot_from_table(state, node_id, "Histogram");
            ui.close_menu();
        }
        if ui.button("📈 Line Plot").clicked() {
            self.create_plot_from_table(state, node_id, "Line");
            ui.close_menu();
        }
        if ui.button("⚪ Scatter Plot").clicked() {
            self.create_plot_from_table(state, node_id, "Scatter");
            ui.close_menu();
        }
        if ui.button("📊 Bar Chart").clicked() {
            self.create_plot_from_table(state, node_id, "Bar");
            ui.close_menu();
        }
        
        ui.separator();
        
        // More plot types
        ui.menu_button("More Plots", |ui| {
            if ui.button("Box Plot").clicked() {
                self.create_plot_from_table(state, node_id, "BoxPlot");
                ui.close_menu();
            }
            if ui.button("Violin Plot").clicked() {
                self.create_plot_from_table(state, node_id, "Violin");
                ui.close_menu();
            }
            if ui.button("Heatmap").clicked() {
                self.create_plot_from_table(state, node_id, "Heatmap");
                ui.close_menu();
            }
            if ui.button("Correlation Matrix").clicked() {
                self.create_plot_from_table(state, node_id, "Correlation");
                ui.close_menu();
            }
            if ui.button("Time Series").clicked() {
                self.create_plot_from_table(state, node_id, "TimeSeries");
                ui.close_menu();
            }
            if ui.button("Radar Chart").clicked() {
                self.create_plot_from_table(state, node_id, "Radar");
                ui.close_menu();
            }
        });
    }
}