//! Plot rendering engine that prepares data for GPU visualization.

use std::sync::Arc;
use pika_core::{
    error::{PikaError, Result},
    plots::{PlotConfig, PlotType, PlotDataConfig},
    types::QueryResult,
};
use crate::gpu::GpuManager;

/// Plot renderer that prepares data for GPU visualization
pub struct PlotRenderer {
    gpu_manager: Option<Arc<GpuManager>>,
}

impl PlotRenderer {
    /// Create a new plot renderer
    pub fn new(gpu_manager: Option<Arc<GpuManager>>) -> Self {
        PlotRenderer { gpu_manager }
    }
    
    /// Prepare plot data for rendering
    pub async fn prepare_plot(
        &self,
        query_result: &QueryResult,
        config: &PlotConfig,
    ) -> Result<pika_core::events::PlotRenderData> {
        // For now, create an empty RecordBatch
        // In a real implementation, this would extract data from the query result
        let schema = Arc::new(duckdb::arrow::datatypes::Schema::empty());
        let batch = duckdb::arrow::record_batch::RecordBatch::new_empty(schema);
        
        Ok(pika_core::events::PlotRenderData {
            data: Arc::new(batch),
            config: config.clone(),
        })
    }
    
    /// Extract data for scatter plot
    fn extract_scatter_data(
        &self,
        batch: &duckdb::arrow::record_batch::RecordBatch,
        config: &PlotDataConfig,
    ) -> Result<Vec<PlotPoint>> {
        match config {
            PlotDataConfig::ScatterConfig { x_column, y_column, .. } => {
                let x_values = super::data_extractor::extract_numeric_values(
                    batch.column_by_name(x_column)
                        .ok_or_else(|| PikaError::MissingField(x_column.clone()))?
                )?;
                
                let y_values = super::data_extractor::extract_numeric_values(
                    batch.column_by_name(y_column)
                        .ok_or_else(|| PikaError::MissingField(y_column.clone()))?
                )?;
                
                Ok(x_values.into_iter()
                    .zip(y_values)
                    .map(|(x, y)| PlotPoint { x, y })
                    .collect())
            }
            _ => Err(PikaError::InvalidPlotConfig("Expected scatter config".to_string())),
        }
    }
    
    /// Calculate plot bounds with padding
    fn calculate_bounds(points: &[PlotPoint]) -> PlotBounds {
        if points.is_empty() {
            return PlotBounds {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            };
        }
        
        let mut bounds = PlotBounds {
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            y_min: f64::INFINITY,
            y_max: f64::NEG_INFINITY,
        };
        
        for point in points {
            bounds.x_min = bounds.x_min.min(point.x);
            bounds.x_max = bounds.x_max.max(point.x);
            bounds.y_min = bounds.y_min.min(point.y);
            bounds.y_max = bounds.y_max.max(point.y);
        }
        
        // Add 5% padding
        let x_padding = (bounds.x_max - bounds.x_min) * 0.05;
        let y_padding = (bounds.y_max - bounds.y_min) * 0.05;
        
        bounds.x_min -= x_padding;
        bounds.x_max += x_padding;
        bounds.y_min -= y_padding;
        bounds.y_max += y_padding;
        
        bounds
    }
}

/// A single plot point
#[derive(Debug, Clone, Copy)]
pub struct PlotPoint {
    pub x: f64,
    pub y: f64,
}

/// Plot bounds in data space
#[derive(Debug, Clone, Copy)]
pub struct PlotBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// Rendering mode based on data size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Direct rendering for small datasets (< 50k points)
    Direct,
    /// Instanced rendering for medium datasets (50k - 5M points)
    Instanced,
    /// Aggregated rendering for large datasets (> 5M points)
    Aggregated,
}

impl RenderMode {
    /// Determine render mode based on point count
    pub fn from_point_count(count: usize) -> Self {
        match count {
            0..=50_000 => RenderMode::Direct,
            50_001..=5_000_000 => RenderMode::Instanced,
            _ => RenderMode::Aggregated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_mode_selection() {
        assert_eq!(RenderMode::from_point_count(100), RenderMode::Direct);
        assert_eq!(RenderMode::from_point_count(50_000), RenderMode::Direct);
        assert_eq!(RenderMode::from_point_count(50_001), RenderMode::Instanced);
        assert_eq!(RenderMode::from_point_count(5_000_000), RenderMode::Instanced);
        assert_eq!(RenderMode::from_point_count(5_000_001), RenderMode::Aggregated);
    }
    
    #[test]
    fn test_bounds_calculation() {
        let points = vec![
            PlotPoint { x: 0.0, y: 0.0 },
            PlotPoint { x: 10.0, y: 10.0 },
            PlotPoint { x: -5.0, y: 15.0 },
        ];
        
        let bounds = PlotRenderer::calculate_bounds(&points);
        
        // Check bounds with 5% padding
        assert!(bounds.x_min < -5.0);
        assert!(bounds.x_max > 10.0);
        assert!(bounds.y_min < 0.0);
        assert!(bounds.y_max > 15.0);
    }
} 