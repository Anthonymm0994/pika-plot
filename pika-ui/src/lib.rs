//! UI components for the Pika-Plot application.

use pika_core::{
    events::EventBus,
    error::Result,
    types::NodeId,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// Core UI modules
pub mod app;
pub mod canvas;
pub mod state;
pub mod theme;
pub mod renderer;

// UI components
pub mod nodes;
pub mod panels;
pub mod plots;
pub mod widgets;

// Features
pub mod shortcuts;
pub mod tooltip_ext;
pub mod notifications;

// Re-export main types
pub use app::PikaApp;
pub use state::AppState;
pub use theme::{apply_dark_theme, apply_light_theme};

/// Initialize the UI system
pub fn init() -> Result<()> {
    tracing::info!("Initializing Pika UI");
    Ok(())
}
