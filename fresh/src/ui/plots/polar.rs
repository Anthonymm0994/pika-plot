use super::{Plot as PlotTrait, PlotData, PlotConfiguration};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct PolarPlot;

impl PlotTrait for PolarPlot {
    fn name(&self) -> &'static str {
        "Polar"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Polar visualization coming soon - {} points", data.points.len()));
        });
    }
}
