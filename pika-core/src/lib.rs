//! Core types and functionality for Pika-Plot.

#![warn(missing_docs)]

pub mod error;
pub mod events;
pub mod nodes;
pub mod plots;
pub mod types;
pub mod utils;
pub mod workspace;
pub mod snapshot;
pub mod node;

// Re-export commonly used types
pub use error::{PikaError, Result};
pub use types::{NodeId, PortId, Connection, Point2, Size2, Camera2D, WindowId};
pub use plots::{PlotConfig, PlotType};
pub use events::{Event, EventBus};
pub use node::{Node, Port, PortType, PortDirection, NodeContext, DataNode};

/// Prelude for common imports
pub mod prelude {
    pub use crate::{
        error::{PikaError, Result},
        types::{NodeId, PortId, Connection, Point2, Size2, Camera2D, WindowId},
        plots::{PlotConfig, PlotType},
        events::{Event, EventBus},
        node::{Node, Port, PortType, PortDirection},
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plots::{PlotConfig, PlotType, AxisConfig, AxisScale, ColorScale, PlotSpecificConfig, PlotTheme};
    use crate::workspace::WorkspaceMode;
    use crate::types::Camera2D;
    use std::collections::HashMap;

    #[test]
    fn test_node_id_generation() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_workspace_mode_default() {
        let mode = WorkspaceMode::default();
        match mode {
            WorkspaceMode::Notebook { cells, active_cell } => {
                assert_eq!(cells.len(), 0);
                assert_eq!(active_cell, None);
            }
            _ => panic!("Expected Notebook mode as default"),
        }
    }

    #[test]
    fn test_canvas_mode_creation() {
        let mode = WorkspaceMode::Canvas {
            nodes: HashMap::new(),
            connections: Vec::new(),
            camera: Camera2D::default(),
        };
        match mode {
            WorkspaceMode::Canvas { nodes, connections, .. } => {
                assert_eq!(nodes.len(), 0);
                assert_eq!(connections.len(), 0);
            }
            _ => panic!("Expected Canvas mode"),
        }
    }

    #[test]
    fn test_error_display() {
        let err = PikaError::InsufficientMemory {
            required: 1000, // MB
            available: 500, // MB
        };
        let msg = err.to_string();
        assert!(msg.contains("Not enough memory"));
        assert!(msg.contains("1000MB"));
        assert!(msg.contains("500MB"));
    }

    #[test]
    fn test_plot_config_scatter() {
        let config = PlotConfig {
            plot_type: PlotType::Scatter,
            title: Some("Test Plot".to_string()),
            x_axis: AxisConfig {
                column: "x".to_string(),
                label: Some("X Axis".to_string()),
                scale: AxisScale::Linear,
                range: None,
                tick_format: None,
            },
            y_axis: AxisConfig {
                column: "y".to_string(),
                label: Some("Y Axis".to_string()),
                scale: AxisScale::Linear,
                range: None,
                tick_format: None,
            },
            color_scale: ColorScale::default(),
            theme: PlotTheme::default(),
            specific: PlotSpecificConfig::Scatter {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                size_column: None,
                color_column: None,
                label_column: None,
                point_size: 5.0,
                opacity: 1.0,
                jitter: None,
            },
        };
        assert_eq!(config.plot_type, PlotType::Scatter);
        assert_eq!(config.title, Some("Test Plot".to_string()));
    }

    #[test]
    fn test_all_plot_types() {
        // Test that all plot types are represented
        let plot_types = vec![
            PlotType::Scatter,
            PlotType::Line,
            PlotType::Bar,
            PlotType::Histogram,
            PlotType::Heatmap,
            PlotType::Box,
            PlotType::Violin,
            PlotType::Area,
            PlotType::Pie,
            PlotType::Donut,
            PlotType::Treemap,
            PlotType::Sunburst,
            PlotType::Sankey,
            PlotType::Network,
            PlotType::Geo,
        ];
        
        // Ensure each type is unique
        let mut seen = std::collections::HashSet::new();
        for pt in plot_types {
            assert!(seen.insert(pt), "Duplicate plot type: {:?}", pt);
        }
    }
}
