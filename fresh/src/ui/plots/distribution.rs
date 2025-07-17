use super::{Plot as PlotTrait, PlotData};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct DistributionPlot;

impl PlotTrait for DistributionPlot {
    fn name(&self) -> &'static str {
        "Distribution"
    }
    
    fn required_x_types(&self) -> Option<Vec<DataType>> {
        Some(vec![DataType::Float64])
    }
    
    fn required_y_types(&self) -> Vec<DataType> {
        vec![DataType::Float64]
    }
    
    fn render(&self, ui: &mut Ui, data: &PlotData) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Distribution visualization coming soon - {} points", data.points.len()));
        });
    }
}
