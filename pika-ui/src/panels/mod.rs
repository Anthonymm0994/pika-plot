//! UI panels module.

mod canvas_panel;
mod data;
mod properties;
mod status_bar;
mod grid_view;

pub use canvas_panel::CanvasPanel;
pub use data::DataPanel;
pub use properties::PropertiesPanel;
pub use status_bar::StatusBar;
pub use grid_view::{GridView, GridViewPanel}; 