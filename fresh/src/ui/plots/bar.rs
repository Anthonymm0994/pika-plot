use egui::{Ui, Color32};
use egui_plot::{Bar, BarChart, Plot, Legend};
use datafusion::arrow::datatypes::DataType;

use super::{Plot as PlotTrait, PlotData};

pub struct BarChartPlot;

impl PlotTrait for BarChartPlot {
    fn name(&self) -> &'static str {
        "Bar Chart"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        // Bar charts can have categorical or numeric X axis
        Some(vec![
            DataType::Utf8,
            DataType::LargeUtf8,
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
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        if data.points.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No data points to display");
            });
            return;
        }
        
        let bars: Vec<Bar> = data.points.iter()
            .enumerate()
            .map(|(i, point)| {
                let mut bar = Bar::new(point.x, point.y);
                if let Some(label) = &point.label {
                    bar = bar.name(label);
                }
                if let Some(color) = point.color {
                    bar = bar.fill(color);
                }
                bar
            })
            .collect();
        
        let chart = BarChart::new(bars)
            .name(&data.title)
            .color(Color32::from_rgb(100, 200, 100));
        
        let plot = Plot::new("bar_chart")
            .x_axis_label(&data.x_label)
            .y_axis_label(&data.y_label)
            .show_grid(data.show_grid);
        
        let plot = if data.show_legend {
            plot.legend(Legend::default())
        } else {
            plot
        };
        
        plot.show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }
} 