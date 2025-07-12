//! Data table widget with virtual scrolling.
//! TODO: Implement based on UI/UX pattern research from agents.

use egui::{Response, Ui};

pub struct DataTable {
    // TODO: Implement
}

impl DataTable {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.label("Data table coming soon...")
    }
} 