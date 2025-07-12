//! Event system for inter-component communication.

use tokio::sync::broadcast;
use std::sync::Arc;

use crate::{
    types::{NodeId, Point2, TableInfo, QueryResult},
    plots::PlotConfig,
};

/// Events for UI-Engine communication
#[derive(Debug, Clone)]
pub enum AppEvent {
    // ===== UI -> Engine Events =====
    
    /// Import a CSV file
    ImportCsv {
        path: std::path::PathBuf,
        options: crate::types::ImportOptions,
    },
    
    /// Execute a SQL query
    ExecuteQuery {
        id: NodeId,
        sql: String,
        cache_key: Option<crate::types::QueryFingerprint>,
    },
    
    /// Prepare plot data for rendering
    PreparePlot {
        id: NodeId,
        source: NodeId,
        config: PlotConfig,
    },
    
    /// Cancel an ongoing operation
    CancelOperation {
        id: NodeId,
    },
    
    /// Clear cache (partial or full)
    ClearCache {
        query_cache: bool,
        gpu_cache: bool,
    },
    
    /// Shutdown the engine
    Shutdown,
    
    // ===== Engine -> UI Events =====
    
    /// Import has started
    ImportStarted {
        path: std::path::PathBuf,
    },
    
    /// Import progress update
    ImportProgress {
        path: std::path::PathBuf,
        progress: f32, // 0.0 to 1.0
    },
    
    /// Import completed successfully
    ImportComplete {
        path: std::path::PathBuf,
        table_info: TableInfo,
    },
    
    /// Import failed
    ImportError {
        path: std::path::PathBuf,
        error: String,
    },
    
    /// Query execution started
    QueryStarted {
        id: NodeId,
    },
    
    /// Query completed
    QueryComplete {
        id: NodeId,
        result: Result<QueryResult, String>,
    },
    
    /// Plot data is ready for GPU rendering
    PlotDataReady {
        id: NodeId,
        data: PlotRenderData,
    },
    
    /// Memory usage warning
    MemoryWarning {
        used_mb: usize,
        available_mb: usize,
        threshold: MemoryThreshold,
    },
    
    /// General error event
    Error {
        context: String,
        error: String,
    },
    
    /// Engine is ready
    EngineReady,
    
    /// Engine is shutting down
    EngineShutdown,
}

/// Memory warning threshold levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryThreshold {
    /// 60-80% usage
    Warning,
    /// 80-90% usage
    Critical,
    /// >90% usage
    Severe,
}

/// Plot data ready for rendering
#[derive(Debug, Clone)]
pub struct PlotRenderData {
    pub bounds: PlotBounds,
    pub point_count: usize,
    pub render_mode: RenderMode,
    pub vertex_data: Arc<Vec<u8>>, // Pre-formatted vertex data
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

/// Event channel helper for UI-Engine communication
pub struct EventChannel {
    sender: tokio::sync::broadcast::Sender<AppEvent>,
    receiver: tokio::sync::broadcast::Receiver<AppEvent>,
}

impl EventChannel {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(1024);
        Self { sender, receiver }
    }
    
    pub fn sender(&self) -> tokio::sync::broadcast::Sender<AppEvent> {
        self.sender.clone()
    }
    
    pub fn receiver(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
} 

/// Comprehensive event bus for UI and engine communication
#[derive(Clone)]
pub struct EventBus {
    /// Main app events channel
    app_events: tokio::sync::broadcast::Sender<AppEvent>,
    
    /// Canvas-specific events channel
    canvas_events: tokio::sync::broadcast::Sender<CanvasEvent>,
    
    /// Node events channel
    node_events: tokio::sync::broadcast::Sender<NodeEvent>,
    
    /// Window events channel
    window_events: tokio::sync::broadcast::Sender<WindowEvent>,
}

impl EventBus {
    /// Create a new event bus with default channel sizes
    pub fn new() -> Self {
        EventBus {
            app_events: tokio::sync::broadcast::channel(1024).0,
            canvas_events: tokio::sync::broadcast::channel(512).0,
            node_events: tokio::sync::broadcast::channel(512).0,
            window_events: tokio::sync::broadcast::channel(256).0,
        }
    }
    
    /// Get the app events sender
    pub fn app_events_sender(&self) -> broadcast::Sender<AppEvent> {
        self.app_events.clone()
    }
    
    /// Get the canvas events sender
    pub fn canvas_events_sender(&self) -> broadcast::Sender<CanvasEvent> {
        self.canvas_events.clone()
    }
    
    /// Subscribe to app events
    pub fn subscribe_app_events(&self) -> broadcast::Receiver<AppEvent> {
        self.app_events.subscribe()
    }
    
    /// Subscribe to canvas events
    pub fn subscribe_canvas_events(&self) -> broadcast::Receiver<CanvasEvent> {
        self.canvas_events.subscribe()
    }
    
    /// Subscribe to node events
    pub fn subscribe_node_events(&self) -> broadcast::Receiver<NodeEvent> {
        self.node_events.subscribe()
    }
    
    /// Subscribe to window events
    pub fn subscribe_window_events(&self) -> broadcast::Receiver<WindowEvent> {
        self.window_events.subscribe()
    }
    
