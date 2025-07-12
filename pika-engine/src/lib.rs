//! Main engine module that coordinates data processing.

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use pika_core::{
    error::{PikaError, Result},
    events::{Event, EventBus, QueryEvent, AppEvent},
    types::{NodeId, QueryResult, TableInfo},
    plots::PlotConfig,
};

pub mod aggregation;
pub mod cache;
pub mod database;
pub mod gpu;
pub mod import;
pub mod memory;
pub mod plot;
pub mod query;
pub mod streaming;
pub mod workspace;
pub mod enhanced_csv;

// Re-exports
pub use database::Database;
pub use import::*;
pub use memory::*;
pub use plot::*;
pub use query::*;
pub use streaming::*;
pub use workspace::*;
pub use enhanced_csv::*;

/// Commands that can be sent to the engine
enum EngineCommand {
    ImportCsv {
        path: std::path::PathBuf,
        options: pika_core::types::ImportOptions,
        node_id: NodeId,
        reply: oneshot::Sender<Result<TableInfo>>,
    },
    ExecuteQuery {
        sql: String,
        node_id: NodeId,
        reply: oneshot::Sender<Result<QueryResult>>,
    },
    PreparePlot {
        query_result: Arc<QueryResult>,
        config: PlotConfig,
        reply: oneshot::Sender<Result<pika_core::events::PlotRenderData>>,
    },
    Shutdown,
}

/// Main engine struct that coordinates all data processing
pub struct Engine {
    command_tx: mpsc::Sender<EngineCommand>,
    event_bus: Arc<EventBus>,
    _handle: tokio::task::JoinHandle<()>,
}

impl Engine {
    /// Create a new engine instance
    pub async fn new(event_bus: Arc<EventBus>) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(100);
        let event_bus_clone = event_bus.clone();
        
        let handle = tokio::spawn(async move {
            let mut engine = EngineWorker::new(event_bus_clone).await;
            engine.run(command_rx).await;
        });
        
        Ok(Engine {
            command_tx,
            event_bus,
            _handle: handle,
        })
    }
    
    /// Create a new engine with default event bus
    pub async fn new_default() -> Result<Self> {
        let event_bus = Arc::new(EventBus::new(1024));
        Self::new(event_bus).await
    }
    
    /// Get the event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }
    
    /// Import a CSV file
    pub async fn import_csv(
        &self,
        path: std::path::PathBuf,
        options: pika_core::types::ImportOptions,
        node_id: NodeId,
    ) -> Result<TableInfo> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.command_tx.send(EngineCommand::ImportCsv {
            path,
            options,
            node_id,
            reply: reply_tx,
        }).await.map_err(|_| PikaError::internal("Engine channel closed"))?;
        
        reply_rx.await.map_err(|_| PikaError::internal("Failed to receive response"))?
    }
    
    /// Execute a SQL query
    pub async fn execute_query(&self, sql: String, node_id: NodeId) -> Result<QueryResult> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.command_tx.send(EngineCommand::ExecuteQuery {
            sql,
            node_id,
            reply: reply_tx,
        }).await.map_err(|_| PikaError::internal("Engine channel closed"))?;
        
        reply_rx.await.map_err(|_| PikaError::internal("Failed to receive response"))?
    }
    
    /// Prepare plot data for rendering
    pub async fn prepare_plot(
        &self,
        query_result: Arc<QueryResult>,
        config: PlotConfig,
    ) -> Result<pika_core::events::PlotRenderData> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.command_tx.send(EngineCommand::PreparePlot {
            query_result,
            config,
            reply: reply_tx,
        }).await.map_err(|_| PikaError::internal("Engine channel closed"))?;
        
        reply_rx.await.map_err(|_| PikaError::internal("Failed to receive response"))?
    }
    
    /// Shutdown the engine
    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx.send(EngineCommand::Shutdown)
            .await
            .map_err(|_| PikaError::internal("Engine channel closed"))?;
        Ok(())
    }
}

/// Internal engine worker that processes commands
struct EngineWorker {
    database: Arc<Mutex<Database>>,
    memory_coordinator: Arc<MemoryCoordinator>,
    query_engine: Arc<QueryEngine>,
    plot_renderer: Arc<PlotRenderer>,
    event_bus: Arc<EventBus>,
}

impl EngineWorker {
    async fn new(event_bus: Arc<EventBus>) -> Self {
        let database = Arc::new(Mutex::new(Database::new().await.unwrap()));
        let memory_coordinator = Arc::new(MemoryCoordinator::new(None));
        let query_engine = Arc::new(QueryEngine::new(database.clone()));
        
        // Initialize GPU manager
        let gpu_manager = match gpu::GpuManager::new().await {
            Ok(manager) => Some(Arc::new(manager)),
            Err(e) => {
                tracing::warn!("Failed to initialize GPU: {}", e);
                None
            }
        };
        
        let plot_renderer = Arc::new(PlotRenderer::new(gpu_manager));
        
        EngineWorker {
            database,
            memory_coordinator,
            query_engine,
            plot_renderer,
            event_bus,
        }
    }
    
    async fn run(&mut self, mut command_rx: mpsc::Receiver<EngineCommand>) {
        while let Some(command) = command_rx.recv().await {
            match command {
                EngineCommand::ImportCsv { path, options, node_id, reply } => {
                    // Send import started event
                    self.event_bus.send(Event::App(AppEvent::FileOpened(
                        path.display().to_string()
                    )));
                    
                    let result = import::import_csv(
                        &path, 
                        &node_id, 
                        &options, 
                        Some(self.event_bus.clone())
                    ).await;
                    
                    match &result {
                        Ok(table_info) => {
                            // Store table info
                            tracing::info!("Imported table: {:?}", table_info);
                        }
                        Err(e) => {
                            tracing::error!("Import failed: {}", e);
                        }
                    }
                    
                    let _ = reply.send(result);
                }
                
                EngineCommand::ExecuteQuery { sql, node_id, reply } => {
                    self.event_bus.send(Event::Query(QueryEvent::Started {
                        node_id,
                        sql: sql.clone(),
                        cache_key: None,
                    }));
                    
                    let result = self.query_engine.execute(&sql).await;
                    
                    match &result {
                        Ok(query_result) => {
                            self.event_bus.send(Event::Query(QueryEvent::Completed {
                                node_id,
                                result: Arc::new(query_result.clone()),
                                cached: false,
                            }));
                        }
                        Err(e) => {
                            self.event_bus.send(Event::Query(QueryEvent::Failed {
                                node_id,
                                error: e.to_string(),
                            }));
                        }
                    }
                    
                    let _ = reply.send(result);
                }
                
                EngineCommand::PreparePlot { query_result, config, reply } => {
                    // For now, just create a simple PlotRenderData
                    // In a real implementation, this would extract data from query_result
                    let plot_data = pika_core::events::PlotRenderData {
                        data: Arc::new(duckdb::arrow::record_batch::RecordBatch::new_empty(
                            Arc::new(duckdb::arrow::datatypes::Schema::empty())
                        )),
                        config,
                    };
                    
                    let _ = reply.send(Ok(plot_data));
                }
                
                EngineCommand::Shutdown => {
                    tracing::info!("Engine shutting down");
                    break;
                }
            }
        }
    }
}

/// Get the available memory in bytes
pub fn get_available_memory() -> u64 {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    sys.available_memory()
}

/// Get the total memory in bytes
pub fn get_total_memory() -> u64 {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    sys.total_memory()
}
