use super::{Plot as PlotTrait, PlotData};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct Scatter3dPlot;

impl PlotTrait for Scatter3dPlot {
    fn name(&self) -> &'static str {
        "Scatter3d"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Scatter3d visualization coming soon - {} points", data.points.len()));
        });
    }
}
