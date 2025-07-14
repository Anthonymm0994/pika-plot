//! Plot rendering modules.

// TEMPORARILY DISABLED - These modules use polars/arrow dependencies
// We'll create simpler versions without heavy dependencies
/*
pub mod bar_plot;
pub mod box_plot;
pub mod correlation_plot;
pub mod heatmap;
pub mod histogram;
pub mod line_plot;
pub mod pie_chart;
pub mod scatter_plot;
pub mod time_series;
pub mod treemap;
pub mod violin_plot;
pub mod utils;
*/

// For now, export a simple placeholder plot trait
pub trait PlotRenderer {
    fn render(&self, ui: &mut egui::Ui);
} 