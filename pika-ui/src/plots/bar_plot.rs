use egui::{Ui, Color32};
use egui_plot::{Plot, Bar, BarChart};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig, BarOrientation};
use pika_engine::plot::extract_category_values;
use crate::theme::{PlotTheme, get_theme_mode};

pub struct BarPlot {
    category_column: String,
    value_column: String,
    orientation: BarOrientation,
    bar_width: f32,
    stacked: bool,
    show_legend: bool,
}

impl BarPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::BarConfig {
                category_column,
                value_column,
                orientation,
                bar_width,
                stacked,
            } => Self {
                category_column: category_column.clone(),
                value_column: value_column.clone(),
                orientation: *orientation,
                bar_width: *bar_width,
                stacked: *stacked,
                show_legend: true,
            },
            _ => panic!("Invalid config for bar plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        // Extract category-value pairs
        match extract_category_values(data, &self.category_column, &self.value_column) {
            Ok(category_values) => {
                // Create plot
                let mut plot = Plot::new("bar_plot")
                    .show_grid(true)
                    .show_axes([true, true]);
                
                // Apply theme colors to plot
                if theme_mode == crate::theme::ThemeMode::Dark {
                    plot = plot.show_background(false);
                }
                
                plot.show(ui, |plot_ui| {
                    let mut bars = Vec::new();
                    
                    for (i, (category, value)) in category_values.iter().enumerate() {
                        let color = plot_theme.categorical_color(i);
                        
                        let bar = Bar::new(i as f64, *value)
                            .width(self.bar_width as f64)
                            .name(category.clone())
                            .fill(color);
                        
                        bars.push(bar);
                    }
                    
                    let bar_chart = BarChart::new(bars);
                    plot_ui.bar_chart(bar_chart);
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error rendering bar plot: {}", e));
            }
        }
    }
} 