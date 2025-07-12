//! Pika-Plot engine for data processing and GPU acceleration.

pub mod aggregation;
pub mod cache;
pub mod database;
pub mod gpu;
pub mod import;
pub mod memory_coordinator;
pub mod query;
pub mod streaming;
pub mod workspace;
pub mod plot;

use pika_core::{
    events::{AppEvent, EventBus, PlotRenderData},
    types::{NodeId, TableInfo, ImportOptions, QueryResult},
    plots::PlotConfig,
    error::{Result, PikaError},
    snapshot::WorkspaceSnapshot,
};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::runtime::Handle;
use crate::memory_coordinator::MemoryCoordinator;

/// Main compute engine that coordinates data processing.
pub struct Engine {
    /// Event bus for UI communication
    event_bus: EventBus,
    
    /// Database connection
    database: Arc<database::Database>,
    
    /// GPU manager
    gpu_manager: Option<Arc<gpu::GpuManager>>,
    
    /// Cache manager
    cache: Arc<cache::QueryCache>,
    
    /// Memory coordinator
    memory_coordinator: Arc<MemoryCoordinator>,
    
    /// Runtime handle for spawning tasks
    runtime: Handle,
    
    /// Command channel for internal operations
    command_rx: mpsc::Receiver<EngineCommand>,
    command_tx: mpsc::Sender<EngineCommand>,
}

/// Commands that can be sent to the engine
enum EngineCommand {
    ImportFile {
        path: std::path::PathBuf,
        options: ImportOptions,
        response: oneshot::Sender<Result<TableInfo>>,
    },
    ExecuteQuery {
        sql: String,
        response: oneshot::Sender<Result<QueryResult>>,
    },
    RenderPlot {
        config: PlotConfig,
        query_result: QueryResult,
        response: oneshot::Sender<Result<PlotRenderData>>,
    },
    Shutdown,
    SaveSnapshot(oneshot::Sender<Result<WorkspaceSnapshot>>),
    RestoreSnapshot(WorkspaceSnapshot),
}

impl Engine {
    /// Create a new engine instance.
    pub async fn new(memory_limit: Option<u64>, runtime: Handle) -> Result<Self> {
        let event_bus = EventBus::new();
        let (command_tx, command_rx) = mpsc::channel(256);
        
        // Get system memory for coordinator
        // TODO: Add sys-info crate for actual memory detection
        let total_ram = 8 * 1024 * 1024 * 1024; // Default 8GB for now
        
        let memory_coordinator = Arc::new(MemoryCoordinator::new(total_ram));
        
        // Create database with configured memory limit
        let database = Arc::new(database::Database::new(memory_limit).await?);
        
        // Configure DuckDB memory limit
        memory_coordinator.configure_duckdb(database.connection())
            .map_err(|e| PikaError::Other(e.to_string()))?;
        
        // Create cache with memory coordinator
        let cache_limit = memory_limit.unwrap_or(1024 * 1024 * 1024);
        let cache = Arc::new(cache::QueryCache::new_with_limit(cache_limit));
        
        // Try to initialize GPU
        let gpu_manager = match gpu::GpuManager::new().await {
            Ok(gpu) => Some(Arc::new(gpu)),
            Err(e) => {
                tracing::warn!("GPU initialization failed: {}", e);
                None
            }
        };
        
        Ok(Self {
            event_bus,
            database,
            gpu_manager,
            cache,
            memory_coordinator,
            runtime,
            command_rx,
            command_tx,
        })
    }
    
