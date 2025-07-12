//! Plot rendering functionality.

pub mod data_extractor;
pub mod renderer;

pub use data_extractor::{
    extract_numeric_values,
    extract_string_values,
    extract_timestamp_values,
    extract_xy_points,
    extract_xyz_points,
    extract_category_values,
    aggregate_by_category,
    extract_time_series,
    extract_ohlc_data,
};
pub use renderer::{PlotRenderer, PlotPoint, PlotBounds, RenderMode}; 