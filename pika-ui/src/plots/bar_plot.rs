//! Bar plot implementation.

use egui::Ui;
use egui_plot::{Plot, Bar, BarChart};

pub struct BarPlot;

impl BarPlot {
    pub fn render(&self, ui: &mut Ui) {
        Plot::new("bar_plot")
            .view_aspect(2.0)
            .show(ui, |plot_ui| {
                // Placeholder bar chart
                let bars = vec![
                    Bar::new(0.0, 1.0),
                    Bar::new(1.0, 2.0),
                    Bar::new(2.0, 1.5),
                ];
                plot_ui.bar_chart(BarChart::new(bars));
            });
    }
} 