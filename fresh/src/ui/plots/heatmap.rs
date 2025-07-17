use super::{Plot as PlotTrait, PlotData};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct HeatmapPlot;

impl PlotTrait for HeatmapPlot {
    fn name(&self) -> &'static str {
        "Heatmap"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![
            DataType::Utf8, DataType::Int64, DataType::Float64
        ])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64, DataType::Int64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Heatmap visualization coming soon - {} points", data.points.len()));
        });
    }
}
