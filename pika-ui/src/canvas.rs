use egui::{Ui, Pos2, Vec2, Color32, Stroke, Rect};
use pika_core::events::EventBus;
use std::sync::Arc;

/// Excalidraw-style canvas implementation
pub struct Canvas {
    event_bus: Arc<EventBus>,
    elements: Vec<CanvasElement>,
    selected_elements: Vec<usize>,
    current_tool: Tool,
    zoom: f32,
    pan: Vec2,
    // Drawing state
    is_creating: bool,
    creation_start: Option<Pos2>,
    is_resizing: bool,
    resize_handle: Option<ResizeHandle>,
    is_dragging: bool,
    drag_start: Option<Pos2>,
    // Freehand drawing
    current_drawing_points: Vec<Pos2>,
}

#[derive(Debug, Clone)]
pub struct CanvasElement {
    pub id: usize,
    pub element_type: ElementType,
    pub position: Pos2,
    pub size: Vec2,
    pub color: Color32,
    pub stroke_width: f32,
    pub points: Vec<Pos2>,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Rectangle,
    Circle,
    Line,
    FreehandDraw,
    Text(String),
    PlotNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Select,
    Rectangle,
    Circle,
    Line,
    FreehandDraw,
    Text,
    PlotNode,
}

#[derive(Debug, Clone, Copy)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

