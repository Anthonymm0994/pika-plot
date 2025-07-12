//! Canvas system for visual node-based interface with threads and breadcrumbs.

use egui::{
    Color32, Painter, Pos2, Rect, Response, 
    Sense, Shape, Stroke, Vec2, Widget, epaint::CubicBezierShape,
};
use pika_core::{
    types::{NodeId, Point2, Size2, Connection},
    node::Node,
    events::{CanvasEvent},
    nodes::NodeType,
};
use std::collections::{HashMap, VecDeque};
use tokio::sync::broadcast;

/// Canvas state managing nodes, connections, and viewport
pub struct CanvasState {
    /// All nodes on the canvas (trait objects)
    pub nodes: HashMap<NodeId, Box<dyn Node>>,
    
    /// Connections between nodes (threads)
    pub connections: Vec<ThreadConnection>,
    
    /// Camera/viewport state
    pub camera: Camera2D,
    
    /// Currently selected nodes
    pub selected_nodes: Vec<NodeId>,
    
    /// Drag state
    drag_state: DragState,
    
    /// Breadcrumb trails for each window/view
    breadcrumbs: HashMap<NodeId, BreadcrumbTrail>,
    
    /// Event sender for canvas events
    event_sender: broadcast::Sender<CanvasEvent>,
    
    /// Smart snapping configuration
    pub snap_config: SnapConfig,
}

/// Thread connection between nodes with visual state
#[derive(Debug, Clone)]
pub struct ThreadConnection {
    pub connection: Connection,
    pub color: Color32,
    pub animated: bool,
    pub hover_state: f32, // 0.0 to 1.0 for smooth hover transitions
}

/// Camera/viewport for infinite panning and zooming
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub center: Pos2,
    pub zoom: f32,
    pub target_center: Pos2,
    pub target_zoom: f32,
    pub animation_speed: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            center: Pos2::new(0.0, 0.0),
            zoom: 1.0,
            target_center: Pos2::new(0.0, 0.0),
            target_zoom: 1.0,
            animation_speed: 0.1,
        }
    }
}

impl Camera2D {
    /// Convert world position to screen position
    pub fn world_to_screen(&self, world_pos: Pos2, screen_rect: Rect) -> Pos2 {
        let screen_center = screen_rect.center();
        let offset = (world_pos - self.center) * self.zoom;
        screen_center + offset
    }
    
    /// Convert screen position to world position
    pub fn screen_to_world(&self, screen_pos: Pos2, screen_rect: Rect) -> Pos2 {
        let screen_center = screen_rect.center();
        let offset = (screen_pos - screen_center) / self.zoom;
        self.center + offset
    }
    
    /// Update camera with smooth interpolation
    pub fn update(&mut self) {
        self.center = self.center.lerp(self.target_center, self.animation_speed);
        self.zoom = egui::lerp(self.zoom..=self.target_zoom, self.animation_speed);
    }
    
    /// Pan the camera
    pub fn pan(&mut self, delta: Vec2) {
        self.target_center -= delta / self.zoom;
    }
    
    /// Zoom the camera around a point
    pub fn zoom_around(&mut self, zoom_delta: f32, focus_point: Pos2) {
        let old_zoom = self.target_zoom;
        self.target_zoom = (self.target_zoom * (1.0 + zoom_delta * 0.1)).clamp(0.1, 10.0);
        
        // Adjust center to zoom around focus point
        let zoom_ratio = self.target_zoom / old_zoom;
        let focus_offset = focus_point - self.target_center;
        self.target_center = focus_point - focus_offset * zoom_ratio;
    }
}

/// Drag state for node manipulation
#[derive(Debug, Clone)]
enum DragState {
    None,
    DraggingNode { node_id: NodeId, offset: Vec2 },
    PanningCanvas { start_pos: Pos2 },
    DrawingConnection { from: (NodeId, String), preview_pos: Pos2 },
    SelectionBox { start: Pos2, current: Pos2 },
}

/// Breadcrumb trail showing data flow path
#[derive(Debug, Clone)]
pub struct BreadcrumbTrail {
    pub trail: VecDeque<BreadcrumbItem>,
    pub max_items: usize,
}

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub node_id: NodeId,
    pub label: String,
    pub icon: BreadcrumbIcon,
}