    /// Process engine events and commands.
    pub async fn process_events(&mut self) -> Result<()> {
        // Check for commands
        match self.command_rx.try_recv() {
            Ok(cmd) => match cmd {
                EngineCommand::ImportFile { path, options, response } => {
                    // Send import started event
                    let _ = self.event_bus.send_app_event(AppEvent::ImportStarted { 
                        path: path.clone() 
                    });
                    
                    let result = import::import_file(&self.database, &path, options, Some(self.event_bus.clone())).await;
                    match result {
                        Ok(table_info) => {
                            let _ = self.event_bus.send_app_event(AppEvent::ImportComplete { 
                                path,
                                table_info: table_info.clone(),
                            });
                            let _ = response.send(Ok(table_info));
                        }
                        Err(e) => {
                            let _ = self.event_bus.send_app_event(AppEvent::ImportError { 
                                path,
                                error: e.to_string(),
                            });
                            let _ = response.send(Err(e));
                        }
                    }
                }
                EngineCommand::ExecuteQuery { sql, response } => {
                    let _ = self.event_bus.send_app_event(AppEvent::QueryStarted { 
                        id: NodeId::default() // Placeholder, actual ID would be generated
                    });
                    
                    let result = query::execute(&self.database, &sql).await;
                    match result {
                        Ok(query_result) => {
                            let _ = self.event_bus.send_app_event(AppEvent::QueryComplete { 
                                id: NodeId::default(), // Placeholder
                                result: Ok(query_result.clone()),
                            });
                            let _ = response.send(Ok(query_result));
                        }
                        Err(e) => {
                            let _ = self.event_bus.send_app_event(AppEvent::QueryComplete { 
                                id: NodeId::default(), // Placeholder
                                result: Err(e.to_string()),
                            });
                            let _ = response.send(Err(e));
                        }
                    }
                }
                EngineCommand::RenderPlot { config, query_result, response } => {
                    if let Some(gpu) = &self.gpu_manager {
                        let renderer = plot::renderer::PlotRenderer::new(gpu.clone());
                        match renderer.prepare_plot_data(&config, &query_result) {
                            Ok(plot_data) => {
                                let _ = response.send(Ok(plot_data));
                            }
                            Err(e) => {
                                let _ = response.send(Err(e));
                            }
                        }
                    } else {
                        let _ = response.send(Err(PikaError::Other("GPU not available".to_string())));
                    }
                }
                EngineCommand::SaveSnapshot(response) => {
                    let snapshot = workspace::create_snapshot(&self.database).await?;
                    let _ = response.send(Ok(snapshot));
                }
                EngineCommand::RestoreSnapshot(snapshot) => {
                    // Can't clone Database, so we need a different approach
                    // For now, just log that we would restore
                    tracing::info!("Would restore snapshot (not implemented)");
                    // TODO: Implement proper snapshot restoration
                }
                EngineCommand::Shutdown => {
                    tracing::info!("Engine received shutdown command.");
                    // No explicit action needed here, the runtime will exit.
                }
            },
            Err(mpsc::error::TryRecvError::Empty) => {},
            Err(e) => return Err(PikaError::Internal(format!("Command channel error: {}", e))),
        }
        
        // Check memory pressure periodically
        self.memory_coordinator.update_memory_pressure();
        
        Ok(())
    }
    
    /// Get the event bus.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    
    /// Get the memory coordinator.
    pub fn memory_coordinator(&self) -> &Arc<MemoryCoordinator> {
        &self.memory_coordinator
    }
    
    /// Import data from a file.
    pub async fn import_file(
        &self,
        path: &std::path::Path,
        options: ImportOptions,
    ) -> Result<TableInfo> {
        let (tx, rx) = oneshot::channel();
        let _ = self.command_tx.send(EngineCommand::ImportFile { 
            path: path.to_path_buf(), 
            options, 
            response: tx 
        }).await;
        rx.await.map_err(|_| PikaError::Other("Failed to receive response".to_string()))?
    }
    
    /// Execute a query and return results.
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult> {
        let (tx, rx) = oneshot::channel();
        let _ = self.command_tx.send(EngineCommand::ExecuteQuery { 
            sql: sql.to_string(), 
            response: tx 
        }).await;
        rx.await.map_err(|_| PikaError::Other("Failed to receive response".to_string()))?
    }
    
    /// Save current state as a snapshot.
    pub async fn save_snapshot(&self) -> Result<WorkspaceSnapshot> {
        let (tx, rx) = oneshot::channel();
        let _ = self.command_tx.send(EngineCommand::SaveSnapshot(tx)).await;
        rx.await.map_err(|_| PikaError::Other("Failed to receive response".to_string()))?
    }
    
    /// Load state from a snapshot.
    pub async fn load_snapshot(&mut self, path: &std::path::Path) -> Result<()> {
        let snapshot = workspace::load_snapshot(path)?;
        let _ = self.command_tx.send(EngineCommand::RestoreSnapshot(snapshot)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_creation() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let engine = Engine::new(Some(1024 * 1024 * 1024), runtime.handle().clone()).await;
        assert!(engine.is_ok());
    }
    
    #[tokio::test] 
    async fn test_event_bus() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let engine = Engine::new(None, runtime.handle().clone()).await.unwrap();
        let mut rx = engine.event_bus().subscribe_app_events();
        
        // Send a test event
        engine.event_bus().send_app_event(AppEvent::EngineReady).unwrap();
        
        // Receive it
        match rx.recv().await {
            Ok(AppEvent::EngineReady) => {},
            _ => panic!("Wrong event type"),
        }
    }
}
