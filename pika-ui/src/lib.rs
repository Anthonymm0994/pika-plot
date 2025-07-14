//! UI components for the Pika-Plot application.

use anyhow::Result;

// Core modules that work
pub mod app;
pub mod canvas;
pub mod state;
pub mod theme;
pub mod tooltip_ext;
pub mod renderer;
pub mod shortcuts;

// Screens
pub mod screens;

// Panels - only basic ones for now
pub mod panels {
    pub mod canvas;
    pub mod canvas_panel;
    pub mod status_bar;
    pub mod properties;
    pub mod data_sources;
    pub mod canvas_toolbar;
    pub mod data;
    pub mod grid_view;
}

// Widgets - only basic ones for now
pub mod widgets {
    pub mod drag_drop;
    pub mod progress_indicator;
    pub mod plot_config;
    pub mod file_import_dialog;
}

// Nodes - only basic ones for now
pub mod nodes {
    pub mod plot_node;
}

// Plots - minimal implementation
pub mod plots {
    pub mod plot_renderer;
}

// TEMPORARILY DISABLED - Will be re-enabled once core functionality works
// pub mod export;
// pub mod export { pub mod csv_export; }

// Re-export commonly used types
pub use app::App;
pub use state::{AppState, Theme};
// pub use export::{export_plot, ExportFormat}; // TODO: Add export module later

/// Initialize the UI system
pub fn init() {
    // Any initialization logic
}
