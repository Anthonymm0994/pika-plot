//! Canvas panel for node-based data visualization.

use crate::{
    panels::canvas_panel::AppEvent,
    state::{AppState, CanvasNode, CanvasNodeType, ToolMode, ConnectionType},
};

use egui::{Ui, Painter, Pos2, Vec2, Color32, Stroke, Rect, Response, Sense, FontId, Shape, Area, vec2};

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
const DEFAULT_TABLE_SIZE: Vec2 = Vec2::new(400.0, 250.0);  // Larger to show data preview
const DEFAULT_SHAPE_SIZE: Vec2 = Vec2::new(150.0, 100.0);

// Smooth animation constants
const PAN_MOMENTUM_DECAY: f32 = 0.92;
const PAN_MOMENTUM_THRESHOLD: f32 = 0.5;
const ZOOM_SMOOTHING: f32 = 0.15;
const DRAG_SMOOTHING: f32 = 0.25;

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
    // View state
    pub pan_offset: Vec2,
    pub zoom: f32,
    
    // Smooth animation state
    pan_velocity: Vec2,
    target_zoom: f32,
    target_pan: Vec2,
    is_panning_with_space: bool,
    last_frame_time: f64,
    
    // Drag state
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    drag_target_pos: Vec2,
    drag_current_pos: Vec2,
    
    // Connection creation
    connecting_from: Option<NodeId>,
    
    // Resize state
    resizing_node: Option<(NodeId, ResizeHandle)>,
    resize_start_pos: Vec2,
    resize_start_size: Vec2,
    
    // Drawing state
    drawing_start: Option<Pos2>,
    current_stroke: Vec<Pos2>,
    preview_shape: Option<NodeId>,
    
    // UI state
    context_menu_pos: Option<Pos2>,
    
    // Performance optimization
    visible_rect: Rect,
    dirty_nodes: HashSet<NodeId>,
    spatial_index: HashMap<(i32, i32), SpatialCell>,
    cached_grid: Option<Vec<Shape>>,
    frame_count: u64,
    last_interaction_frame: u64,
    
    // Button tracking for table nodes
    table_button_rects: HashMap<NodeId, TableButtonRects>,
    show_plot_menu_for_node: Option<NodeId>,
}

