//! Canvas system for visual node-based interface with threads and breadcrumbs.

use pika_core::{
    events::{EventBus, CanvasEvent, NodeEvent},
    types::{NodeId, Point2, Connection},
    Node,
};
use egui::{Ui, Color32, Pos2, Vec2, Rect, Response, Sense};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Simple canvas for node editing
pub struct Canvas {
    pub zoom: f32,
    pub pan: Vec2,
    pub event_sender: broadcast::Sender<CanvasEvent>,
}

impl Canvas {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            event_sender,
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Draw background
        ui.painter().rect_filled(
            available_rect,
            0.0,
            Color32::from_rgb(40, 40, 45),
        );
        
        // Handle input
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        
        if response.hovered() {
            let zoom_delta = ui.input(|i| i.raw_scroll_delta.y) * 0.001;
            self.zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);
        }
        
        response
    }
} 