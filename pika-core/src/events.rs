//! Event system for UI and engine communication.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::types::{NodeId, QueryResult};
use crate::plots::PlotConfig;

/// Main event types in the system
#[derive(Debug, Clone)]
pub enum Event {
    /// Application-level events
    App(AppEvent),
    
    /// Canvas interaction events
    Canvas(CanvasEvent),
    
    /// Node-specific events
    Node(NodeEvent),
    
    /// Query execution events
    Query(QueryEvent),
    
    /// Plot rendering events
    Plot(PlotEvent),
    
    /// Window management events
    Window(WindowEvent),
}

/// Query execution events
#[derive(Debug, Clone)]
pub enum QueryEvent {
    /// Query started execution
    Started {
        node_id: NodeId,
        sql: String,
        cache_key: Option<String>,
    },
    
    /// Query completed successfully
    Completed {
        node_id: NodeId,
        result: Arc<QueryResult>,
        cached: bool,
    },
    
    /// Query failed
    Failed {
        node_id: NodeId,
        error: String,
    },
    
    /// Query was cancelled
    Cancelled {
        node_id: NodeId,
    },
}

/// Application-level events
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Application started
    Started,
    
    /// File opened
    FileOpened(String),
    
    /// File saved
    FileSaved(String),
    
    /// Memory warning
    MemoryWarning {
        used: u64,
        available: u64,
    },
    
    /// Settings changed
    SettingsChanged,
}

/// Canvas interaction events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanvasEvent {
    /// Node was moved
    NodeMoved {
        node_id: NodeId,
        new_position: (f32, f32),
    },
    
    /// Node was selected
    NodeSelected {
        node_id: NodeId,
        multi_select: bool,
    },
    
    /// Node was deleted
    NodeDeleted {
        node_id: NodeId,
    },
    
    /// Connection created
    ConnectionCreated {
        from_node: NodeId,
        from_port: String,
        to_node: NodeId,
        to_port: String,
    },
    
    /// Connection deleted
    ConnectionDeleted {
        from_node: NodeId,
        to_node: NodeId,
    },
    
    /// Canvas panned
    CanvasPanned {
        delta: (f32, f32),
    },
    
    /// Canvas zoomed
    CanvasZoomed {
        zoom_delta: f32,
        focus_point: (f32, f32),
    },
}

/// Node-specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeEvent {
    /// Node created
    Created {
        node_id: NodeId,
        node_type: String,
    },
    
    /// Node updated
    Updated {
        node_id: NodeId,
    },
    
    /// Node execution started
    ExecutionStarted {
        node_id: NodeId,
    },
    
    /// Node execution completed
    ExecutionCompleted {
        node_id: NodeId,
    },
    
    /// Node execution failed
    ExecutionFailed {
        node_id: NodeId,
        error: String,
    },
}

/// Plot rendering events
#[derive(Debug, Clone)]
pub enum PlotEvent {
    /// Plot render requested
    RenderRequested {
        node_id: NodeId,
        config: PlotConfig,
    },
    
    /// Plot rendered successfully
    Rendered {
        node_id: NodeId,
        render_time_ms: u64,
    },
    
    /// Plot render failed
    RenderFailed {
        node_id: NodeId,
        error: String,
    },
}

/// Window management events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowEvent {
    /// Window opened
    Opened {
        window_id: String,
        window_type: String,
    },
    
    /// Window closed
    Closed {
        window_id: String,
    },
    
    /// Window focused
    Focused {
        window_id: String,
    },
    
    /// Window resized
    Resized {
        window_id: String,
        new_size: (f32, f32),
    },
}

/// Event bus for broadcasting events
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        EventBus { sender }
    }
    
    /// Send an event
    pub fn send(&self, event: Event) {
        // Ignore send errors (no receivers)
        let _ = self.sender.send(event);
    }
    
    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Data passed to plot rendering callbacks
#[derive(Debug, Clone)]
pub struct PlotRenderData {
    pub data: Arc<duckdb::arrow::record_batch::RecordBatch>,
    pub config: PlotConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_bus() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe();
        
        let event = Event::App(AppEvent::Started);
        bus.send(event.clone());
        
        let received = rx.try_recv().unwrap();
        match received {
            Event::App(AppEvent::Started) => {},
            _ => panic!("Wrong event received"),
        }
    }
} 