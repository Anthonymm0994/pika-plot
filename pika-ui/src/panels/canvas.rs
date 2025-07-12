//! Canvas panel for node-based data visualization.

use crate::state::{AppState, NodeConnection, ConnectionType};
use pika_core::events::AppEvent;
use pika_core::types::NodeId;
use tokio::sync::broadcast::Sender;
use egui::{Context, Ui, Painter, Pos2, Vec2, Color32, Stroke, Rect, Response, Sense};

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
}

impl CanvasPanel {
    pub fn new(_ctx: &Context) -> Self {
        Self {
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            connecting_from: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState, event_tx: &Sender<AppEvent>) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            Sense::click_and_drag(),
        );
        
        // Handle canvas pan with middle mouse
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan_offset += response.drag_delta();
        }
        
        // Handle zoom with scroll
        if response.hovered() {
            ui.input(|i| {
                if i.scroll_delta.y != 0.0 {
                    let zoom_delta = 1.0 + i.scroll_delta.y * 0.01;
                    self.zoom *= zoom_delta;
                    self.zoom = self.zoom.clamp(0.1, 5.0);
                }
            });
        }
        
        // Transform helper
        let to_screen = |pos: Pos2| -> Pos2 {
            pos2(
                pos.x * self.zoom + self.pan_offset.x + response.rect.left(),
                pos.y * self.zoom + self.pan_offset.y + response.rect.top(),
            )
        };
        
        let from_screen = |pos: Pos2| -> Pos2 {
            pos2(
                (pos.x - response.rect.left() - self.pan_offset.x) / self.zoom,
                (pos.y - response.rect.top() - self.pan_offset.y) / self.zoom,
            )
        };
        
        // Draw grid background
        self.draw_grid(&painter, &response.rect);
        
        // Draw connections
        for connection in &state.connections {
            if let (Some(from_node), Some(to_node)) = (
                state.data_nodes.get(&connection.from),
                state.data_nodes.get(&connection.to),
            ) {
                let from_pos = to_screen(from_node.position + Vec2::new(from_node.size.x, from_node.size.y / 2.0));
                let to_pos = to_screen(to_node.position + Vec2::new(0.0, to_node.size.y / 2.0));
                
                // Draw bezier curve
                let control1 = from_pos + Vec2::new(50.0, 0.0);
                let control2 = to_pos - Vec2::new(50.0, 0.0);
                
                let points = self.bezier_points(from_pos, control1, control2, to_pos, 20);
                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(2.0, Color32::from_gray(100)),
                ));
            }
        }
        
        // Draw connection being created
        if let Some(from_id) = self.connecting_from {
            if let Some(from_node) = state.data_nodes.get(&from_id) {
                let from_pos = to_screen(from_node.position + Vec2::new(from_node.size.x, from_node.size.y / 2.0));
                let to_pos = response.interact_pointer_pos().unwrap_or(from_pos);
                
                painter.line_segment(
                    [from_pos, to_pos],
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 200)),
                );
            }
        }
        
        // Draw nodes
        for (id, node) in state.data_nodes.iter_mut() {
            let screen_pos = to_screen(node.position);
            let screen_size = node.size * self.zoom;
            
            let node_rect = Rect::from_min_size(screen_pos, screen_size);
            let node_response = ui.allocate_rect(node_rect, Sense::click_and_drag());
            
            // Draw node background
            let selected = state.selected_node == Some(*id);
            let fill_color = if selected {
                Color32::from_gray(60)
            } else {
                Color32::from_gray(40)
            };
            
            painter.rect_filled(
                node_rect,
                5.0,
                fill_color,
            );
            
            // Draw node border
            let border_color = if selected {
                Color32::from_rgb(100, 150, 200)
            } else {
                Color32::from_gray(80)
            };
            
            painter.rect_stroke(
                node_rect,
                5.0,
                Stroke::new(2.0, border_color),
            );
            
            // Draw node content
            painter.text(
                node_rect.center() - Vec2::new(0.0, 20.0),
                egui::Align2::CENTER_CENTER,
                &node.name,
                egui::FontId::proportional(16.0 * self.zoom),
                Color32::WHITE,
            );
            
            painter.text(
                node_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} rows", node.table_info.row_count),
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::from_gray(180),
            );
            
            painter.text(
                node_rect.center() + Vec2::new(0.0, 20.0),
                egui::Align2::CENTER_CENTER,
                format!("{} cols", node.table_info.columns.len()),
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::from_gray(180),
            );
            
            // Handle node interactions
            if node_response.clicked() {
                state.selected_node = Some(*id);
            }
            
            if node_response.drag_started() {
                self.dragging_node = Some(*id);
                self.drag_offset = screen_pos - response.interact_pointer_pos().unwrap_or(screen_pos);
            }
            
            if self.dragging_node == Some(*id) && response.dragged() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    node.position = from_screen(pointer_pos + self.drag_offset);
                }
            }
            
            if node_response.drag_released() {
                self.dragging_node = None;
            }
            
            // Context menu
            node_response.context_menu(|ui| {
                if ui.button("Execute Query").clicked() {
                    event_tx.send(AppEvent::ExecuteQuery {
                        id: *id,
                        sql: format!("SELECT * FROM {} LIMIT 100", node.table_info.table_name),
                        cache_key: None,
                    }).ok();
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Connect to...").clicked() {
                    self.connecting_from = Some(*id);
                    ui.close_menu();
                }
            });
        }
        
        // Handle connection completion
        if self.connecting_from.is_some() && response.clicked() {
            // Check if we clicked on a node
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let canvas_pos = from_screen(pointer_pos);
                
                for (id, node) in &state.data_nodes {
                    let node_rect = Rect::from_min_size(node.position, node.size);
                    if node_rect.contains(canvas_pos) && Some(*id) != self.connecting_from {
                        // Create connection
                        if let Some(from_id) = self.connecting_from {
                            state.connections.push(NodeConnection {
                                from: from_id,
                                to: *id,
                                connection_type: ConnectionType::DataFlow,
                            });
                        }
                        break;
                    }
                }
            }
            
            self.connecting_from = None;
        }
        
        // Draw zoom indicator
        painter.text(
            response.rect.right_bottom() - Vec2::new(10.0, 10.0),
            egui::Align2::RIGHT_BOTTOM,
            format!("{}%", (self.zoom * 100.0) as i32),
            egui::FontId::proportional(12.0),
            Color32::from_gray(150),
        );
    }
    
    fn draw_grid(&self, painter: &Painter, rect: &Rect) {
        let grid_size = 20.0 * self.zoom;
        
        if grid_size > 5.0 {  // Don't draw grid when zoomed out too far
            let offset_x = self.pan_offset.x % grid_size;
            let offset_y = self.pan_offset.y % grid_size;
            
            // Vertical lines
            let mut x = rect.left() + offset_x;
            while x < rect.right() {
                painter.line_segment(
                    [pos2(x, rect.top()), pos2(x, rect.bottom())],
                    Stroke::new(1.0, Color32::from_gray(30)),
                );
                x += grid_size;
            }
            
            // Horizontal lines
            let mut y = rect.top() + offset_y;
            while y < rect.bottom() {
                painter.line_segment(
                    [pos2(rect.left(), y), pos2(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_gray(30)),
                );
                y += grid_size;
            }
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
            
            points.push(pos2(x, y));
        }
        
        points
    }
}

use egui::pos2; 