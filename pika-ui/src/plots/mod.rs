//! Plot rendering UI components

mod scatter_plot;
mod line_plot;
mod bar_plot;
mod histogram_plot;
mod heatmap_plot;
mod box_plot;
mod plot_renderer;

pub use scatter_plot::ScatterPlot;
pub use line_plot::LinePlot;
pub use bar_plot::BarPlot;
pub use histogram_plot::HistogramPlot;
pub use heatmap_plot::HeatmapPlot;
pub use box_plot::BoxPlot;
pub use plot_renderer::{PlotRenderer, render_plot};

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