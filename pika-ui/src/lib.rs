//! UI components for the Pika-Plot application.

use pika_core::{
    events::EventBus,
    error::Result,
    types::NodeId,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// Core modules that work
pub mod app;
pub mod canvas;
pub mod state;
pub mod theme;
pub mod tooltip_ext;
pub mod renderer;
pub mod shortcuts;

// Panels - only basic ones for now
pub mod panels {
    pub mod canvas_panel;
    pub mod status_bar;
    pub mod properties;
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
// pub mod workspace;
// pub mod notifications;
// pub mod advanced_widgets;

// Re-exports for convenience
pub use app::PikaApp;
pub use state::AppState;

/// Initialize the UI system
pub fn init() -> Result<()> {
    tracing::info!("Initializing Pika UI");
    Ok(())
}
