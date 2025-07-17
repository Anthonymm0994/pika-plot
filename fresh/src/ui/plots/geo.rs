use super::{Plot as PlotTrait, PlotData};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct GeoPlot;

impl PlotTrait for GeoPlot {
    fn name(&self) -> &'static str {
        "Geo"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Geo visualization coming soon - {} points", data.points.len()));
        });
    }
}
