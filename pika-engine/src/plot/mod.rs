//! Plot rendering functionality.

mod data_extractor;
mod renderer;

pub use renderer::{PlotRenderer, PlotBounds};
pub use data_extractor::extract_numeric_values; 