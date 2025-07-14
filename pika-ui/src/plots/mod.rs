//! Plot rendering modules.

// These modules use egui_plot which is a lightweight plotting library
pub mod bar_plot;
pub mod box_plot;
pub mod correlation_plot;
pub mod enhanced_scatter_plot;
pub mod heatmap_plot;
pub mod histogram_plot;
pub mod line_plot;
pub mod plot_renderer;
pub mod radar_plot;
pub mod scatter_plot;
pub mod time_series_plot;
pub mod violin_plot;

// Re-export commonly used items
pub use plot_renderer::{PlotRenderer, PlotRenderContext};

// For now, export a simple placeholder plot trait
pub trait PlotView {
    fn render(&self, ui: &mut egui::Ui);
} 