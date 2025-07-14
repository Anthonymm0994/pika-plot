//! UI panels for the application.

pub mod canvas;
pub mod canvas_panel;
pub mod data;
pub mod grid_view;
pub mod properties;
pub mod status_bar;
pub mod data_sources;
pub mod canvas_toolbar;

// Re-export commonly used items
pub use self::canvas::CanvasPanel;
pub use self::properties::PropertiesPanel;
pub use self::status_bar::StatusBar; 