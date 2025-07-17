use super::{Plot as PlotTrait, PlotData, PlotConfiguration};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct GeoPlot;

impl PlotTrait for GeoPlot {
    fn name(&self) -> &'static str { "Geographic Plot" }
    fn required_x_types(&self) -> Option<Vec<DataType>> { None }
    fn required_y_types(&self) -> Vec<DataType> { vec![] }
    fn render(&self, ui: &mut Ui, _data: &PlotData, _config: &PlotConfiguration) {
        ui.centered_and_justified(|ui| {
            ui.label("Geographic plot visualization coming soon");
        });
    }
    fn prepare_data(&self, _query_result: &crate::core::QueryResult, _config: &PlotConfiguration) -> Result<PlotData, String> {
        Ok(PlotData {
            points: vec![],
            series: vec![],
            metadata: super::PlotMetadata {
                title: "Geographic Plot".to_string(),
                x_label: "".to_string(),
                y_label: "".to_string(),
                show_legend: true,
                show_grid: true,
                color_scheme: super::ColorScheme::default(),
            },
            statistics: None,
        })
    }
}
