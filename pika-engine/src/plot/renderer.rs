//! Plot rendering implementation.

use pika_core::{
    error::{PikaError, Result},
    events::{PlotRenderData, PlotBounds, RenderMode},
    plots::{PlotConfig, PlotDataConfig},
    types::QueryResult,
};
use crate::gpu::{GpuManager, PlotVertex, PlotInstance, PlotUniforms};
use crate::plot::data_extractor;
use std::sync::Arc;
use arrow::record_batch::RecordBatch;

/// Plot renderer for GPU-accelerated visualization
pub struct PlotRenderer {
    gpu: Arc<GpuManager>,
}

impl PlotRenderer {
    /// Create a new plot renderer
    pub fn new(gpu: Arc<GpuManager>) -> Self {
        PlotRenderer { gpu }
    }
    
    /// Prepare plot data for rendering
    pub fn prepare_plot_data(
        &self,
        config: &PlotConfig,
        query_result: &QueryResult,
    ) -> Result<PlotRenderData> {
        // For now, use the first batch if available
        let batch = query_result.batches.first()
            .ok_or_else(|| PikaError::Internal("No data batches available".to_string()))?;
        
        // Extract data points based on plot type
        let point_data = match &config.specific {
            PlotDataConfig::ScatterConfig { x_column, y_column, .. } => {
                let points = data_extractor::extract_xy_points(batch, x_column, y_column)?;
                points.into_iter().map(|(x, y)| (x as f32, y as f32)).collect()
            }
            PlotDataConfig::LineConfig { x_column, y_column, .. } => {
                let mut points = data_extractor::extract_xy_points(batch, x_column, y_column)?;
                // Sort by x for line plots
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                points.into_iter().map(|(x, y)| (x as f32, y as f32)).collect()
            }
            PlotDataConfig::BarConfig { category_column, value_column, .. } => {
                let pairs = data_extractor::extract_category_values(batch, category_column, value_column)?;
                let aggregated = data_extractor::aggregate_by_category(pairs);
                
                // Convert categories to x positions
                let mut points = Vec::new();
                for (i, (_category, value)) in aggregated.iter().enumerate() {
                    points.push((i as f32, *value as f32));
                }
                points
            }
            PlotDataConfig::HistogramConfig { column, num_bins, .. } => {
                let values = data_extractor::extract_numeric_values(
                    batch.column_by_name(column)
                        .ok_or_else(|| PikaError::Internal(format!("Column '{}' not found", column)))?
                )?;
                
                // Calculate histogram bins
                let (min, max) = values.iter()
                    .filter(|v| !v.is_nan())
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
                        (min.min(v), max.max(v))
                    });
                
                let bin_width = (max - min) / *num_bins as f64;
                let mut bins = vec![0; *num_bins];
                
                for value in values.iter().filter(|v| !v.is_nan()) {
                    let bin_idx = ((value - min) / bin_width).floor() as usize;
                    let bin_idx = bin_idx.min(*num_bins - 1);
                    bins[bin_idx] += 1;
                }
                
                // Convert to points
                let mut points = Vec::new();
                for (i, &count) in bins.iter().enumerate() {
                    let x = min + (i as f64 + 0.5) * bin_width;
                    points.push((x as f32, count as f32));
                }
                points
            }
            _ => return Err(PikaError::NotImplemented {
                feature: format!("Plot type {:?} not yet implemented", config.plot_type)
            }),
        };
        
        // Determine render mode based on data size
        let render_mode = self.gpu.select_render_mode(point_data.len());
        
        // Calculate bounds
        let bounds = calculate_bounds(&point_data);
        
        // Prepare GPU buffers based on render mode
        let gpu_buffers = match render_mode {
            RenderMode::Direct => self.create_direct_buffers(&point_data)?,
            RenderMode::Instanced => self.prepare_instanced_buffers(&point_data)?,
            RenderMode::Aggregated => self.create_aggregated_buffers(&point_data)?,
        };
        
        Ok(PlotRenderData {
            bounds,
            point_count: point_data.len(),
            render_mode,
            vertex_data: Arc::new(vec![]), // TODO: Convert buffers to bytes
        })
    }
    
    /// Create buffers for direct rendering
    fn create_direct_buffers(&self, points: &[(f32, f32)]) -> Result<GpuBuffers> {
        // Create vertex data
        let vertices: Vec<PlotVertex> = points
            .iter()
            .map(|&(x, y)| PlotVertex {
                position: [x, y],
                color: [0.2, 0.6, 1.0, 1.0], // Blue
            })
            .collect();
        
        let vertex_buffer = self.gpu.create_buffer(
            &vertices,
            wgpu::BufferUsages::VERTEX,
        );
        
        Ok(GpuBuffers {
            vertex_buffer: Some(vertex_buffer),
            index_buffer: None,
            instance_buffer: None,
            uniform_buffer: None,
            vertex_count: vertices.len() as u32,
            instance_count: 0,
        })
    }
    
    /// Prepare buffers for instanced rendering
    fn prepare_instanced_buffers(&self, points: &[(f32, f32)]) -> Result<GpuBuffers> {
        // Create base mesh (e.g., a quad)
        let vertices = vec![
            PlotVertex {
                position: [-1.0, -1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PlotVertex {
                position: [1.0, -1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PlotVertex {
                position: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            PlotVertex {
                position: [-1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        
        let vertex_buffer = self.gpu.create_buffer(
            &vertices,
            wgpu::BufferUsages::VERTEX,
        );
        
        // Create instance data
        let instances: Vec<PlotInstance> = points
            .iter()
            .map(|&(x, y)| PlotInstance {
                position: [x, y],
                size: 5.0,
                color: [0.2, 0.6, 1.0, 1.0],
            })
            .collect();
        
        let instance_buffer = self.gpu.create_buffer(
            &instances,
            wgpu::BufferUsages::VERTEX,
        );
        
        Ok(GpuBuffers {
            vertex_buffer: Some(vertex_buffer),
            index_buffer: None,
            instance_buffer: Some(instance_buffer),
            uniform_buffer: None,
            vertex_count: vertices.len() as u32,
            instance_count: instances.len() as u32,
        })
    }
    
    /// Prepare buffers for aggregated rendering
    fn create_aggregated_buffers(&self, points: &[(f32, f32)]) -> Result<GpuBuffers> {
        // For now, fall back to direct rendering
        // TODO: Implement actual aggregation on GPU
        self.create_direct_buffers(points)
    }
    
    /// Render to an egui painter callback
    pub fn render_to_egui(
        &self,
        plot_data: &PlotRenderData,
        info: &egui::PaintCallbackInfo,
        painter: &egui_wgpu::Renderer,
    ) {
        // This would integrate with egui's paint callback system
        // The actual implementation would use the wgpu render pass
    }
}

/// GPU buffers for rendering
pub struct GpuBuffers {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub uniform_buffer: Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub instance_count: u32,
}

/// Calculate plot bounds with padding
fn calculate_bounds(points: &[(f32, f32)]) -> PlotBounds {
    if points.is_empty() {
        return PlotBounds {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        };
    }
    
    let mut bounds = PlotBounds {
        x_min: points[0].0 as f64,
        x_max: points[0].0 as f64,
        y_min: points[0].1 as f64,
        y_max: points[0].1 as f64,
    };
    
    for &(x, y) in points {
        bounds.x_min = bounds.x_min.min(x as f64);
        bounds.x_max = bounds.x_max.max(x as f64);
        bounds.y_min = bounds.y_min.min(y as f64);
        bounds.y_max = bounds.y_max.max(y as f64);
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