#[derive(Debug, Clone, Copy)]
pub enum BreadcrumbIcon {
    Dataset,
    Query,
    Plot,
    Transform,
    Export,
}



/// Smart snapping configuration
#[derive(Debug, Clone)]
pub struct SnapConfig {
    pub grid_size: f32,
    pub snap_threshold: f32,
    pub alignment_guides: bool,
    pub snap_to_nodes: bool,
}

impl Default for SnapConfig {
    fn default() -> Self {
        SnapConfig {
            grid_size: 20.0,
            snap_threshold: 10.0,
            alignment_guides: true,
            snap_to_nodes: true,
        }
    }
}

impl CanvasState {
    /// Create a new canvas state
    pub fn new(event_sender: broadcast::Sender<CanvasEvent>) -> Self {
        CanvasState {
            nodes: HashMap::new(),
            connections: Vec::new(),
            camera: Camera2D::default(),
            selected_nodes: Vec::new(),
            drag_state: DragState::None,
            breadcrumbs: HashMap::new(),
            event_sender,
            snap_config: SnapConfig::default(),
        }
    }
    
    /// Add a node to the canvas
    pub fn add_node(&mut self, node: Box<dyn Node>) -> NodeId {
        let id = node.id();
        
        // Initialize breadcrumb trail for plot nodes
        if node.type_name() == "PlotNode" {
            let trail = BreadcrumbTrail {
                trail: VecDeque::new(),
                max_items: 5,
            };
            self.breadcrumbs.insert(id, trail);
        }
        
        self.nodes.insert(id, node);
        id
    }
    
    /// Create a connection between nodes
    pub fn create_connection(&mut self, from: (NodeId, String), to: (NodeId, String)) {
        let connection = Connection {
            from: (from.0, pika_core::types::PortId::Output(from.1.clone())),
            to: (to.0, pika_core::types::PortId::Input(to.1.clone())),
        };
        
        // Determine thread color based on connection type
        let color = self.get_thread_color(&connection);
        
        let thread = ThreadConnection {
            connection: connection.clone(),
            color,
            animated: false,
            hover_state: 0.0,
        };
        
        self.connections.push(thread);
        
        // Update breadcrumbs
        self.update_breadcrumbs(from.0, to.0);
        
        // Send event
        let _ = self.event_sender.send(CanvasEvent::ConnectionCreated { 
            from,
            to,
        });
    }
    
    /// Get thread color based on data type
    fn get_thread_color(&self, connection: &Connection) -> Color32 {
        // Color coding based on connection type
        // This is a simplified version - could be enhanced with actual type checking
        match (&self.nodes.get(&connection.from.0), &self.nodes.get(&connection.to.0)) {
            (Some(from), Some(to)) => {
                match (from.type_name(), to.type_name()) {
                    ("TableNode", "QueryNode") => Color32::from_rgb(100, 200, 255), // Blue for data
                    ("QueryNode", "PlotNode") => Color32::from_rgb(255, 150, 100), // Orange for results
                    ("QueryNode", "QueryNode") => Color32::from_rgb(150, 255, 150), // Green for query chains
                    _ => Color32::from_rgb(200, 200, 200), // Gray default
                }
            }
            _ => Color32::from_rgb(200, 200, 200),
        }
    }
    
    /// Update breadcrumb trails when connections change
    fn update_breadcrumbs(&mut self, from: NodeId, to: NodeId) {
        if let Some(to_node) = self.nodes.get(&to) {
            // Only update breadcrumbs for plot nodes
            if to_node.type_name() == "PlotNode" {
                if let Some(from_node) = self.nodes.get(&from) {
                    let item = BreadcrumbItem {
                        node_id: from,
                        label: self.get_node_label(from_node),
                        icon: self.get_node_icon(from_node.type_name()),
                    };
                    
                    if let Some(trail) = self.breadcrumbs.get_mut(&to) {
                        // Add to trail, maintaining max size
                        trail.trail.push_back(item);
                        while trail.trail.len() > trail.max_items {
                            trail.trail.pop_front();
                        }
                    }
                }
            }
        }
    }
    
