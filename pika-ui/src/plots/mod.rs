//! Plot rendering UI components

pub mod bar_plot;
pub mod box_plot;
pub mod heatmap_plot;
pub mod histogram_plot;
pub mod line_plot;
pub mod scatter_plot;
pub mod plot_renderer;
pub mod enhanced_scatter_plot;

// Re-exports
pub use bar_plot::*;
pub use box_plot::*;
pub use heatmap_plot::*;
pub use histogram_plot::*;
pub use line_plot::*;
pub use scatter_plot::*;
pub use plot_renderer::*;
pub use enhanced_scatter_plot::*;

use pika_core::plots::{PlotConfig, PlotType};
use egui::Ui;

/// Render any plot type based on configuration
pub fn render_plot_by_config(ui: &mut Ui, config: &PlotConfig, data: &arrow::record_batch::RecordBatch) {
    match config.plot_type {
        PlotType::Scatter => {
            let plot = ScatterPlot::from_config(config);
            plot.render(ui, data);
        }
        PlotType::Line => {
            let plot = LinePlot::from_config(config);
            plot.render(ui, data);
        }
        PlotType::Bar => {
            let plot = BarPlot::from_config(config);
            plot.render(ui, data);
        }
        PlotType::Histogram => {
            let plot = HistogramPlot::from_config(config);
            plot.render(ui, data);
        }
        PlotType::Heatmap => {
            let plot = HeatmapPlot::from_config(config);
            plot.render(ui, data);
        }
        PlotType::BoxPlot => {
            let plot = BoxPlot::from_config(config);
            plot.render(ui, data);
        }
        _ => {
            ui.label(format!("Plot type {:?} not yet implemented in UI", config.plot_type));
        }
    }
} 