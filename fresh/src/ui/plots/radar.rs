use super::{Plot as PlotTrait, PlotData, PlotConfiguration};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct RadarPlot;

impl PlotTrait for RadarPlot {
    fn name(&self) -> &'static str {
        "Radar"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData, _config: &PlotConfiguration) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Radar visualization coming soon - {} points", data.points.len()));
        });
    }
}
