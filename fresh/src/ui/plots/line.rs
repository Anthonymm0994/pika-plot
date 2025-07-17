use egui::{Ui, Color32};
use egui_plot::{Line, Plot, PlotPoints, Legend};
use datafusion::arrow::datatypes::DataType;

use super::{Plot as PlotTrait, PlotData};

pub struct LineChartPlot;

impl PlotTrait for LineChartPlot {
    fn name(&self) -> &'static str {
        "Line Chart"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Line charts typically need numeric or temporal X axis
        Some(vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float32, DataType::Float64,
            DataType::Date32, DataType::Date64,
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Millisecond, None),
        ])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        // Y axis must be numeric
        vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float16, DataType::Float32, DataType::Float64,
            DataType::Decimal128(38, 10), DataType::Decimal256(76, 10),
        ]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
            });
            return;
        }
        
        // Sort points by X value for proper line rendering
        let mut sorted_points = data.points.clone();
        sorted_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        
        let plot_points: PlotPoints = sorted_points.iter()
            .map(|p| [p.x, p.y])
            .collect();
        
        let line = Line::new(plot_points)
            .name(&data.title)
            .color(Color32::from_rgb(100, 150, 250));
        
        let plot = Plot::new("line_chart")
            .x_axis_label(&data.x_label)
            .y_axis_label(&data.y_label)
            .show_grid(data.show_grid);
        
        let plot = if data.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| plot_ui.line(line));
    }
} 