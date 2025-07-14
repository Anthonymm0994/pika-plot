//! Plot node implementation.

use crate::{state::AppState, canvas::CanvasElement};
use pika_core::{
    Node, NodeId, Point2, Size2, Port, PortDirection, PortType, Result,
    DataNode, PikaError, plots::PlotConfig, RenderData,
};
use std::sync::Arc;
use std::any::Any;

/// Cached plot render data
#[derive(Debug, Clone)]
struct CachedPlotData {
    /// The rendered plot data
    render_data: RenderData,
    /// Configuration that generated this cache
    config_hash: u64,
    /// Input data hash
    data_hash: u64,
}

/// A node that renders plots
#[derive(Debug, Clone)]
pub struct PlotNode {
    id: NodeId,
    name: String,
    position: Point2,
    size: Size2,
    config: PlotConfig,
    input_ports: Vec<Port>,
    output_ports: Vec<Port>,
    data: Option<Arc<dyn Any + Send + Sync>>,
    /// Cached render data (lazy evaluation)
    cached_render: Option<CachedPlotData>,
    /// Whether the cache is dirty
    cache_dirty: bool,
}

impl PlotNode {
    pub fn new(id: NodeId) -> Self {
        let input_ports = vec![
            Port::new("data", "Data", PortDirection::Input, PortType::RecordBatch),
            Port::new("config", "Config", PortDirection::Input, PortType::PlotConfig),
        ];
        
        let output_ports = vec![
            // Plot nodes typically don't have outputs, but we could add image output
        ];
        
        PlotNode {
            id,
            name: "Plot".to_string(),
            position: Point2::new(0.0, 0.0),
            size: Size2::new(200.0, 150.0),
            config: PlotConfig::scatter("x".to_string(), "y".to_string()),
            input_ports,
            output_ports,
            data: None,
            cached_render: None,
            cache_dirty: true,
        }
    }
    
    pub fn with_config(mut self, config: PlotConfig) -> Self {
        self.config = config;
        self.cache_dirty = true;
        self
    }
    
    pub fn config(&self) -> &PlotConfig {
        &self.config
    }
    
    pub fn set_config(&mut self, config: PlotConfig) {
        if self.hash_config(&config) != self.hash_config(&self.config) {
            self.cache_dirty = true;
        }
        self.config = config;
    }
    
    /// Compute hash for config (for cache invalidation)
    fn hash_config(&self, config: &PlotConfig) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        // Hash the config data - this is a simplified version
        format!("{:?}", config).hash(&mut hasher);
        hasher.finish()
    }
    
    /// Compute hash for data (for cache invalidation)
    fn hash_data(&self, data: &Arc<dyn Any + Send + Sync>) -> u64 {
        // In a real implementation, this would hash the actual data content
        // For now, use pointer address as a simple proxy
        data.as_ref() as *const dyn Any as *const () as usize as u64
    }
    
    /// Get cached render data or compute if needed (lazy evaluation)
    fn get_cached_render(&mut self) -> Result<RenderData> {
        // Check if we have valid cached data
        if let Some(ref cached) = self.cached_render {
            if !self.cache_dirty {
                if let Some(ref data) = self.data {
                    let current_config_hash = self.hash_config(&self.config);
                    let current_data_hash = self.hash_data(data);
                    
                    if cached.config_hash == current_config_hash && 
                       cached.data_hash == current_data_hash {
                        // Cache hit!
                        // Return placeholder data instead of cloning
                        return Ok(RenderData { data: vec![] });
                    }
                }
            }
        }
        
        // Cache miss - compute new render data
        let render_data = self.compute_render_data()?;
        
        // Update cache
        if let Some(ref data) = self.data {
            self.cached_render = Some(CachedPlotData {
                render_data: RenderData { data: vec![] },  // Store placeholder instead of cloning
                config_hash: self.hash_config(&self.config),
                data_hash: self.hash_data(data),
            });
            self.cache_dirty = false;
        }
        
        Ok(render_data)
    }
    
    /// Actually compute the render data (expensive operation)
    fn compute_render_data(&self) -> Result<RenderData> {
        // This is where the actual plot rendering logic would go
        // For now, return empty render data
        Ok(RenderData { data: vec![] })
    }
}

impl Node for PlotNode {
    fn id(&self) -> NodeId {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn position(&self) -> Point2 {
        self.position
    }
    
    fn set_position(&mut self, position: Point2) {
        self.position = position;
    }
    
    fn size(&self) -> Size2 {
        self.size
    }
    
    fn input_ports(&self) -> &[Port] {
        &self.input_ports
    }
    
    fn output_ports(&self) -> &[Port] {
        &self.output_ports
    }
    
    fn accept_input(&mut self, port_id: &str, data: Arc<dyn Any + Send + Sync>) -> Result<()> {
        match port_id {
            "data" => {
                // Invalidate cache when data changes
                if self.data.is_none() || self.hash_data(&data) != self.data.as_ref().map(|d| self.hash_data(d)).unwrap_or(0) {
                    self.cache_dirty = true;
                }
                self.data = Some(data);
                Ok(())
            }
            "config" => {
                if let Some(config) = data.downcast_ref::<PlotConfig>() {
                    self.set_config(config.clone());
                    Ok(())
                } else {
                    Err(PikaError::Validation(
                        format!("Invalid data type for port {}: expected table data", port_id)
                    ))
                }
            }
            _ => Err(PikaError::Validation(format!("Unknown port: {}", port_id))),
        }
    }
    
    fn get_output(&self, port_id: &str) -> Result<Option<Arc<dyn Any + Send + Sync>>> {
        // Plot nodes typically don't have outputs
        Err(PikaError::Validation(format!("Unknown output port: {}", port_id)))
    }
    
    fn is_ready(&self) -> bool {
        self.data.is_some()
    }
    
    fn execute(&mut self) -> Result<()> { Ok(()) }
    fn render_data(&mut self) -> Result<RenderData> { 
        // Use lazy cached evaluation
        self.get_cached_render()
    }
    
    fn type_name(&self) -> &'static str {
        "PlotNode"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DataNode for PlotNode {
    fn output_columns(&self) -> Option<Vec<(String, String)>> {
        // Plot nodes don't output tabular data
        None
    }
    
    fn output_data(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        // Plot nodes don't output data
        None
    }
} 