    /// Get a label for a node
    fn get_node_label(&self, node: &Box<dyn Node>) -> String {
        match node.type_name() {
            "TableNode" => node.name().to_string(),
            "QueryNode" => {
                // Just use the node name
                node.name().to_string()
            }
            "PlotNode" => node.name().to_string(),
            "TransformNode" => node.name().to_string(),
            "ExportNode" => "Export".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    
    /// Get icon for node type
    fn get_node_icon(&self, type_name: &str) -> BreadcrumbIcon {
        match type_name {
            "TableNode" => BreadcrumbIcon::Dataset,
            "QueryNode" => BreadcrumbIcon::Query,
            "PlotNode" => BreadcrumbIcon::Plot,
            "TransformNode" => BreadcrumbIcon::Transform,
            "ExportNode" => BreadcrumbIcon::Export,
            _ => BreadcrumbIcon::Dataset,
        }
    }
    
    /// Apply smart snapping to a position
    fn apply_snap(&self, pos: Pos2) -> Pos2 {
        if !self.snap_config.alignment_guides {
            return pos;
        }
        
        let grid_size = self.snap_config.grid_size;
        let threshold = self.snap_config.snap_threshold;
        
        // Grid snapping
        let grid_x = (pos.x / grid_size).round() * grid_size;
        let grid_y = (pos.y / grid_size).round() * grid_size;
        
        let mut snapped_pos = pos;
        
        if (pos.x - grid_x).abs() < threshold {
            snapped_pos.x = grid_x;
        }
        if (pos.y - grid_y).abs() < threshold {
            snapped_pos.y = grid_y;
        }
        
        // Node alignment snapping
        if self.snap_config.snap_to_nodes {
            for (_, node) in &self.nodes {
                let node_pos = Pos2::new(node.position().x, node.position().y);
                
                // Snap to node X
                if (pos.x - node_pos.x).abs() < threshold {
                    snapped_pos.x = node_pos.x;
                }
                
                // Snap to node Y
                if (pos.y - node_pos.y).abs() < threshold {
                    snapped_pos.y = node_pos.y;
                }
            }
        }
        
        snapped_pos
    }
}

/// Canvas widget for rendering
pub struct CanvasWidget<'a> {
    state: &'a mut CanvasState,
}

impl<'a> CanvasWidget<'a> {
    pub fn new(state: &'a mut CanvasState) -> Self {
        CanvasWidget { state }
    }
}

impl<'a> Widget for CanvasWidget<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            Sense::click_and_drag(),
        );
        
        // Handle input
        self.handle_input(ui, &response, rect);
        
        // Update camera animation
        self.state.camera.update();
        
        // Draw canvas
        let painter = ui.painter_at(rect);
        self.draw_canvas(&painter, rect);
        
        response
    }
}

