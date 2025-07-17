use super::{Plot as PlotTrait, PlotData};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct CorrelationPlot;

impl PlotTrait for CorrelationPlot {
    fn name(&self) -> &'static str {
        "Correlation"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Correlation visualization coming soon - {} points", data.points.len()));
        });
    }
}
