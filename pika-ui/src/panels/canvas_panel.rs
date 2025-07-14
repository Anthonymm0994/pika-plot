//! Canvas panel for node-based visual interface.

use pika_core::{
    events::EventBus,
};
use crate::canvas::Canvas;
use crate::state::AppState;
use egui::Ui;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;

// Define AppEvent locally since it's not available in the crate
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Placeholder event types
    CanvasUpdated,
}

pub struct CanvasPanel {
    canvas: Canvas,
}

impl CanvasPanel {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            canvas: Canvas::new(event_bus),
        }
    }
    
    pub fn show(&mut self, ui: &mut Ui, _state: &mut AppState, _event_tx: &Sender<AppEvent>) {
        // Show canvas toolbar
        self.canvas.show_toolbar(ui);
        
        ui.separator();
        
        // Show main canvas
        self.canvas.show(ui);
    }
} 