use egui::{Ui, Color32, Stroke};
use egui_plot::{Plot, PlotPoints, Points, Line, Legend, PlotUi};
use datafusion::arrow::datatypes::DataType;

use super::{Plot as PlotTrait, PlotData};

pub struct BoxPlotImpl;

impl PlotTrait for BoxPlotImpl {
    fn name(&self) -> &'static str {
        "Box Plot"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Box plots can group by categorical X axis or work without X
        Some(vec![
            DataType::Utf8,
            DataType::LargeUtf8,
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
        ])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        // Y axis must be numeric
        vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float16, DataType::Float32, DataType::Float64,
        ]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
            });
            return;
        }
        
        // Calculate box plot statistics
        let values: Vec<f64> = data.points.iter().map(|p| p.y).collect();
        let stats = calculate_box_plot_stats(&values);
        
        let plot = Plot::new("box_plot")
            .x_axis_label(&data.x_label)
            .y_axis_label(&data.y_label)
            .show_grid(data.show_grid);
        
        let plot = if data.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| {
            render_box_plot(plot_ui, &stats, 0.0, &data.title);
        });
    }
}

#[derive(Debug)]
struct BoxPlotStats {
    min: f64,
    q1: f64,
    median: f64,
    q3: f64,
    max: f64,
    outliers: Vec<f64>,
}

fn calculate_box_plot_stats(values: &[f64]) -> BoxPlotStats {
    if values.is_empty() {
        return BoxPlotStats {
            min: 0.0,
            q1: 0.0,
            median: 0.0,
            q3: 0.0,
            max: 0.0,
            outliers: vec![],
        };
    }
    
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let n = sorted.len();
    let q1 = percentile(&sorted, 0.25);
    let median = percentile(&sorted, 0.5);
    let q3 = percentile(&sorted, 0.75);
    
    let iqr = q3 - q1;
    let lower_fence = q1 - 1.5 * iqr;
    let upper_fence = q3 + 1.5 * iqr;
    
    let mut outliers = Vec::new();
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    
    for &value in &sorted {
        if value < lower_fence || value > upper_fence {
            outliers.push(value);
        } else {
            min = min.min(value);
            max = max.max(value);
        }
    }
    
    BoxPlotStats {
        min,
        q1,
        median,
        q3,
        max,
        outliers,
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let n = sorted_values.len();
    let index = p * (n - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = index - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
}

fn render_box_plot(plot_ui: &mut PlotUi, stats: &BoxPlotStats, x_pos: f64, name: &str) {
    let box_width = 0.3;
    let whisker_width = 0.15;
    
    // Box (Q1 to Q3)
    let box_points = vec![
        [x_pos - box_width/2.0, stats.q1],
        [x_pos + box_width/2.0, stats.q1],
        [x_pos + box_width/2.0, stats.q3],
        [x_pos - box_width/2.0, stats.q3],
        [x_pos - box_width/2.0, stats.q1],
    ];
    plot_ui.line(Line::new(PlotPoints::from(box_points))
        .color(Color32::from_rgb(100, 150, 250))
        .width(2.0)
        .name(name));
    
    // Median line
    plot_ui.line(Line::new(PlotPoints::from(vec![
        [x_pos - box_width/2.0, stats.median],
        [x_pos + box_width/2.0, stats.median],
    ])).color(Color32::from_rgb(250, 100, 100)).width(3.0));
    
    // Whiskers
    // Lower whisker
    plot_ui.line(Line::new(PlotPoints::from(vec![
        [x_pos, stats.q1],
        [x_pos, stats.min],
    ])).color(Color32::from_rgb(100, 150, 250)).width(1.0));
    
    plot_ui.line(Line::new(PlotPoints::from(vec![
        [x_pos - whisker_width/2.0, stats.min],
        [x_pos + whisker_width/2.0, stats.min],
    ])).color(Color32::from_rgb(100, 150, 250)).width(1.0));
    
    // Upper whisker
    plot_ui.line(Line::new(PlotPoints::from(vec![
        [x_pos, stats.q3],
        [x_pos, stats.max],
    ])).color(Color32::from_rgb(100, 150, 250)).width(1.0));
    
    plot_ui.line(Line::new(PlotPoints::from(vec![
        [x_pos - whisker_width/2.0, stats.max],
        [x_pos + whisker_width/2.0, stats.max],
    ])).color(Color32::from_rgb(100, 150, 250)).width(1.0));
    
    // Outliers
    if !stats.outliers.is_empty() {
        let outlier_points: PlotPoints = stats.outliers.iter()
            .map(|&y| [x_pos, y])
            .collect();
        plot_ui.points(Points::new(outlier_points)
            .color(Color32::from_rgb(250, 100, 100))
            .radius(3.0));
    }
} 