impl CanvasPanel {
    pub fn new(_ctx: std::sync::Arc<pika_core::events::EventBus>) -> Self {
        Self {
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            pan_velocity: Vec2::ZERO,
            target_zoom: 1.0,
            target_pan: Vec2::ZERO,
            is_panning_with_space: false,
            last_frame_time: 0.0,
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            drag_target_pos: Vec2::ZERO,
            drag_current_pos: Vec2::ZERO,
            connecting_from: None,
            resizing_node: None,
            resize_start_pos: Vec2::ZERO,
            resize_start_size: Vec2::ZERO,
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
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, ctx: &egui::Context, event_tx: &Sender<AppEvent>) -> Response {
        // First, handle query windows separately (like Pebble)
        self.show_query_windows(ctx, state);
        
        // Then show the main canvas
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Draw the canvas background and grid
        let painter = ui.painter_at(available_rect);
        painter.rect_filled(available_rect, 0.0, Color32::from_rgb(18, 18, 18));
        
        // Draw grid if enabled
        if state.canvas_state.show_grid {
            self.draw_visible_grid(&painter, &available_rect);
        }
        
        self.frame_count += 1;
        
        // Calculate frame delta time for smooth animations
        let current_time = ui.ctx().input(|i| i.time);
        let delta_time = if self.last_frame_time > 0.0 {
            (current_time - self.last_frame_time).min(0.1) // Cap at 100ms to prevent jumps
        } else {
            0.016 // 60 FPS default
        };
        self.last_frame_time = current_time;
        
        // Update cursor based on current state
        self.update_cursor(ui, &response, state);
        
        // Smooth zoom animation
        if (self.zoom - self.target_zoom).abs() > 0.001 {
            self.zoom = self.zoom + (self.target_zoom - self.zoom) * ZOOM_SMOOTHING;
            state.canvas_state.zoom = self.zoom;
            self.cached_grid = None;
            ui.ctx().request_repaint();
        }
        
        // Apply pan momentum
        if self.pan_velocity.length() > PAN_MOMENTUM_THRESHOLD {
            self.pan_offset += self.pan_velocity * delta_time as f32 * 60.0;
            self.pan_velocity *= PAN_MOMENTUM_DECAY;
            state.canvas_state.pan_offset = self.pan_offset;
            self.cached_grid = None;
            ui.ctx().request_repaint();
        } else {
            self.pan_velocity = Vec2::ZERO;
        }
        
        // Track if we're interacting for performance optimization
        let is_interacting = response.dragged() || response.clicked() || 
                           response.double_clicked() || response.hovered();
        if is_interacting {
            self.last_interaction_frame = self.frame_count;
        }
        
        // Handle spacebar + drag for pan (Photoshop style)
        let space_pressed = ui.input(|i| i.key_down(egui::Key::Space));
        if space_pressed && response.drag_started() {
            self.is_panning_with_space = true;
        }
        if !space_pressed {
            self.is_panning_with_space = false;
        }
        
        // Handle panning (middle mouse OR space+drag)
        let is_panning = response.dragged_by(egui::PointerButton::Middle) || 
                        (self.is_panning_with_space && response.dragged());
        
        if is_panning {
            let delta = response.drag_delta();
            self.pan_offset += delta;
            self.pan_velocity = delta / delta_time as f32;
            state.canvas_state.pan_offset = self.pan_offset;
            self.cached_grid = None;
        } else if response.drag_stopped() {
            // Keep momentum when drag stops
            // Velocity is already set during dragging
        }
        
        // Handle zoom with scroll wheel - zoom to mouse position
        response.ctx.input(|i| {
            if response.hovered() && i.raw_scroll_delta.y != 0.0 {
                let zoom_delta = 1.0 + i.raw_scroll_delta.y * 0.01;
                let new_zoom = (self.target_zoom * zoom_delta).clamp(0.1, 5.0);
                
                // Zoom towards mouse position
                if let Some(mouse_pos) = response.hover_pos() {
                    let mouse_canvas_before = (mouse_pos - response.rect.min - self.pan_offset) / self.zoom;
                    self.target_zoom = new_zoom;
                    let mouse_canvas_after = (mouse_pos - response.rect.min - self.pan_offset) / new_zoom;
                    self.pan_offset += (mouse_canvas_after - mouse_canvas_before) * new_zoom;
                    state.canvas_state.pan_offset = self.pan_offset;
                }
                
                self.cached_grid = None;
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
        
        // Double-click handled in handle_select_tool for table nodes
        
        // Tool-specific handling BEFORE drawing nodes so preview shapes appear immediately
        match state.tool_mode {
            ToolMode::Select => self.handle_select_tool(&response, state, from_screen, to_screen, event_tx),
            ToolMode::Pan => {
                // Pan is handled above with middle mouse, so nothing extra needed here
            },
                            ToolMode::Rectangle => { /* Shape tools disabled */ },
                ToolMode::Circle => { /* Shape tools disabled */ },
                ToolMode::Line => { /* Line tool disabled */ },
                            ToolMode::Draw => self.handle_draw_tool(&response, state, from_screen, &painter, to_screen),
                ToolMode::Text => { /* Text tool disabled */ },
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

        // Query editing is now handled in the query window, not directly on the canvas
        
        response
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
        
        // Handle node dragging with smooth interpolation
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
                // Handle node dragging with smooth interpolation
                else if let Some(node_id) = self.dragging_node {
                    // Update target position with grid snapping
                    let raw_pos = canvas_pos.to_vec2() - self.drag_offset;
                    let shift_pressed = response.ctx.input(|i| i.modifiers.shift);
                    self.drag_target_pos = self.snap_to_grid(raw_pos, GRID_SIZE, shift_pressed);
                    
                    if let Some(node) = state.get_canvas_node_mut(node_id) {
                        // Initialize current position if needed
                        if self.drag_current_pos == Vec2::ZERO {
                            self.drag_current_pos = node.position;
                        }
                        
                        // Smooth interpolation towards target
                        let diff = self.drag_target_pos - self.drag_current_pos;
                        self.drag_current_pos += diff * DRAG_SMOOTHING;
                        node.position = self.drag_current_pos;
                        
                        // Request repaint if still moving
                        if diff.length() > 0.1 {
                            response.ctx.request_repaint();
                        }
                        
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
            self.drag_current_pos = Vec2::ZERO;
            self.drag_target_pos = Vec2::ZERO;
        }
        
        // Table nodes on canvas don't open query windows - that's done from the data panel
    }
    
    /*
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
    */
    
    /*
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
    */
    
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
    
    /*
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
    */
    
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
        /* Note functionality disabled
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
        */
        
        ui.separator();
        
        /* Shape creation disabled
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
        */
        
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
    
    fn draw_node(&self, painter: &Painter, node: &CanvasNode, to_screen: impl Fn(Pos2) -> Pos2, is_selected: bool, state: &AppState) -> Option<TableButtonRects> {
        let pos = to_screen(Pos2::new(node.position.x, node.position.y));
        let size = node.size * self.zoom;
        let rect = Rect::from_min_size(pos, size);
        
        // Hover effect
        let is_hovered = painter.ctx().pointer_hover_pos()
            .map(|p| rect.contains(p))
            .unwrap_or(false);
        
        match &node.node_type {
            CanvasNodeType::Table { table_info } => {
                // Render table with data preview like Pebble
                painter.rect_filled(
                    rect,
                    4.0,
                    Color32::from_gray(35),  // Slightly lighter background
                );
                
                // Border
                painter.rect_stroke(
                    rect,
                    4.0,
                    if is_selected {
                        Stroke::new(2.0, Color32::from_rgb(100, 150, 200))
                    } else {
                        Stroke::new(1.0, Color32::from_gray(60))
                    },
                );
                
                // Title bar with table name
                let title_height = 25.0;
                let title_rect = Rect::from_min_size(rect.min, vec2(rect.width(), title_height));
                painter.rect_filled(
                    title_rect,
                    egui::Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 },
                    Color32::from_gray(45),
                );
                
                painter.text(
                    title_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &table_info.name,
                    FontId::proportional(13.0),
                    Color32::from_gray(220),
                );
                
                // Draw table data preview
                let data_rect = Rect::from_min_size(
                    rect.min + vec2(0.0, title_height),
                    vec2(rect.width(), rect.height() - title_height)
                );
                
                // Create mock data for preview
                let headers = &table_info.columns[..table_info.columns.len().min(5)]
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>();
                
                let row_height = 22.0;
                let header_height = 25.0;
                let padding = 5.0;
                
                // Header background
                let header_rect = Rect::from_min_size(
                    data_rect.min,
                    vec2(data_rect.width(), header_height)
                );
                painter.rect_filled(
                    header_rect,
                    0.0,
                    Color32::from_gray(50),
                );
                
                // Draw headers
                let col_width = (data_rect.width() - padding * 2.0) / headers.len() as f32;
                for (i, header) in headers.iter().enumerate() {
                    let x = data_rect.min.x + padding + i as f32 * col_width;
                    painter.text(
                        Pos2::new(x + col_width / 2.0, header_rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        header,
                        FontId::proportional(11.0),
                        Color32::from_gray(200),
                    );
                }
                
                // Draw some sample data rows
                let visible_rows = ((data_rect.height() - header_height) / row_height).floor() as usize;
                for row_idx in 0..visible_rows.min(5) {
                    let y = data_rect.min.y + header_height + row_idx as f32 * row_height;
                    
                    // Alternate row colors
                    if row_idx % 2 == 1 {
                        painter.rect_filled(
                            Rect::from_min_size(
                                Pos2::new(data_rect.min.x, y),
                                vec2(data_rect.width(), row_height)
                            ),
                            0.0,
                            Color32::from_gray(40),
                        );
                    }
                    
                    // Draw row data
                    for col_idx in 0..headers.len() {
                        let x = data_rect.min.x + padding + col_idx as f32 * col_width;
                        let sample_value = match (table_info.name.as_str(), col_idx) {
                            ("MOCK_DATA_0", 0) => (row_idx + 1).to_string(),  // ID
                            ("MOCK_DATA_0", 1) => format!("Customer_{:03}", row_idx),  // Name
                            ("MOCK_DATA_0", 2) => format!("customer{}@example.com", row_idx),  // Email
                            ("MOCK_DATA_0", 3) => "2024-01-01".to_string(),  // Date
                            ("MOCK_DATA_0", 4) => format!("{:.2}", 100.0 + row_idx as f64 * 50.0),  // Amount
                            ("MOCK_DATA_0", _) => "...".to_string(),
                            
                            ("MOCK_DATA_1", 0) => (row_idx + 1).to_string(),  // ID
                            ("MOCK_DATA_1", 1) => format!("Product_{}", row_idx),  // Product
                            ("MOCK_DATA_1", 2) => ["Available", "Out of Stock", "Limited"][row_idx % 3].to_string(),  // Status
                            ("MOCK_DATA_1", 3) => format!("{}", 10 + row_idx * 5),  // Quantity
                            ("MOCK_DATA_1", 4) => format!("${:.2}", 19.99 + row_idx as f64 * 10.0),  // Price
                            ("MOCK_DATA_1", _) => "...".to_string(),
                            
                            ("MOCK_DATA_2", 0) => format!("ORD{:04}", row_idx + 1000),  // Order ID
                            ("MOCK_DATA_2", 1) => format!("Customer {}", row_idx + 1),  // Customer
                            ("MOCK_DATA_2", 2) => ["Pending", "Shipped", "Delivered"][row_idx % 3].to_string(),  // Status
                            ("MOCK_DATA_2", 3) => format!("${:.2}", 250.0 + row_idx as f64 * 100.0),  // Total
                            ("MOCK_DATA_2", 4) => "2024-01-15".to_string(),  // Date
                            ("MOCK_DATA_2", _) => "...".to_string(),
                            
                            _ => format!("Row{}_Col{}", row_idx, col_idx),
                        };
                        
                        painter.text(
                            Pos2::new(x + col_width / 2.0, y + row_height / 2.0),
                            egui::Align2::CENTER_CENTER,
                            sample_value,
                            FontId::proportional(10.0),
                            Color32::from_gray(180),
                        );
                    }
                }
                
                // Show row count at bottom
                let info_y = rect.max.y - 15.0;
                painter.text(
                    Pos2::new(rect.center().x, info_y),
                    egui::Align2::CENTER_CENTER,
                    format!("{} rows total", table_info.row_count.unwrap_or(0)),
                    FontId::proportional(10.0),
                    Color32::from_gray(120),
                );
                
                None // No button rects for table nodes
            }
            CanvasNodeType::Plot { plot_type } => {
                // Draw plot node with professional styling
                painter.rect_filled(
                    rect,
                    8.0,
                    if is_selected { Color32::from_rgb(45, 45, 45) } else { Color32::from_rgb(35, 35, 35) },
                );
                painter.rect_stroke(
                    rect,
                    8.0,
                    Stroke::new(2.0, if is_selected { Color32::from_rgb(250, 150, 100) } else { Color32::from_gray(60) }),
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
            /* Note and Shape nodes disabled
            CanvasNodeType::Note { content } => {
                // Draw note node
                painter.rect_filled(
                    rect,
                    5.0,
                    if is_selected { Color32::from_rgb(80, 80, 60) } else { Color32::from_rgb(60, 60, 40) },
                );
                painter.rect_stroke(
                    rect,
                    5.0,
                    Stroke::new(2.0, if is_selected { Color32::from_rgb(250, 250, 100) } else { Color32::from_gray(100) }),
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
                if is_selected {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::from_rgb(100, 150, 250)),
                    );
                }
                None
            }
            */
        }
    }
    
    fn draw_grid(&self, painter: &Painter, rect: &Rect) {
        let grid_size = GRID_SIZE * self.zoom;
        let grid_color = Color32::from_gray(30);
        
        // Extend bounds to ensure grid covers entire visible area
        let extended_rect = rect.expand(VISIBLE_MARGIN);
        
        // Calculate grid bounds with proper offset accounting
        let offset_x = self.pan_offset.x % grid_size;
        let offset_y = self.pan_offset.y % grid_size;
        
        let left = extended_rect.left() - (extended_rect.left() - offset_x).rem_euclid(grid_size);
        let top = extended_rect.top() - (extended_rect.top() - offset_y).rem_euclid(grid_size);
        
        // Draw vertical lines
        let mut x = left;
        while x <= extended_rect.right() {
            painter.line_segment(
                [Pos2::new(x, extended_rect.top()), Pos2::new(x, extended_rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }
            
        // Draw horizontal lines
        let mut y = top;
        while y <= extended_rect.bottom() {
            painter.line_segment(
                [Pos2::new(extended_rect.left(), y), Pos2::new(extended_rect.right(), y)],
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
        let grid_size = GRID_SIZE * self.zoom;
        let grid_color = Color32::from_gray(30);
        
        // Extend bounds to ensure grid covers entire visible area
        let extended_rect = rect.expand(VISIBLE_MARGIN);
        
        // Calculate grid bounds with proper offset accounting
        let offset_x = self.pan_offset.x % grid_size;
        let offset_y = self.pan_offset.y % grid_size;
        
        let left = extended_rect.left() - (extended_rect.left() - offset_x).rem_euclid(grid_size);
        let top = extended_rect.top() - (extended_rect.top() - offset_y).rem_euclid(grid_size);
        
        // Vertical lines
        let mut x = left;
        while x <= extended_rect.right() {
            shapes.push(Shape::LineSegment {
                points: [Pos2::new(x, extended_rect.top()), Pos2::new(x, extended_rect.bottom())],
                stroke: Stroke::new(1.0, grid_color).into(),
            });
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = top;
        while y <= extended_rect.bottom() {
            shapes.push(Shape::LineSegment {
                points: [Pos2::new(extended_rect.left(), y), Pos2::new(extended_rect.right(), y)],
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

    // Add this new method for cursor feedback
    fn update_cursor(&self, ui: &mut Ui, response: &Response, state: &AppState) {
        if !response.hovered() {
            return;
        }
        
        let space_pressed = ui.input(|i| i.key_down(egui::Key::Space));
        
        // Set cursor based on current state and tool
        if space_pressed || response.dragged_by(egui::PointerButton::Middle) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        } else if self.is_panning_with_space {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else {
            match state.tool_mode {
                ToolMode::Select => {
                    // Check if hovering over a node or resize handle
                    if self.dragging_node.is_some() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else if let Some(pos) = response.hover_pos() {
                        let canvas_pos = self.screen_to_canvas(pos, response.rect);
                        
                        // Check resize handles first
                        if let Some(selected_id) = state.selected_node {
                            if let Some(node) = state.get_canvas_node(selected_id) {
                                if let Some(handle) = self.get_resize_handle_at_pos(node, canvas_pos, |p| self.canvas_to_screen(p, response.rect)) {
                                    match handle {
                                        ResizeHandle::TopLeft | ResizeHandle::BottomRight => {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                                        }
                                        ResizeHandle::TopRight | ResizeHandle::BottomLeft => {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                                        }
                                    }
                                    return;
                                }
                            }
                        }
                        
                        // Check if hovering over a node
                        if self.find_node_at_pos(state, canvas_pos).is_some() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        } else {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        }
                    }
                }
                ToolMode::Pan => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
                ToolMode::Draw => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair),
                ToolMode::Text => ui.ctx().set_cursor_icon(egui::CursorIcon::Text),
                _ => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair),
            }
        }
    }
    
    // Helper methods for coordinate conversion
    fn screen_to_canvas(&self, pos: Pos2, rect: Rect) -> Pos2 {
        Pos2::new(
            (pos.x - rect.left() - self.pan_offset.x) / self.zoom,
            (pos.y - rect.top() - self.pan_offset.y) / self.zoom,
        )
    }
    
    fn canvas_to_screen(&self, pos: Pos2, rect: Rect) -> Pos2 {
        Pos2::new(
            pos.x * self.zoom + self.pan_offset.x + rect.left(),
            pos.y * self.zoom + self.pan_offset.y + rect.top(),
        )
    }
    
    // Add grid snapping helper
    fn snap_to_grid(&self, pos: Vec2, grid_size: f32, shift_pressed: bool) -> Vec2 {
        if shift_pressed {
            // Hold shift to disable snapping
            pos
        } else {
            // Snap to grid in canvas space (not screen space)
            Vec2::new(
                (pos.x / GRID_SIZE).round() * GRID_SIZE,
                (pos.y / GRID_SIZE).round() * GRID_SIZE,
            )
        }
    }
    
    // Visual feedback for snapping
    fn draw_snap_guides(&self, painter: &Painter, pos: Pos2, to_screen: impl Fn(Pos2) -> Pos2, shift_pressed: bool) {
        let snapped = self.snap_to_grid(pos.to_vec2(), GRID_SIZE, shift_pressed);
        if (snapped - pos.to_vec2()).length() < 5.0 {
            // Draw snap indicator
            let screen_pos = to_screen(Pos2::new(snapped.x, snapped.y));
            painter.circle_filled(screen_pos, 3.0, Color32::from_rgba_premultiplied(100, 150, 250, 128));
            
            // Draw alignment guides
            let guide_color = Color32::from_rgba_premultiplied(100, 150, 250, 64);
            let guide_stroke = Stroke::new(1.0, guide_color);
            
            // Vertical guide
            painter.line_segment(
                [
                    Pos2::new(screen_pos.x, screen_pos.y - 20.0),
                    Pos2::new(screen_pos.x, screen_pos.y + 20.0),
                ],
                guide_stroke,
            );
            
            // Horizontal guide
            painter.line_segment(
                [
                    Pos2::new(screen_pos.x - 20.0, screen_pos.y),
                    Pos2::new(screen_pos.x + 20.0, screen_pos.y),
                ],
                guide_stroke,
            );
        }
    }
    
    /// Show query windows as separate egui Windows (like Pebble)
    fn show_query_windows(&mut self, ctx: &egui::Context, state: &mut AppState) {
        let mut windows_to_close = Vec::new();
        let mut windows_to_execute = Vec::new();
        
        // Collect the node IDs first to avoid borrow checker issues
        let window_ids: Vec<NodeId> = state.query_windows.keys().cloned().collect();
        
        for node_id in window_ids {
            if let Some(window) = state.query_windows.get_mut(&node_id) {
                if !window.is_open {
                    windows_to_close.push(node_id);
                    continue;
                }
                
                let mut open = true;
                let mut should_execute = false;
                let mut page_changed = false;
                let window_id = node_id; // Copy for use in closure
                
                egui::Window::new(&window.title)
                    .id(egui::Id::new(format!("query_window_{:?}", node_id)))
                    .default_size([600.0, 400.0])
                    .resizable(true)
                    .collapsible(true)
                    .open(&mut open)
                    .show(ctx, |ui| {
                        // Query editor section
                        ui.group(|ui| {
                            ui.label("SQL Query:");
                            
                            let response = egui::TextEdit::multiline(&mut window.query)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(3)
                                .show(ui);
                            
                            // Execute on Ctrl+Enter
                            if response.response.has_focus() 
                                && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                                should_execute = true;
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            if ui.button("▶ Execute").clicked() {
                                should_execute = true;
                            }
                            
                            ui.separator();
                            
                            if ui.button("Export Page").clicked() {
                                // Export current page
                            }
                            
                            if ui.button("Export All").clicked() {
                                // Export all results
                            }
                        });
                        
                        ui.separator();
                        
                        // Error display
                        if let Some(error) = &window.error {
                            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("❌ {}", error));
                            ui.separator();
                        }
                        
                        // Results display
                        if let Some(result) = &window.result {
                            ui.label(format!("Results: {} rows (showing page {} of {})", 
                                result.total_rows, 
                                window.page + 1,
                                (result.total_rows as f32 / window.page_size as f32).ceil() as usize
                            ));
                            
                            // Table display using egui_extras
                            egui::ScrollArea::both()
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    use egui_extras::{TableBuilder, Column};
                                    
                                    TableBuilder::new(ui)
                                        .striped(true)
                                        .resizable(true)
                                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                        .columns(Column::initial(100.0).resizable(true), result.columns.len())
                                        .header(20.0, |mut header| {
                                            for column in &result.columns {
                                                header.col(|ui| {
                                                    ui.strong(column);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for row in &result.rows {
                                                body.row(20.0, |mut row_ui| {
                                                    for value in row {
                                                        row_ui.col(|ui| {
                                                            ui.label(value);
                                                        });
                                                    }
                                                });
                                            }
                                        });
                                });
                            
                            // Pagination controls
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("◀ Previous").clicked() && window.page > 0 {
                                    window.page -= 1;
                                    page_changed = true;
                                }
                                
                                ui.label(format!("Page {} of {}", 
                                    window.page + 1,
                                    (result.total_rows as f32 / window.page_size as f32).ceil() as usize
                                ));
                                
                                if ui.button("Next ▶").clicked() {
                                    let max_page = (result.total_rows as f32 / window.page_size as f32).ceil() as usize - 1;
                                    if window.page < max_page {
                                        window.page += 1;
                                        page_changed = true;
                                    }
                                }
                                
                                ui.separator();
                                
                                ui.label("Page size:");
                                if ui.add(egui::DragValue::new(&mut window.page_size).range(10..=100)).changed() {
                                    window.page = 0; // Reset to first page
                                    page_changed = true;
                                }
                            });
                        }
                    });
                    
                window.is_open = open;
                
                // Track windows that need query execution
                if should_execute || page_changed {
                    windows_to_execute.push(window_id);
                }
            }
        }
        
        // Execute queries for windows that need it
        for node_id in windows_to_execute {
            self.execute_query_for_window(state, node_id);
        }
        
        // Remove closed windows
        for id in windows_to_close {
            state.query_windows.remove(&id);
        }
    }

    fn execute_query_for_window(&mut self, state: &mut AppState, node_id: NodeId) {
        // Get the table info first to avoid borrow checker issues
        let table_info_opt = state.get_canvas_node(node_id)
            .and_then(|node| match &node.node_type {
                CanvasNodeType::Table { table_info } => Some(table_info.clone()),
                _ => None,
            });
            
        if let Some(table_info) = table_info_opt {
            if let Some(window) = state.query_windows.get_mut(&node_id) {
                // Generate mock query results
                let headers: Vec<String> = table_info.columns.iter().map(|c| c.name.clone()).collect();
                let mut rows = Vec::new();
                
                // Calculate rows for current page
                let start_row = window.page * window.page_size;
                let end_row = ((window.page + 1) * window.page_size).min(table_info.row_count.unwrap_or(100));
                
                // Generate mock data for the current page
                for i in start_row..end_row {
                    let mut row = Vec::new();
                    for col in &table_info.columns {
                        let value = match col.data_type.to_uppercase().as_str() {
                            "INTEGER" => (i + 1).to_string(),
                            "TEXT" | "VARCHAR" => {
                                match col.name.to_lowercase().as_str() {
                                    "first_name" => ["John", "Jane", "Bob", "Alice", "Charlie", "David", "Emma", "Frank", "Grace", "Henry"][i % 10].to_string(),
                                    "last_name" => ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"][i % 10].to_string(),
                                    "product_name" => format!("Product {}", i + 1),
                                    "email" => format!("user{}@example.com", i + 1),
                                    "gender" => if i % 2 == 0 { "Male" } else { "Female" }.to_string(),
                                    "ip_address" => format!("192.168.1.{}", (i % 255) + 1),
                                    "category" => ["Electronics", "Clothing", "Food", "Books", "Toys"][i % 5].to_string(),
                                    "location" => ["New York", "Los Angeles", "Chicago", "Houston", "Phoenix"][i % 5].to_string(),
                                    _ => format!("{} {}", col.name, i + 1),
                                }
                            }
                            "REAL" | "FLOAT" | "DOUBLE" => {
                                match col.name.to_lowercase().as_str() {
                                    "price" => format!("{:.2}", (i + 1) as f64 * 9.99),
                                    "temperature" => format!("{:.1}", 20.0 + (i as f64 * 0.5)),
                                    "humidity" => format!("{:.1}", 40.0 + (i as f64 * 0.3)),
                                    _ => format!("{:.2}", (i + 1) as f64 * 10.5),
                                }
                            }
                            "BOOLEAN" | "BOOL" => {
                                if i % 2 == 0 { "true" } else { "false" }.to_string()
                            }
                            _ => format!("Data {}", i + 1),
                        };
                        row.push(value);
                    }
                    rows.push(row);
                }
                
                window.result = Some(crate::state::QueryWindowResult {
                    columns: headers,
                    rows,
                    total_rows: table_info.row_count.unwrap_or(100),
                });
                window.error = None;
            }
        }
    }

    fn draw_visible_grid(&self, painter: &Painter, rect: &Rect) {
        let grid_size = GRID_SIZE * self.zoom;
        let grid_color = Color32::from_gray(30);
        
        // Calculate grid bounds with proper offset accounting
        let offset_x = self.pan_offset.x % grid_size;
        let offset_y = self.pan_offset.y % grid_size;
        
        let left = rect.left() - (rect.left() - offset_x).rem_euclid(grid_size);
        let top = rect.top() - (rect.top() - offset_y).rem_euclid(grid_size);
        
        // Vertical lines
        let mut x = left;
        while x <= rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = top;
        while y <= rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
            y += grid_size;
        }
    }
}