impl Canvas {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            elements: Vec::new(),
            selected_elements: Vec::new(),
            current_tool: Tool::Select,
            zoom: 1.0,
            pan: Vec2::ZERO,
            is_creating: false,
            creation_start: None,
            is_resizing: false,
            resize_handle: None,
            is_dragging: false,
            drag_start: None,
            current_drawing_points: Vec::new(),
        }
    }
    
    pub fn show_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🎨 Pika-Plot Canvas");
            ui.separator();
            
            if ui.selectable_label(self.current_tool == Tool::Select, "👆 Select").clicked() {
                self.current_tool = Tool::Select;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::Rectangle, "⬜ Rectangle").clicked() {
                self.current_tool = Tool::Rectangle;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::Circle, "⭕ Circle").clicked() {
                self.current_tool = Tool::Circle;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::Line, "📏 Line").clicked() {
                self.current_tool = Tool::Line;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::FreehandDraw, "✏️ Draw").clicked() {
                self.current_tool = Tool::FreehandDraw;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::Text, "🔤 Text").clicked() {
                self.current_tool = Tool::Text;
                self.reset_interaction_state();
            }
            if ui.selectable_label(self.current_tool == Tool::PlotNode, "📊 Plot").clicked() {
                self.current_tool = Tool::PlotNode;
                self.reset_interaction_state();
            }
            
            ui.separator();
            ui.label(format!("Zoom: {:.1}x", self.zoom));
            ui.label(format!("Elements: {}", self.elements.len()));
            
            if !self.elements.is_empty() {
                ui.separator();
                if ui.button("🗑️ Clear").clicked() {
                    self.elements.clear();
                    self.selected_elements.clear();
                    self.reset_interaction_state();
                }
            }
        });
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        let pointer_pos = response.interact_pointer_pos().unwrap_or(rect.center());
        
        // Handle input based on current state
        if response.clicked() {
            self.handle_click(pointer_pos);
        }
        
        if response.drag_started() {
            self.handle_drag_start(pointer_pos);
        }
        
        if response.dragged() {
            self.handle_drag(pointer_pos, response.drag_delta());
        }
        
        if response.drag_stopped() {
            self.handle_drag_end();
        }
        
        // Handle hover for resize handles
        if self.current_tool == Tool::Select && !self.selected_elements.is_empty() {
            if let Some(handle) = self.get_resize_handle_at(pointer_pos) {
                ui.ctx().set_cursor_icon(match handle {
                    ResizeHandle::TopLeft | ResizeHandle::BottomRight => egui::CursorIcon::ResizeNwSe,
                    ResizeHandle::TopRight | ResizeHandle::BottomLeft => egui::CursorIcon::ResizeNeSw,
                    ResizeHandle::Top | ResizeHandle::Bottom => egui::CursorIcon::ResizeVertical,
                    ResizeHandle::Left | ResizeHandle::Right => egui::CursorIcon::ResizeHorizontal,
                });
            }
        }
        
        // Draw background
        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(50, 50, 50));
        
        // Draw grid
        self.draw_grid(ui, rect);
        
        // Draw elements
        for (i, element) in self.elements.iter().enumerate() {
            let is_selected = self.selected_elements.contains(&i);
            self.draw_element(ui, element, is_selected);
        }
        
        // Draw current creation preview
        if self.is_creating && self.creation_start.is_some() {
            self.draw_creation_preview(ui, pointer_pos);
        }
        
        // Draw current freehand drawing
        if self.current_tool == Tool::FreehandDraw && !self.current_drawing_points.is_empty() {
            self.draw_current_freehand(ui);
        }
        
        // Draw selection indicators and resize handles
        self.draw_selection_indicators(ui);
        
        // Show help text
        if self.elements.is_empty() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "🎨 Welcome to Pika-Plot!\n\n1. Select a tool from the toolbar above\n2. Click and drag to create shapes (like Excalidraw)\n3. Use Select tool to move and resize elements\n4. Create plot nodes to visualize data\n\nTip: Drag to create, select to modify!",
                egui::FontId::proportional(16.0),
                Color32::LIGHT_GRAY,
            );
        }
    }
    
    fn reset_interaction_state(&mut self) {
        self.is_creating = false;
        self.creation_start = None;
        self.is_resizing = false;
        self.resize_handle = None;
        self.is_dragging = false;
        self.drag_start = None;
        self.current_drawing_points.clear();
    }
    
    fn handle_click(&mut self, pos: Pos2) {
        match self.current_tool {
            Tool::Select => {
                // Check if clicking on a resize handle
                if let Some(handle) = self.get_resize_handle_at(pos) {
                    self.is_resizing = true;
                    self.resize_handle = Some(handle);
                    return;
                }
                
                // Find clicked element
                self.selected_elements.clear();
                for (i, element) in self.elements.iter().enumerate().rev() {
                    if self.point_in_element(pos, element) {
                        self.selected_elements.push(i);
                        break;
                    }
                }
            }
            Tool::Text => {
                // For text, create immediately on click
                let element = CanvasElement {
                    id: self.elements.len(),
                    element_type: ElementType::Text("Click to edit".to_string()),
                    position: pos,
                    size: Vec2::new(120.0, 20.0),
                    color: Color32::WHITE,
                    stroke_width: 1.0,
                    points: vec![],
                    rotation: 0.0,
                };
                self.elements.push(element);
                println!("🔤 Created text at {:?}", pos);
            }
            _ => {
                // For other tools, start creation mode
                self.is_creating = true;
                self.creation_start = Some(pos);
            }
        }
    }
    
    fn handle_drag_start(&mut self, pos: Pos2) {
        match self.current_tool {
            Tool::Select => {
                if self.is_resizing {
                    // Already handled in handle_click
                    return;
                }
                
                // Start dragging selected elements
                if !self.selected_elements.is_empty() {
                    self.is_dragging = true;
                    self.drag_start = Some(pos);
                }
            }
            Tool::FreehandDraw => {
                // Start freehand drawing
                self.current_drawing_points = vec![pos];
                println!("✏️ Started freehand drawing at {:?}", pos);
            }
            _ => {
                // Creation mode is handled in handle_click
            }
        }
    }
    
    fn handle_drag(&mut self, current_pos: Pos2, delta: Vec2) {
        if self.is_resizing {
            self.handle_resize(current_pos, delta);
        } else if self.is_dragging && self.current_tool == Tool::Select {
            // Move selected elements
            for &i in &self.selected_elements {
                if let Some(element) = self.elements.get_mut(i) {
                    element.position += delta;
                }
            }
        } else if self.current_tool == Tool::FreehandDraw {
            // Add points to freehand drawing
            self.current_drawing_points.push(current_pos);
        }
        // For creation mode, the preview is handled in draw_creation_preview
    }
    
    fn handle_drag_end(&mut self) {
        if self.is_creating && self.creation_start.is_some() {
            let start = self.creation_start.unwrap();
            let end = self.elements.last().map_or(start, |_| start); // This will be set properly in draw_creation_preview
            
            // We need to get the current pointer position, but since we don't have it here,
            // we'll create the element in draw_creation_preview when dragging
        } else if self.current_tool == Tool::FreehandDraw && self.current_drawing_points.len() >= 2 {
            let element = CanvasElement {
                id: self.elements.len(),
                element_type: ElementType::FreehandDraw,
                position: self.current_drawing_points[0],
                size: Vec2::new(100.0, 100.0),
                color: Color32::from_rgb(255, 100, 255),
                stroke_width: 2.0,
                points: self.current_drawing_points.clone(),
                rotation: 0.0,
            };
            self.elements.push(element);
            println!("✏️ Created freehand drawing with {} points", self.current_drawing_points.len());
        }
        
        self.reset_interaction_state();
    }
    
    fn handle_resize(&mut self, _current_pos: Pos2, delta: Vec2) {
        if let Some(handle) = self.resize_handle {
            for &i in &self.selected_elements {
                if let Some(element) = self.elements.get_mut(i) {
                    match handle {
                        ResizeHandle::TopLeft => {
                            element.position += delta;
                            element.size -= delta;
                        }
                        ResizeHandle::TopRight => {
                            element.position.y += delta.y;
                            element.size.x += delta.x;
                            element.size.y -= delta.y;
                        }
                        ResizeHandle::BottomLeft => {
                            element.position.x += delta.x;
                            element.size.x -= delta.x;
                            element.size.y += delta.y;
                        }
                        ResizeHandle::BottomRight => {
                            element.size += delta;
                        }
                        ResizeHandle::Top => {
                            element.position.y += delta.y;
                            element.size.y -= delta.y;
                        }
                        ResizeHandle::Bottom => {
                            element.size.y += delta.y;
                        }
                        ResizeHandle::Left => {
                            element.position.x += delta.x;
                            element.size.x -= delta.x;
                        }
                        ResizeHandle::Right => {
                            element.size.x += delta.x;
                        }
                    }
                    
                    // Ensure minimum size
                    element.size.x = element.size.x.max(10.0);
                    element.size.y = element.size.y.max(10.0);
                }
            }
        }
    }
    
    fn draw_creation_preview(&mut self, ui: &mut Ui, current_pos: Pos2) {
        if let Some(start) = self.creation_start {
            let size = Vec2::new((current_pos.x - start.x).abs(), (current_pos.y - start.y).abs());
            let position = Pos2::new(start.x.min(current_pos.x), start.y.min(current_pos.y));
            
            // Only create preview if we have meaningful size
            if size.x > 5.0 || size.y > 5.0 {
                let stroke = Stroke::new(2.0, Color32::YELLOW);
                
                match self.current_tool {
                    Tool::Rectangle => {
                        let rect = Rect::from_min_size(position, size);
                        ui.painter().rect_stroke(rect, 5.0, stroke);
                        
                        // Create the actual element on significant drag
                        if size.x > 20.0 && size.y > 20.0 {
                            // Check if we need to create the element
                            let should_create = self.elements.is_empty() || 
                                self.elements.last().map_or(true, |e| e.element_type != ElementType::Rectangle || e.position != position);
                            
                            if should_create {
                                let element = CanvasElement {
                                    id: self.elements.len(),
                                    element_type: ElementType::Rectangle,
                                    position,
                                    size,
                                    color: Color32::from_rgb(100, 149, 237),
                                    stroke_width: 2.0,
                                    points: vec![],
                                    rotation: 0.0,
                                };
                                self.elements.push(element);
                                println!("📐 Created rectangle at {:?} with size {:?}", position, size);
                            } else if let Some(last_element) = self.elements.last_mut() {
                                // Update the last element
                                last_element.position = position;
                                last_element.size = size;
                            }
                        }
                    }
                    Tool::Circle => {
                        let center = position + size / 2.0;
                        let radius = (size.x + size.y) / 4.0;
                        ui.painter().circle_stroke(center, radius, stroke);
                        
                        if radius > 10.0 {
                            let should_create = self.elements.is_empty() || 
                                self.elements.last().map_or(true, |e| e.element_type != ElementType::Circle || e.position != position);
                            
                            if should_create {
                                let element = CanvasElement {
                                    id: self.elements.len(),
                                    element_type: ElementType::Circle,
                                    position,
                                    size,
                                    color: Color32::from_rgb(255, 99, 71),
                                    stroke_width: 2.0,
                                    points: vec![],
                                    rotation: 0.0,
                                };
                                self.elements.push(element);
                                println!("⭕ Created circle at {:?} with size {:?}", position, size);
                            } else if let Some(last_element) = self.elements.last_mut() {
                                last_element.position = position;
                                last_element.size = size;
                            }
                        }
                    }
                    Tool::Line => {
                        ui.painter().line_segment([start, current_pos], stroke);
                        
                        if (current_pos - start).length() > 10.0 {
                            let should_create = self.elements.is_empty() || 
                                self.elements.last().map_or(true, |e| e.element_type != ElementType::Line);
                            
                            if should_create {
                                let element = CanvasElement {
                                    id: self.elements.len(),
                                    element_type: ElementType::Line,
                                    position: start,
                                    size,
                                    color: Color32::from_rgb(255, 255, 100),
                                    stroke_width: 2.0,
                                    points: vec![start, current_pos],
                                    rotation: 0.0,
                                };
                                self.elements.push(element);
                                println!("📏 Created line from {:?} to {:?}", start, current_pos);
                            } else if let Some(last_element) = self.elements.last_mut() {
                                last_element.points = vec![start, current_pos];
                            }
                        }
                    }
                    Tool::PlotNode => {
                        let rect = Rect::from_min_size(position, size.max(Vec2::new(100.0, 80.0)));
                        ui.painter().rect_stroke(rect, 8.0, stroke);
                        
                        if size.x > 50.0 && size.y > 40.0 {
                            let should_create = self.elements.is_empty() || 
                                self.elements.last().map_or(true, |e| e.element_type != ElementType::PlotNode);
                            
                            if should_create {
                                let element = CanvasElement {
                                    id: self.elements.len(),
                                    element_type: ElementType::PlotNode,
                                    position,
                                    size: size.max(Vec2::new(150.0, 100.0)),
                                    color: Color32::from_rgb(50, 205, 50),
                                    stroke_width: 2.0,
                                    points: vec![],
                                    rotation: 0.0,
                                };
                                self.elements.push(element);
                                println!("📊 Created plot node at {:?} with size {:?}", position, size);
                            } else if let Some(last_element) = self.elements.last_mut() {
                                last_element.position = position;
                                last_element.size = size.max(Vec2::new(150.0, 100.0));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn draw_current_freehand(&self, ui: &mut Ui) {
        for points in self.current_drawing_points.windows(2) {
            ui.painter().line_segment(
                [points[0], points[1]],
                Stroke::new(2.0, Color32::YELLOW),
            );
        }
    }
    
    fn get_resize_handle_at(&self, pos: Pos2) -> Option<ResizeHandle> {
        if self.selected_elements.len() != 1 {
            return None;
        }
        
        let element = &self.elements[self.selected_elements[0]];
        let rect = Rect::from_min_size(element.position, element.size);
        let handle_size = 8.0;
        
        let handles = [
            (ResizeHandle::TopLeft, rect.min),
            (ResizeHandle::TopRight, Pos2::new(rect.max.x, rect.min.y)),
            (ResizeHandle::BottomLeft, Pos2::new(rect.min.x, rect.max.y)),
            (ResizeHandle::BottomRight, rect.max),
            (ResizeHandle::Top, Pos2::new(rect.center().x, rect.min.y)),
            (ResizeHandle::Bottom, Pos2::new(rect.center().x, rect.max.y)),
            (ResizeHandle::Left, Pos2::new(rect.min.x, rect.center().y)),
            (ResizeHandle::Right, Pos2::new(rect.max.x, rect.center().y)),
        ];
        
        for (handle, handle_pos) in handles {
            let handle_rect = Rect::from_center_size(handle_pos, Vec2::splat(handle_size));
            if handle_rect.contains(pos) {
                return Some(handle);
            }
        }
        
        None
    }
    
    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let grid_color = Color32::from_rgba_unmultiplied(100, 100, 100, 50);
        
        // Vertical lines
        let mut x = rect.min.x;
        while x < rect.max.x {
            ui.painter().line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = rect.min.y;
        while y < rect.max.y {
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, grid_color),
            );
            y += grid_size;
        }
    }
    
    fn draw_element(&self, ui: &mut Ui, element: &CanvasElement, is_selected: bool) {
        let stroke_width = if is_selected { element.stroke_width + 1.0 } else { element.stroke_width };
        let color = if is_selected { Color32::YELLOW } else { element.color };
        
        match &element.element_type {
            ElementType::Rectangle => {
                let rect = Rect::from_min_size(element.position, element.size);
                ui.painter().rect_stroke(rect, 5.0, Stroke::new(stroke_width, color));
                if is_selected {
                    ui.painter().rect_filled(rect, 5.0, Color32::from_rgba_unmultiplied(255, 255, 0, 20));
                }
            }
            ElementType::Circle => {
                let center = element.position + element.size / 2.0;
                let radius = (element.size.x + element.size.y) / 4.0;
                ui.painter().circle_stroke(center, radius, Stroke::new(stroke_width, color));
                if is_selected {
                    ui.painter().circle_filled(center, radius, Color32::from_rgba_unmultiplied(255, 255, 0, 20));
                }
            }
            ElementType::PlotNode => {
                let rect = Rect::from_min_size(element.position, element.size);
                ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(40, 40, 40));
                ui.painter().rect_stroke(rect, 8.0, Stroke::new(stroke_width, color));
                
                // Draw plot icon
                let icon_pos = element.position + Vec2::new(10.0, 10.0);
                ui.painter().text(
                    icon_pos,
                    egui::Align2::LEFT_TOP,
                    "📊 Plot Node",
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
                
                // Add connection points
                let center = rect.center();
                let connection_points = [
                    Pos2::new(rect.min.x, center.y), // Left
                    Pos2::new(rect.max.x, center.y), // Right
                    Pos2::new(center.x, rect.min.y), // Top
                    Pos2::new(center.x, rect.max.y), // Bottom
                ];
                
                for point in connection_points {
                    ui.painter().circle_filled(point, 4.0, Color32::WHITE);
                    ui.painter().circle_stroke(point, 4.0, Stroke::new(1.0, Color32::BLACK));
                }
            }
            ElementType::Text(text) => {
                ui.painter().text(
                    element.position,
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(14.0),
                    color,
                );
                
                if is_selected {
                    let text_rect = Rect::from_min_size(element.position, element.size);
                    ui.painter().rect_stroke(text_rect, 2.0, Stroke::new(1.0, Color32::YELLOW));
                }
            }
            ElementType::Line => {
                if element.points.len() >= 2 {
                    ui.painter().line_segment(
                        [element.points[0], element.points[1]],
                        Stroke::new(stroke_width, color),
                    );
                    
                    if is_selected {
                        // Draw endpoint indicators
                        for point in &element.points {
                            ui.painter().circle_filled(*point, 4.0, Color32::YELLOW);
                        }
                    }
                }
            }
            ElementType::FreehandDraw => {
                for points in element.points.windows(2) {
                    ui.painter().line_segment(
                        [points[0], points[1]],
                        Stroke::new(stroke_width, color),
                    );
                }
                
                if is_selected && !element.points.is_empty() {
                    // Draw start and end point indicators
                    ui.painter().circle_filled(element.points[0], 4.0, Color32::GREEN);
                    if let Some(last_point) = element.points.last() {
                        ui.painter().circle_filled(*last_point, 4.0, Color32::RED);
                    }
                }
            }
        }
    }
    
    fn draw_selection_indicators(&self, ui: &mut Ui) {
        for &i in &self.selected_elements {
            if let Some(element) = self.elements.get(i) {
                let rect = Rect::from_min_size(element.position, element.size);
                
                // Draw resize handles
                let handle_size = 8.0;
                let handles = [
                    rect.min,                                    // Top-left
                    Pos2::new(rect.max.x, rect.min.y),         // Top-right
                    rect.max,                                    // Bottom-right
                    Pos2::new(rect.min.x, rect.max.y),         // Bottom-left
                    Pos2::new(rect.center().x, rect.min.y),    // Top
                    Pos2::new(rect.center().x, rect.max.y),    // Bottom
                    Pos2::new(rect.min.x, rect.center().y),    // Left
                    Pos2::new(rect.max.x, rect.center().y),    // Right
                ];
                
                for handle in handles {
                    ui.painter().rect_filled(
                        Rect::from_center_size(handle, Vec2::splat(handle_size)),
                        2.0,
                        Color32::WHITE
                    );
                    ui.painter().rect_stroke(
                        Rect::from_center_size(handle, Vec2::splat(handle_size)),
                        2.0,
                        Stroke::new(1.0, Color32::BLACK)
                    );
                }
            }
        }
    }
    
    fn point_in_element(&self, point: Pos2, element: &CanvasElement) -> bool {
        match &element.element_type {
            ElementType::Line => {
                // Check if point is near the line
                if element.points.len() >= 2 {
                    let start = element.points[0];
                    let end = element.points[1];
                    let distance = self.point_to_line_distance(point, start, end);
                    distance < 5.0 // 5 pixel tolerance
                } else {
                    false
                }
            }
            ElementType::FreehandDraw => {
                // Check if point is near any segment of the freehand drawing
                for points in element.points.windows(2) {
                    let distance = self.point_to_line_distance(point, points[0], points[1]);
                    if distance < 5.0 {
                        return true;
                    }
                }
                false
            }
            ElementType::Circle => {
                let center = element.position + element.size / 2.0;
                let radius = (element.size.x + element.size.y) / 4.0;
                let distance = (point - center).length();
                (distance - radius).abs() < 10.0 // 10 pixel tolerance around the circle
            }
            _ => {
                let rect = Rect::from_min_size(element.position, element.size);
                rect.contains(point)
            }
        }
    }
    
    fn point_to_line_distance(&self, point: Pos2, line_start: Pos2, line_end: Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        
        let line_length_sq = line_vec.length_sq();
        if line_length_sq == 0.0 {
            return point_vec.length();
        }
        
        let t = (point_vec.dot(line_vec) / line_length_sq).clamp(0.0, 1.0);
        let projection = line_start + t * line_vec;
        (point - projection).length()
    }
} 