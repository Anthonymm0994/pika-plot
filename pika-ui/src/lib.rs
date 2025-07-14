//! Pika UI - User interface components for the Pika data visualization platform.

// Module declarations
pub mod app;
pub mod canvas;
pub mod export;
pub mod nodes;
pub mod notifications;
pub mod panels;
pub mod plots;
pub mod renderer;
pub mod screens;
pub mod shortcuts;
pub mod state;
pub mod theme;
pub mod tooltip_ext;
pub mod widgets;
pub mod workspace;

// Public exports
pub use app::App;
pub use panels::canvas_panel::AppEvent;
pub use state::{AppState, ViewMode};
pub use state::Theme;

/// Initialize the UI system
pub fn init() {
    // Any initialization logic
}
