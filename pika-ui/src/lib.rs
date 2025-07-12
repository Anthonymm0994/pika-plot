//! Pika-Plot UI library.

use pika_core::{
    events::{AppEvent, EventChannel},
    types::NodeId,
    error::Result,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main UI structure
pub struct PikaUI {
    config: UIConfig,
}

impl PikaUI {
    /// Create a new UI instance
    pub fn new(config: UIConfig) -> Self {
        Self { config }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &UIConfig {
        &self.config
    }
}

/// UI configuration
pub struct UIConfig {
    pub theme: Theme,
    pub canvas_config: CanvasConfig,
}

/// Theme configuration
pub enum Theme {
    Light,
    Dark,
    Auto,
}

/// Canvas configuration
pub struct CanvasConfig {
    pub grid_size: f32,
    pub snap_threshold: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            canvas_config: CanvasConfig::default(),
        }
    }
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            grid_size: 20.0,
            snap_threshold: 10.0,
            zoom_speed: 0.1,
            pan_speed: 1.0,
        }
    }
}

pub mod app;
pub mod canvas;
pub mod export;
pub mod nodes;
pub mod panels;
pub mod state;
pub mod theme;
pub mod widgets;
pub mod workspace;
pub mod plots;

pub use app::PikaApp;
pub use canvas::{CanvasState, Camera2D, BreadcrumbTrail};
pub use export::{ExportManager, ExportType, ExportFormat};
pub use nodes::{TableNode, QueryNode, PlotNode};
pub use state::{AppState, ViewMode};
pub use workspace::{Workspace, WorkspaceMode};
