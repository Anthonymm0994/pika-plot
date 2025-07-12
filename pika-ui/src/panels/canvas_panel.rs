//! Canvas panel for node-based visual interface.

use crate::canvas::Canvas;
use pika_core::{
    events::{EventBus, CanvasEvent, NodeEvent},
    Node, NodeId,
};
use egui::{Context, Ui, Color32, Pos2, Vec2, Rect};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct CanvasPanel {
    canvas: Canvas,
    event_bus: Arc<EventBus>,
}

impl CanvasPanel {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            canvas: Canvas::new(),
            event_bus,
        }
    }
    
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.heading("Canvas");
        
        let available_rect = ui.available_rect_before_wrap();
        
        // Simple canvas rendering for now
        ui.allocate_rect(available_rect, egui::Sense::click_and_drag());
        
        // Draw background
        ui.painter().rect_filled(
            available_rect,
            0.0,
            Color32::from_rgb(40, 40, 45),
        );
        
        // Draw grid
        self.draw_grid(ui, available_rect);
        
        // Draw nodes (placeholder)
        self.draw_nodes(ui, available_rect);
    }
    
    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let grid_size = 20.0;
        let color = Color32::from_rgb(60, 60, 65);
        
        // Vertical lines
        let mut x = rect.left();
        while x <= rect.right() {
            ui.painter().line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                (1.0, color),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = rect.top();
        while y <= rect.bottom() {
            ui.painter().line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                (1.0, color),
            );
            y += grid_size;
        }
    }
    
    fn draw_nodes(&self, ui: &mut Ui, rect: Rect) {
        // Placeholder node rendering
        let node_rect = Rect::from_center_size(
            rect.center(),
            Vec2::new(120.0, 80.0),
        );
        
        ui.painter().rect_filled(
            node_rect,
            4.0,
            Color32::from_rgb(70, 70, 80),
        );
        
        ui.painter().rect_stroke(
            node_rect,
            4.0,
            (1.0, Color32::WHITE),
        );
        
        ui.painter().text(
            node_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Sample Node",
            egui::FontId::default(),
            Color32::WHITE,
        );
    }
} 