//! Plot configuration widget.
//! TODO: Implement configuration UI for different plot types.

use egui::{Response, Ui};
use pika_core::plots::PlotType;

pub struct PlotConfigPanel {
    plot_type: PlotType,
}

impl PlotConfigPanel {
    pub fn new(plot_type: PlotType) -> Self {
        Self { plot_type }
    }
    
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.label(format!("Configure {:?} plot", self.plot_type))
    }
} 