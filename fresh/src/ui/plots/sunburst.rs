use super::{Plot as PlotTrait, PlotData, PlotConfiguration};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct SunburstPlot;

impl PlotTrait for SunburstPlot {
    fn name(&self) -> &'static str { "Sunburst Chart" }
    fn required_x_types(&self) -> Option<Vec<DataType>> { None }
    fn required_y_types(&self) -> Vec<DataType> { vec![] }
    fn render(&self, ui: &mut Ui, _data: &PlotData, _config: &PlotConfiguration) {
        ui.centered_and_justified(|ui| {
            ui.label("Sunburst chart visualization coming soon");
        });
    }
    fn prepare_data(&self, _query_result: &crate::core::QueryResult, _config: &PlotConfiguration) -> Result<PlotData, String> {
        Ok(PlotData {
            points: vec![],
            series: vec![],
            metadata: super::PlotMetadata {
                title: "Sunburst Chart".to_string(),
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
