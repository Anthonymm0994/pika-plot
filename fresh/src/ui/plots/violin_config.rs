/// Configuration for violin plots
#[derive(Debug, Clone)]
pub struct ViolinPlotConfig {
    pub bandwidth: f32,
    pub show_box_plot: bool,
    pub show_mean: bool,
    pub show_median: bool,
    pub show_quartiles: bool,
    pub show_outliers: bool,
    pub violin_width: f32,
}

impl Default for ViolinPlotConfig {
    fn default() -> Self {
        Self {
            bandwidth: 0.5,
            show_box_plot: true,
            show_mean: true,
            show_median: true,
            show_quartiles: true,
            show_outliers: true,
            violin_width: 0.8,
        }
    }
}