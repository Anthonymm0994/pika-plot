use egui::{Ui, Color32};
use egui_plot::{Plot, PlotPoints, Polygon, Legend};
use arrow::record_batch::RecordBatch;
use pika_core::plots::{PlotConfig, PlotDataConfig};
use pika_engine::plot::{extract_numeric_values, extract_category_values};
use crate::theme::{PlotTheme, get_theme_mode};
use std::collections::HashMap;

pub struct BoxPlot {
    category_column: Option<String>,
    value_column: String,
    show_outliers: bool,
    box_width: f32,
    show_legend: bool,
}

impl BoxPlot {
    pub fn from_config(config: &PlotConfig) -> Self {
        match &config.specific {
            PlotDataConfig::BoxPlotConfig {
                category_column,
                value_column,
                show_outliers,
                box_width,
            } => Self {
                category_column: category_column.clone(),
                value_column: value_column.clone(),
                show_outliers: *show_outliers,
                box_width: *box_width,
                show_legend: true,
            },
            _ => panic!("Invalid config for box plot"),
        }
    }
    
    pub fn render(&self, ui: &mut Ui, data: &RecordBatch) {
        // Get theme-aware colors
        let theme_mode = get_theme_mode(ui.ctx());
        let plot_theme = PlotTheme::for_mode(theme_mode);
        
        // Create plot
        let mut plot = Plot::new("box_plot")
            .legend(Legend::default())
            .show_grid(true)
            .show_axes([true, true]);
        
        // Apply theme colors to plot
        if theme_mode == crate::theme::ThemeMode::Dark {
            plot = plot.show_background(false);
        }
        
        plot.show(ui, |plot_ui| {
            if let Some(category_col) = &self.category_column {
                // Grouped box plot by category
                match extract_category_values(data, category_col, &self.value_column) {
                    Ok(category_values) => {
                        let mut grouped_data: HashMap<String, Vec<f64>> = HashMap::new();
                        
                        for (category, value) in category_values {
                            grouped_data.entry(category).or_insert_with(Vec::new).push(value);
                        }
                        
                        for (i, (category, values)) in grouped_data.iter().enumerate() {
                            let color = plot_theme.categorical_color(i);
                            self.render_box(plot_ui, values, i as f64, &color, Some(category));
                        }
                    }
                    Err(e) => {
                        // Fallback to single box plot
                        if let Some(value_array) = data.column_by_name(&self.value_column) {
                            if let Ok(values) = extract_numeric_values(value_array) {
                                let color = plot_theme.categorical_color(0);
                                self.render_box(plot_ui, &values, 0.0, &color, None);
                            }
                        }
                    }
                }
            } else {
                // Single box plot
                if let Some(value_array) = data.column_by_name(&self.value_column) {
                    match extract_numeric_values(value_array) {
                        Ok(values) => {
                            let color = plot_theme.categorical_color(0);
                            self.render_box(plot_ui, &values, 0.0, &color, Some(&self.value_column));
                        }
                        Err(e) => {
                            // Error will be displayed by the plot framework
                        }
                    }
                } else {
                    // Column not found - error will be displayed by the plot framework
                }
            }
        });
    }
    
    fn render_box(&self, plot_ui: &mut egui_plot::PlotUi, values: &[f64], x_position: f64, color: &Color32, label: Option<&str>) {
        if values.is_empty() {
            return;
        }
        
        // Calculate box plot statistics
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let q1 = percentile(&sorted_values, 25.0);
        let median = percentile(&sorted_values, 50.0);
        let q3 = percentile(&sorted_values, 75.0);
        let iqr = q3 - q1;
        
        let lower_whisker = q1 - 1.5 * iqr;
        let upper_whisker = q3 + 1.5 * iqr;
        
        let box_width = self.box_width as f64;
        let half_width = box_width / 2.0;
        
        // Draw box
        let box_points = vec![
            [x_position - half_width, q1],
            [x_position + half_width, q1],
            [x_position + half_width, q3],
            [x_position - half_width, q3],
        ];
        
        let fill_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 64);
        let box_polygon = Polygon::new(PlotPoints::new(box_points))
            .fill_color(fill_color)
            .stroke(egui::Stroke::new(2.0, *color));
        
        plot_ui.polygon(box_polygon);
        
        // Draw median line
        let median_points = vec![
            [x_position - half_width, median],
            [x_position + half_width, median],
        ];
        let median_line = egui_plot::Line::new(PlotPoints::new(median_points))
            .color(*color)
            .width(3.0);
        
        if let Some(name) = label {
            plot_ui.line(median_line.name(name));
        } else {
            plot_ui.line(median_line);
        }
        
        // Draw whiskers
        let lower_whisker_actual = sorted_values.iter().find(|&&v| v >= lower_whisker).copied().unwrap_or(sorted_values[0]);
        let upper_whisker_actual = sorted_values.iter().rev().find(|&&v| v <= upper_whisker).copied().unwrap_or(sorted_values[sorted_values.len() - 1]);
        
        // Lower whisker
        let lower_whisker_line = egui_plot::Line::new(PlotPoints::new(vec![
            [x_position, q1],
            [x_position, lower_whisker_actual],
        ]))
        .color(*color)
        .width(1.0);
        plot_ui.line(lower_whisker_line);
        
        // Upper whisker
        let upper_whisker_line = egui_plot::Line::new(PlotPoints::new(vec![
            [x_position, q3],
            [x_position, upper_whisker_actual],
        ]))
        .color(*color)
        .width(1.0);
        plot_ui.line(upper_whisker_line);
        
        // Draw outliers if enabled
        if self.show_outliers {
            let outliers: Vec<f64> = sorted_values.iter()
                .filter(|&&v| v < lower_whisker_actual || v > upper_whisker_actual)
                .copied()
                .collect();
            
            for outlier in outliers {
                let outlier_points = egui_plot::Points::new(PlotPoints::new(vec![[x_position, outlier]]))
                    .radius(2.0)
                    .color(*color)
                    .shape(egui_plot::MarkerShape::Circle);
                plot_ui.points(outlier_points);
            }
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (p / 100.0) * (sorted_values.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = index - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
} 