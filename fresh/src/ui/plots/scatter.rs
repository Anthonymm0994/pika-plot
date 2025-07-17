use egui::{Ui, Color32};
use egui_plot::{Points, Plot, PlotPoints, Legend, MarkerShape};
use datafusion::arrow::datatypes::DataType;

use super::{Plot as PlotTrait, PlotData};

pub struct ScatterPlotImpl;

impl PlotTrait for ScatterPlotImpl {
    fn name(&self) -> &'static str {
        "Scatter Plot"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Scatter plots need numeric X axis
        Some(vec![
            DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
            DataType::Float32, DataType::Float64,
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
    
    fn optional_column_types(&self) -> Vec<(&'static str, Vec<DataType>)> {
        vec![
            ("Size", vec![
                DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
                DataType::UInt8, DataType::UInt16, DataType::UInt32, DataType::UInt64,
                DataType::Float32, DataType::Float64,
            ]),
            ("Color", vec![
                DataType::Utf8, DataType::LargeUtf8,
                DataType::Int8, DataType::Int16, DataType::Int32, DataType::Int64,
            ]),
        ]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
            });
            return;
        }
        
        let plot_points: PlotPoints = data.points.iter()
            .map(|p| [p.x, p.y])
            .collect();
        
        let mut points = Points::new(plot_points)
            .name(&data.title)
            .radius(5.0)
            .shape(MarkerShape::Circle)
            .color(Color32::from_rgb(100, 200, 200));
        
        // Apply custom sizes if available
        if let Some(first_point) = data.points.first() {
            if let Some(size) = first_point.size {
                points = points.radius(size);
            }
        }
        
        let plot = Plot::new("scatter_plot")
            .x_axis_label(&data.x_label)
            .y_axis_label(&data.y_label)
            .show_grid(data.show_grid)
            .data_aspect(1.0);
        
        let plot = if data.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| plot_ui.points(points));
    }
} 