use super::{Plot as PlotTrait, PlotData, PlotConfiguration};
use egui::Ui;
use datafusion::arrow::datatypes::DataType;

pub struct TimeAnalysisPlot;

impl PlotTrait for TimeAnalysisPlot {
    fn name(&self) -> &'static str { "Time Series Analysis" }
    fn required_x_types(&self) -> Option<Vec<DataType>> { None }
    fn required_y_types(&self) -> Vec<DataType> { vec![] }
    fn render(&self, ui: &mut Ui, _data: &PlotData, _config: &PlotConfiguration) {
        ui.centered_and_justified(|ui| {
            ui.label("Time series analysis visualization coming soon");
        });
    }
    fn prepare_data(&self, _query_result: &crate::core::QueryResult, _config: &PlotConfiguration) -> Result<PlotData, String> {
        Ok(PlotData {
            points: vec![],
            series: vec![],
            metadata: super::PlotMetadata {
                title: "Time Series Analysis".to_string(),
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
