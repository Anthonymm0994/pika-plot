use egui::Ui;
use arrow::record_batch::RecordBatch;
use pika_core::plots::PlotConfig;

pub struct PlotRenderer;

impl PlotRenderer {
    pub fn new() -> Self {
        Self
    }
}

/// Render a plot based on its configuration
pub fn render_plot(ui: &mut Ui, config: &PlotConfig, data: &RecordBatch) {
    super::render_plot_by_config(ui, config, data);
} 