    /// Send an app event
    pub fn send_app_event(&self, event: AppEvent) -> Result<usize, broadcast::error::SendError<AppEvent>> {
        self.app_events.send(event)
    }
    
    /// Send a canvas event
    pub fn send_canvas_event(&self, event: CanvasEvent) -> Result<usize, broadcast::error::SendError<CanvasEvent>> {
        self.canvas_events.send(event)
    }
    
    /// Send a node event
    pub fn send_node_event(&self, event: NodeEvent) -> Result<usize, broadcast::error::SendError<NodeEvent>> {
        self.node_events.send(event)
    }
    
    /// Send a window event
    pub fn send_window_event(&self, event: WindowEvent) -> Result<usize, broadcast::error::SendError<WindowEvent>> {
        self.window_events.send(event)
    }
}

/// Canvas-specific events
#[derive(Debug, Clone)]
pub enum CanvasEvent {
    /// Node was moved on the canvas
    NodeMoved { 
        node_id: NodeId, 
        old_pos: Point2, 
        new_pos: Point2 
    },
    
    /// Node was selected
    NodeSelected { 
        node_id: NodeId,
        multi_select: bool,
    },
    
    /// Node was deselected
    NodeDeselected { 
        node_id: NodeId 
    },
    
    /// Connection was created between nodes
    ConnectionCreated { 
        from: (NodeId, String),
        to: (NodeId, String),
    },
    
    /// Connection was removed
    ConnectionRemoved { 
        from: (NodeId, String),
        to: (NodeId, String),
    },
    
    /// Viewport changed (pan/zoom)
    ViewportChanged { 
        center: Point2, 
        zoom: f32 
    },
    
    /// Selection box changed
    SelectionChanged {
        selected_nodes: Vec<NodeId>,
    },
    
    /// Request to center view on nodes
    CenterOnNodes {
        node_ids: Vec<NodeId>,
    },
}

/// Node-specific events
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// Query was linked to another node
    QueryLinked {
        query_node: NodeId,
        target_node: NodeId,
    },
    
    /// Plot was refreshed with new data
    PlotRefreshed {
        plot_node: NodeId,
        row_count: usize,
    },
    
    /// Dataset was loaded
    DatasetLoaded {
        node_id: NodeId,
        table_name: String,
        row_count: usize,
    },
    
    /// Dataset was unloaded
    DatasetUnloaded {
        node_id: NodeId,
    },
    
    /// Node execution started
    NodeExecutionStarted {
        node_id: NodeId,
    },
    
    /// Node execution completed
    NodeExecutionCompleted {
        node_id: NodeId,
        success: bool,
        execution_time: std::time::Duration,
    },
    
    /// Node configuration changed
    NodeConfigChanged {
        node_id: NodeId,
        config_type: String,
    },
}

/// Window management events
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Window was moved
    WindowMoved {
        window_id: WindowId,
        new_position: Point2,
    },
    
    /// Window was resized
    WindowResized {
        window_id: WindowId,
        new_size: Point2,
    },
    
    /// Window was closed
    WindowClosed {
        window_id: WindowId,
    },
    
    /// Window was focused
    WindowFocused {
        window_id: WindowId,
    },
    
    /// New floating window created
    FloatingWindowCreated {
        window_id: WindowId,
        node_id: NodeId,
        position: Point2,
    },
    
    /// Window docked to main view
    WindowDocked {
        window_id: WindowId,
        dock_position: DockPosition,
    },
}

/// Window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WindowId(pub uuid::Uuid);

impl WindowId {
    pub fn new() -> Self {
        WindowId(uuid::Uuid::new_v4())
    }
}

/// Dock positions for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPosition {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

#[cfg(test)]
mod event_bus_tests {
    use super::*;
    
    #[test]
    fn test_event_bus_creation() {
        let bus = EventBus::new();
        
        // Test subscribing to different event types
        let _app_rx = bus.subscribe_app_events();
        let _canvas_rx = bus.subscribe_canvas_events();
        let _node_rx = bus.subscribe_node_events();
        let _window_rx = bus.subscribe_window_events();
    }
    
    #[tokio::test]
    async fn test_event_sending() {
        let bus = EventBus::new();
        let mut canvas_rx = bus.subscribe_canvas_events();
        
        // Send a canvas event
        let event = CanvasEvent::NodeSelected {
            node_id: NodeId::new(),
            multi_select: false,
        };
        
        bus.send_canvas_event(event.clone()).unwrap();
        
        // Receive the event
        match canvas_rx.recv().await {
            Ok(received) => {
                match (received, event) {
                    (CanvasEvent::NodeSelected { node_id: id1, .. }, 
                     CanvasEvent::NodeSelected { node_id: id2, .. }) => {
                        assert_eq!(id1, id2);
                    }
                    _ => panic!("Event mismatch"),
                }
            }
            Err(_) => panic!("Failed to receive event"),
        }
    }
    
    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new();
        
        // Create multiple subscribers
        let mut rx1 = bus.subscribe_app_events();
        let mut rx2 = bus.subscribe_app_events();
        
        // Send an event
        let event = AppEvent::EngineReady;
        bus.send_app_event(event).unwrap();
        
        // Both should receive it
        assert!(matches!(rx1.try_recv(), Ok(AppEvent::EngineReady)));
        assert!(matches!(rx2.try_recv(), Ok(AppEvent::EngineReady)));
    }
} 