impl<'a> CanvasWidget<'a> {
    /// Handle user input
    fn handle_input(&mut self, ui: &egui::Ui, response: &Response, rect: Rect) {
        // Mouse wheel zoom
        if let Some(hover_pos) = response.hover_pos() {
            let zoom_delta = ui.input(|i| i.scroll_delta.y) * 0.001;
            if zoom_delta != 0.0 {
                let world_pos = self.state.camera.screen_to_world(hover_pos, rect);
                self.state.camera.zoom_around(zoom_delta, world_pos);
            }
        }
        
        // Handle drag operations
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let world_pos = self.state.camera.screen_to_world(pos, rect);
                
                // Check if clicking on a node
                let clicked_node = self.find_node_at_pos(world_pos);
                
                if let Some(node_id) = clicked_node {
                    // Start dragging node
                    let node_pos = Pos2::new(
                        self.state.nodes[&node_id].position().x,
                        self.state.nodes[&node_id].position().y,
                    );
                    self.state.drag_state = DragState::DraggingNode {
                        node_id,
                        offset: world_pos - node_pos,
                    };
                    
                    // Select node
                    if !ui.input(|i| i.modifiers.shift) {
                        self.state.selected_nodes.clear();
                    }
                    self.state.selected_nodes.push(node_id);
                } else if ui.input(|i| i.modifiers.shift) {
                    // Start selection box
                    self.state.drag_state = DragState::SelectionBox {
                        start: world_pos,
                        current: world_pos,
                    };
                } else {
                    // Start panning canvas
                    self.state.drag_state = DragState::PanningCanvas {
                        start_pos: pos,
                    };
                }
            }
        }
        
        // Handle dragging
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let world_pos = self.state.camera.screen_to_world(pos, rect);
                
                match &self.state.drag_state {
                    DragState::DraggingNode { node_id, offset } => {
                        // Move node with snapping
                        let new_pos = self.state.apply_snap(world_pos - *offset);
                        if let Some(node) = self.state.nodes.get_mut(node_id) {
                            let old_pos = node.position(); // Store old position
                            node.set_position(Point2 { x: new_pos.x, y: new_pos.y });
                            
                            // Send event
                            let _ = self.state.event_sender.send(CanvasEvent::NodeMoved {
                                node_id: *node_id,
                                old_pos,
                                new_pos: node.position(),
                            });
                        }
                    }
                    DragState::PanningCanvas { start_pos } => {
                        let delta = pos - *start_pos;
                        self.state.camera.pan(delta);
                        self.state.drag_state = DragState::PanningCanvas { start_pos: pos };
                    }
                    DragState::SelectionBox { start, .. } => {
                        self.state.drag_state = DragState::SelectionBox {
                            start: *start,
                            current: world_pos,
                        };
                    }
                    _ => {}
                }
            }
        }
        
        // Handle drag end
        if response.drag_released() {
            match &self.state.drag_state {
                DragState::SelectionBox { start, current } => {
                    // Select nodes in box
                    let selection_rect = Rect::from_two_pos(*start, *current);
                    self.state.selected_nodes.clear();
                    
                    for (id, node) in &self.state.nodes {
                        let node_pos = Pos2::new(node.position().x, node.position().y);
                        if selection_rect.contains(node_pos) {
                            self.state.selected_nodes.push(*id);
                        }
                    }
                }
                _ => {}
            }
            
            self.state.drag_state = DragState::None;
        }
    }
    
    /// Find node at world position
    fn find_node_at_pos(&self, world_pos: Pos2) -> Option<NodeId> {
        for (id, node) in &self.state.nodes {
            let node_rect = Rect::from_center_size(
                Pos2::new(node.position().x, node.position().y),
                Vec2::new(node.size().width, node.size().height),
            );
            
            if node_rect.contains(world_pos) {
                return Some(*id);
            }
        }
        None
    }
    
    /// Draw the canvas
    fn draw_canvas(&self, painter: &Painter, rect: Rect) {
        // Background
        painter.rect_filled(rect, 0.0, Color32::from_gray(20));
        
        // Draw grid if zoomed in enough
        if self.state.camera.zoom > 0.5 {
            self.draw_grid(painter, rect);
        }
        
        // Draw connections (threads)
        for thread in &self.state.connections {
            self.draw_thread(painter, rect, thread);
        }
        
        // Draw nodes
        for (id, node) in &self.state.nodes {
            let selected = self.state.selected_nodes.contains(id);
            self.draw_node(painter, rect, node, selected);
        }
        
        // Draw selection box
        if let DragState::SelectionBox { start, current } = &self.state.drag_state {
            let screen_start = self.state.camera.world_to_screen(*start, rect);
            let screen_current = self.state.camera.world_to_screen(*current, rect);
            let selection_rect = Rect::from_two_pos(screen_start, screen_current);
            
            painter.rect(
                selection_rect,
                0.0,
                Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
            );
        }
        
        // Draw breadcrumb trails
        self.draw_breadcrumbs(painter, rect);
    }
    
    /// Draw background grid
    fn draw_grid(&self, painter: &Painter, rect: Rect) {
        let grid_size = self.state.snap_config.grid_size * self.state.camera.zoom;
        
        if grid_size < 5.0 {
            return; // Too small to see
        }
        
        let color = Color32::from_gray(30);
        let camera = &self.state.camera;
        
        // Calculate grid bounds
        let top_left = camera.screen_to_world(rect.min, rect);
        let bottom_right = camera.screen_to_world(rect.max, rect);
        
        let start_x = (top_left.x / self.state.snap_config.grid_size).floor() * self.state.snap_config.grid_size;
        let start_y = (top_left.y / self.state.snap_config.grid_size).floor() * self.state.snap_config.grid_size;
        
        // Draw vertical lines
        let mut x = start_x;
        while x <= bottom_right.x {
            let screen_x = camera.world_to_screen(Pos2::new(x, 0.0), rect).x;
            painter.line_segment(
                [Pos2::new(screen_x, rect.min.y), Pos2::new(screen_x, rect.max.y)],
                Stroke::new(1.0, color),
            );
            x += self.state.snap_config.grid_size;
        }
        
        // Draw horizontal lines
        let mut y = start_y;
        while y <= bottom_right.y {
            let screen_y = camera.world_to_screen(Pos2::new(0.0, y), rect).y;
            painter.line_segment(
                [Pos2::new(rect.min.x, screen_y), Pos2::new(rect.max.x, screen_y)],
                Stroke::new(1.0, color),
            );
            y += self.state.snap_config.grid_size;
        }
    }
    
    /// Draw a thread connection with bezier curves
    fn draw_thread(&self, painter: &Painter, rect: Rect, thread: &ThreadConnection) {
        let from_node = match self.state.nodes.get(&thread.connection.from.0) {
            Some(node) => node,
            None => return,
        };
        
        let to_node = match self.state.nodes.get(&thread.connection.to.0) {
            Some(node) => node,
            None => return,
        };
        
        // Calculate connection points
        let from_pos = Pos2::new(
            from_node.position().x + from_node.size().width / 2.0,
            from_node.position().y,
        );
        let to_pos = Pos2::new(
            to_node.position().x - to_node.size().width / 2.0,
            to_node.position().y,
        );
        
        let from_screen = self.state.camera.world_to_screen(from_pos, rect);
        let to_screen = self.state.camera.world_to_screen(to_pos, rect);
        
        // Calculate control points for bezier curve
        let distance = (to_screen - from_screen).length();
        let control_offset = distance.min(100.0) * 0.5;
        
        let control1 = from_screen + Vec2::new(control_offset, 0.0);
        let control2 = to_screen - Vec2::new(control_offset, 0.0);
        
        // Determine visibility based on zoom
        let base_alpha = if self.state.camera.zoom < 0.5 {
            // Fade out when zoomed out
            (self.state.camera.zoom * 2.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        
        // Apply hover state
        let alpha = base_alpha * (0.5 + thread.hover_state * 0.5);
        let color = Color32::from_rgba_unmultiplied(
            thread.color.r(),
            thread.color.g(),
            thread.color.b(),
            (thread.color.a() as f32 * alpha) as u8,
        );
        
        // Draw bezier curve
        let bezier = CubicBezierShape::from_points_stroke(
            [from_screen, control1, control2, to_screen],
            false,
            Color32::TRANSPARENT,
            Stroke::new(2.0 + thread.hover_state * 2.0, color),
        );
        
        painter.add(bezier);
        
        // Draw arrow head
        let arrow_size = 8.0 * self.state.camera.zoom;
        let arrow_dir = (to_screen - control2).normalized();
        let arrow_normal = Vec2::new(-arrow_dir.y, arrow_dir.x);
        
        let arrow_points = vec![
            to_screen,
            to_screen - arrow_dir * arrow_size + arrow_normal * arrow_size * 0.5,
            to_screen - arrow_dir * arrow_size - arrow_normal * arrow_size * 0.5,
        ];
        
        painter.add(Shape::convex_polygon(arrow_points, color, Stroke::NONE));
    }
    
    /// Draw a node
    fn draw_node(&self, painter: &Painter, rect: Rect, node: &Box<dyn Node>, selected: bool) {
        let pos = self.state.camera.world_to_screen(
            Pos2::new(node.position().x, node.position().y),
            rect,
        );
        let size = Vec2::new(node.size().width, node.size().height) * self.state.camera.zoom;
        
        let node_rect = Rect::from_center_size(pos, size);
        
        // Node background color based on type
        let bg_color = match node.type_name() {
            "TableNode" => Color32::from_rgb(60, 120, 180),
            "QueryNode" => Color32::from_rgb(180, 120, 60),
            "PlotNode" => Color32::from_rgb(120, 180, 60),
            "TransformNode" => Color32::from_rgb(180, 60, 120),
            "ExportNode" => Color32::from_rgb(60, 180, 120),
            _ => Color32::from_rgb(100, 100, 100),
        };
        
        // Draw node background
        painter.rect_filled(node_rect, 5.0, bg_color);
        
        // Draw selection outline
        if selected {
            painter.rect_stroke(
                node_rect.expand(2.0),
                5.0,
                Stroke::new(2.0, Color32::from_rgb(255, 200, 100)),
            );
        }
        
        // Draw node label
        let label = self.state.get_node_label(node);
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::default(),
            Color32::WHITE,
        );
        
        // Draw ports
        // TODO: Add visual port indicators
    }
    
    /// Draw breadcrumb trails
    fn draw_breadcrumbs(&self, painter: &Painter, rect: Rect) {
        let mut y_offset = 10.0;
        
        for (node_id, trail) in &self.state.breadcrumbs {
            if trail.trail.is_empty() {
                continue;
            }
            
            // Only show breadcrumbs for selected nodes or all if none selected
            if !self.state.selected_nodes.is_empty() && !self.state.selected_nodes.contains(node_id) {
                continue;
            }
            
            let mut x_offset = 10.0;
            
            // Draw breadcrumb background
            let breadcrumb_height = 30.0;
            let total_width = trail.trail.len() as f32 * 120.0;
            
            painter.rect_filled(
                Rect::from_min_size(
                    rect.min + Vec2::new(x_offset - 5.0, y_offset - 5.0),
                    Vec2::new(total_width, breadcrumb_height),
                ),
                5.0,
                Color32::from_rgba_unmultiplied(40, 40, 40, 200),
            );
            
            // Draw each breadcrumb item
            for (i, item) in trail.trail.iter().enumerate() {
                let item_rect = Rect::from_min_size(
                    rect.min + Vec2::new(x_offset, y_offset),
                    Vec2::new(100.0, 20.0),
                );
                
                // Draw item background
                let item_color = match item.icon {
                    BreadcrumbIcon::Dataset => Color32::from_rgb(60, 120, 180),
                    BreadcrumbIcon::Query => Color32::from_rgb(180, 120, 60),
                    BreadcrumbIcon::Plot => Color32::from_rgb(120, 180, 60),
                    BreadcrumbIcon::Transform => Color32::from_rgb(180, 60, 120),
                    BreadcrumbIcon::Export => Color32::from_rgb(60, 180, 120),
                };
                
                painter.rect_filled(item_rect, 3.0, item_color);
                
                // Draw text
                painter.text(
                    item_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &item.label,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
                
                // Draw separator arrow
                if i < trail.trail.len() - 1 {
                    painter.text(
                        item_rect.right_center() + Vec2::new(10.0, 0.0),
                        egui::Align2::CENTER_CENTER,
                        "→",
                        egui::FontId::proportional(14.0),
                        Color32::from_gray(180),
                    );
                }
                
                x_offset += 120.0;
            }
            
            y_offset += 40.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_transforms() {
        let mut camera = Camera2D::default();
        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        
        // Test world to screen
        let world_pos = Pos2::new(100.0, 100.0);
        let screen_pos = camera.world_to_screen(world_pos, screen_rect);
        assert_eq!(screen_pos, Pos2::new(500.0, 400.0)); // 400 + 100, 300 + 100
        
        // Test screen to world
        let world_pos_back = camera.screen_to_world(screen_pos, screen_rect);
        assert_eq!(world_pos_back, world_pos);
        
        // Test with zoom
        camera.zoom = 2.0;
        let screen_pos_zoomed = camera.world_to_screen(world_pos, screen_rect);
        assert_eq!(screen_pos_zoomed, Pos2::new(600.0, 500.0)); // 400 + 100*2, 300 + 100*2
    }
    
    #[test]
    fn test_snap_to_grid() {
        let (tx, _) = broadcast::channel(100);
        let state = CanvasState::new(tx);
        
        // Test grid snapping
        let pos = Pos2::new(23.0, 47.0);
        let snapped = state.apply_snap(pos);
        assert_eq!(snapped, Pos2::new(20.0, 40.0)); // Snapped to 20x20 grid
        
        // Test position already on grid
        let pos = Pos2::new(40.0, 60.0);
        let snapped = state.apply_snap(pos);
        assert_eq!(snapped, pos);
    }
    
    #[test]
    fn test_breadcrumb_trail() {
        let mut trail = BreadcrumbTrail {
            trail: VecDeque::new(),
            max_items: 3,
        };
        
        // Add items
        for i in 0..5 {
            trail.trail.push_back(BreadcrumbItem {
                node_id: NodeId::new(),
                label: format!("Item {}", i),
                icon: BreadcrumbIcon::Query,
            });
            
            // Maintain max size
            while trail.trail.len() > trail.max_items {
                trail.trail.pop_front();
            }
        }
        
        assert_eq!(trail.trail.len(), 3);
        assert_eq!(trail.trail[0].label, "Item 2");
        assert_eq!(trail.trail[2].label, "Item 4");
    }
} 