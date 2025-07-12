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
    use crate::plots::{PlotConfig, PlotType, PlotDataConfig};
    use crate::workspace::WorkspaceMode;
    use std::collections::HashSet;

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
            WorkspaceMode::Canvas => {
                // Default is Canvas mode
            }
            _ => panic!("Expected Canvas mode as default"),
        }
    }

    #[test]
    fn test_error_display() {
        let err = PikaError::internal("Test error message");
        let msg = err.to_string();
        assert!(msg.contains("Test error message"));
    }

    #[test]
    fn test_plot_config_scatter() {
        let config = PlotConfig::scatter("x".to_string(), "y".to_string());
        assert_eq!(config.plot_type, PlotType::Scatter);
        assert_eq!(config.x_label, Some("x".to_string()));
        assert_eq!(config.y_label, Some("y".to_string()));
        
        match config.specific {
            PlotDataConfig::ScatterConfig { x_column, y_column, .. } => {
                assert_eq!(x_column, "x");
                assert_eq!(y_column, "y");
            }
            _ => panic!("Expected ScatterConfig"),
        }
    }

    #[test]
    fn test_all_plot_types() {
        // Test that all plot types are represented
        let plot_types = vec![
            PlotType::Scatter,
            PlotType::Line,
            PlotType::Bar,
            PlotType::Histogram,
            PlotType::BoxPlot,
            PlotType::Violin,
            PlotType::Heatmap,
            PlotType::Correlation,
            PlotType::Scatter3D,
            PlotType::Surface3D,
            PlotType::Contour,
            PlotType::TimeSeries,
            PlotType::Candlestick,
            PlotType::Stream,
            PlotType::Treemap,
            PlotType::Sunburst,
            PlotType::Sankey,
            PlotType::Network,
            PlotType::Radar,
            PlotType::Polar,
            PlotType::ParallelCoordinates,
            PlotType::Geo,
            PlotType::Anomaly,
            PlotType::Distribution,
        ];
        
        // Ensure each type is unique
        let mut seen = std::collections::HashSet::new();
        for pt in plot_types {
            assert!(seen.insert(pt), "Duplicate plot type: {:?}", pt);
        }
    }
}
