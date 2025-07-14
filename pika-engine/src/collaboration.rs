use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock as AsyncRwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// CRDT imports for conflict-free replication
use y_octo::{Doc, Text, Array, Map, Transaction};
use loro::{LoroDoc, LoroText, LoroList, LoroMap};
use lww_table::LwwDb;

/// Real-time collaboration engine with CRDT-based synchronization
pub struct CollaborationEngine {
    // Y-Octo CRDT document for rich text and complex data structures
    y_doc: Arc<RwLock<Doc>>,
    
    // Loro CRDT document for high-performance collaboration
    loro_doc: Arc<RwLock<LoroDoc>>,
    
    // LWW (Last-Write-Wins) table for simple key-value data
    lww_db: Arc<RwLock<LwwDb>>,
    
    // Active collaboration sessions
    sessions: Arc<RwLock<HashMap<String, CollaborationSession>>>,
    
    // Real-time synchronization channels
    sync_channels: SyncChannels,
    
    // Conflict resolution strategies
    conflict_resolver: ConflictResolver,
    
    // Operational transform engine
    ot_engine: OperationalTransformEngine,
    
    // Live cursor tracking
    cursor_tracker: CursorTracker,
    
    // Presence awareness
    presence_manager: PresenceManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub session_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub color: (u8, u8, u8),
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: UserPermissions,
    pub cursor_position: Option<CursorPosition>,
    pub selection: Option<Selection>,
    pub current_tool: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub can_edit: bool,
    pub can_comment: bool,
    pub can_share: bool,
    pub can_export: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
    pub canvas_id: String,
    pub element_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
    pub selected_elements: Vec<String>,
    pub selection_type: SelectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType {
    Text,
    Shape,
    Multiple,
    Region,
}

pub struct SyncChannels {
    // Broadcast channel for real-time updates
    update_sender: broadcast::Sender<CollaborationUpdate>,
    update_receiver: broadcast::Receiver<CollaborationUpdate>,
    
    // Channel for cursor movements
    cursor_sender: broadcast::Sender<CursorUpdate>,
    cursor_receiver: broadcast::Receiver<CursorUpdate>,
    
    // Channel for presence updates
    presence_sender: broadcast::Sender<PresenceUpdate>,
    presence_receiver: broadcast::Receiver<PresenceUpdate>,
    
    // Channel for conflict resolution
    conflict_sender: mpsc::Sender<ConflictEvent>,
    conflict_receiver: mpsc::Receiver<ConflictEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationUpdate {
    pub update_id: String,
    pub session_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub operation: Operation,
    pub vector_clock: VectorClock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    // Text operations
    TextInsert { position: usize, text: String },
    TextDelete { position: usize, length: usize },
    TextFormat { range: (usize, usize), format: TextFormat },
    
    // Shape operations
    ShapeCreate { shape: Shape },
    ShapeUpdate { shape_id: String, properties: ShapeProperties },
    ShapeDelete { shape_id: String },
    ShapeMove { shape_id: String, position: (f64, f64) },
    
    // Canvas operations
    CanvasCreate { canvas: Canvas },
    CanvasUpdate { canvas_id: String, properties: CanvasProperties },
    CanvasDelete { canvas_id: String },
    
    // Data operations
    DataInsert { table: String, row: String, column: String, value: serde_json::Value },
    DataUpdate { table: String, row: String, column: String, value: serde_json::Value },
    DataDelete { table: String, row: String, column: Option<String> },
    
    // Plot operations
    PlotCreate { plot: Plot },
    PlotUpdate { plot_id: String, properties: PlotProperties },
    PlotDelete { plot_id: String },
    
    // Annotation operations
    AnnotationCreate { annotation: Annotation },
    AnnotationUpdate { annotation_id: String, properties: AnnotationProperties },
    AnnotationDelete { annotation_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    pub clocks: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormat {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub color: Option<String>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub id: String,
    pub shape_type: ShapeType,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub rotation: f64,
    pub properties: ShapeProperties,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
    Arrow,
    Text,
    Path,
    Image,
    Chart,
    Table,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeProperties {
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub visible: bool,
    pub locked: bool,
    pub z_index: i32,
    pub custom_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: String,
    pub name: String,
    pub size: (f64, f64),
    pub background_color: String,
    pub grid_enabled: bool,
    pub grid_size: f64,
    pub zoom_level: f64,
    pub pan_offset: (f64, f64),
    pub properties: CanvasProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasProperties {
    pub infinite_canvas: bool,
    pub snap_to_grid: bool,
    pub show_rulers: bool,
    pub show_guides: bool,
    pub collaboration_enabled: bool,
    pub real_time_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    pub id: String,
    pub plot_type: String,
    pub data_source: String,
    pub position: (f64, f64),
    pub size: (f64, f64),
    pub properties: PlotProperties,
    pub interactive_state: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotProperties {
    pub title: String,
    pub theme: String,
    pub animation_enabled: bool,
    pub interaction_enabled: bool,
    pub export_formats: Vec<String>,
    pub real_time_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub annotation_type: AnnotationType,
    pub position: (f64, f64),
    pub content: String,
    pub properties: AnnotationProperties,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Comment,
    Highlight,
    Arrow,
    Callout,
    Sticky,
    Voice,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationProperties {
    pub color: String,
    pub size: f64,
    pub visible: bool,
    pub resolved: bool,
    pub replies: Vec<AnnotationReply>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationReply {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorUpdate {
    pub session_id: String,
    pub user_id: String,
    pub position: CursorPosition,
    pub tool: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub session_id: String,
    pub user_id: String,
    pub status: PresenceStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

pub struct ConflictResolver {
    strategies: HashMap<String, ConflictResolutionStrategy>,
}

#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    LastWriteWins,
    FirstWriteWins,
    Merge,
    UserChoice,
    Automatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEvent {
    pub conflict_id: String,
    pub operation1: Operation,
    pub operation2: Operation,
    pub timestamp: DateTime<Utc>,
    pub resolution_strategy: ConflictResolutionStrategy,
}

pub struct OperationalTransformEngine {
    pending_operations: Arc<RwLock<Vec<Operation>>>,
    applied_operations: Arc<RwLock<Vec<Operation>>>,
    transform_functions: HashMap<String, TransformFunction>,
}

pub type TransformFunction = Box<dyn Fn(&Operation, &Operation) -> Result<(Operation, Operation)> + Send + Sync>;

pub struct CursorTracker {
    active_cursors: Arc<RwLock<HashMap<String, CursorState>>>,
    cursor_colors: HashMap<String, (u8, u8, u8)>,
    cursor_update_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct CursorState {
    pub position: CursorPosition,
    pub tool: String,
    pub selection: Option<Selection>,
    pub last_update: Instant,
    pub is_active: bool,
}

pub struct PresenceManager {
    user_presence: Arc<RwLock<HashMap<String, PresenceInfo>>>,
    presence_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub status: PresenceStatus,
    pub last_seen: DateTime<Utc>,
    pub current_canvas: Option<String>,
    pub active_tool: Option<String>,
}

impl CollaborationEngine {
    pub fn new() -> Result<Self> {
        let (update_sender, update_receiver) = broadcast::channel(1000);
        let (cursor_sender, cursor_receiver) = broadcast::channel(1000);
        let (presence_sender, presence_receiver) = broadcast::channel(100);
        let (conflict_sender, conflict_receiver) = mpsc::channel(100);

        let sync_channels = SyncChannels {
            update_sender,
            update_receiver,
            cursor_sender,
            cursor_receiver,
            presence_sender,
            presence_receiver,
            conflict_sender,
            conflict_receiver,
        };

        let mut conflict_strategies = HashMap::new();
        conflict_strategies.insert("default".to_string(), ConflictResolutionStrategy::LastWriteWins);
        conflict_strategies.insert("text".to_string(), ConflictResolutionStrategy::Merge);
        conflict_strategies.insert("shape".to_string(), ConflictResolutionStrategy::LastWriteWins);

        let conflict_resolver = ConflictResolver {
            strategies: conflict_strategies,
        };

        let ot_engine = OperationalTransformEngine {
            pending_operations: Arc::new(RwLock::new(Vec::new())),
            applied_operations: Arc::new(RwLock::new(Vec::new())),
            transform_functions: HashMap::new(),
        };

        let cursor_tracker = CursorTracker {
            active_cursors: Arc::new(RwLock::new(HashMap::new())),
            cursor_colors: HashMap::new(),
            cursor_update_interval: Duration::from_millis(50),
        };

        let presence_manager = PresenceManager {
            user_presence: Arc::new(RwLock::new(HashMap::new())),
            presence_timeout: Duration::from_secs(30),
        };

        Ok(Self {
            y_doc: Arc::new(RwLock::new(Doc::new())),
            loro_doc: Arc::new(RwLock::new(LoroDoc::new())),
            lww_db: Arc::new(RwLock::new(LwwDb::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            sync_channels,
            conflict_resolver,
            ot_engine,
            cursor_tracker,
            presence_manager,
        })
    }

    /// Start a new collaboration session
    pub async fn start_session(&self, user_id: String, user_name: String) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let color = self.generate_user_color(&user_id);
        
        let session = CollaborationSession {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            user_name,
            user_avatar: None,
            color,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            permissions: UserPermissions {
                can_edit: true,
                can_comment: true,
                can_share: false,
                can_export: true,
                is_admin: false,
            },
            cursor_position: None,
            selection: None,
            current_tool: "select".to_string(),
            is_active: true,
        };

        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.clone(), session);
        }

        // Update presence
        self.update_presence(user_id, PresenceStatus::Online).await?;

        // Broadcast session start
        let presence_update = PresenceUpdate {
            session_id: session_id.clone(),
            user_id,
            status: PresenceStatus::Online,
            timestamp: Utc::now(),
        };

        let _ = self.sync_channels.presence_sender.send(presence_update);

        Ok(session_id)
    }

    /// End a collaboration session
    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let user_id = {
            let mut sessions = self.sessions.write().unwrap();
            if let Some(session) = sessions.remove(session_id) {
                session.user_id
            } else {
                return Err(anyhow::anyhow!("Session not found"));
            }
        };

        // Update presence to offline
        self.update_presence(user_id.clone(), PresenceStatus::Offline).await?;

        // Remove cursor
        {
            let mut cursors = self.cursor_tracker.active_cursors.write().unwrap();
            cursors.remove(session_id);
        }

        // Broadcast session end
        let presence_update = PresenceUpdate {
            session_id: session_id.to_string(),
            user_id,
            status: PresenceStatus::Offline,
            timestamp: Utc::now(),
        };

        let _ = self.sync_channels.presence_sender.send(presence_update);

        Ok(())
    }

    /// Apply an operation using CRDT
    pub async fn apply_operation(&self, session_id: &str, operation: Operation) -> Result<()> {
        let update_id = Uuid::new_v4().to_string();
        let user_id = {
            let sessions = self.sessions.read().unwrap();
            let session = sessions.get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.user_id.clone()
        };

        // Apply operation based on type
        match &operation {
            Operation::TextInsert { position, text } => {
                self.apply_text_insert(*position, text).await?;
            },
            Operation::TextDelete { position, length } => {
                self.apply_text_delete(*position, *length).await?;
            },
            Operation::ShapeCreate { shape } => {
                self.apply_shape_create(shape).await?;
            },
            Operation::ShapeUpdate { shape_id, properties } => {
                self.apply_shape_update(shape_id, properties).await?;
            },
            Operation::ShapeDelete { shape_id } => {
                self.apply_shape_delete(shape_id).await?;
            },
            Operation::DataInsert { table, row, column, value } => {
                self.apply_data_insert(table, row, column, value).await?;
            },
            Operation::DataUpdate { table, row, column, value } => {
                self.apply_data_update(table, row, column, value).await?;
            },
            Operation::DataDelete { table, row, column } => {
                self.apply_data_delete(table, row, column).await?;
            },
            _ => {
                // Handle other operation types
            }
        }

        // Create collaboration update
        let update = CollaborationUpdate {
            update_id,
            session_id: session_id.to_string(),
            user_id,
            timestamp: Utc::now(),
            operation,
            vector_clock: self.get_vector_clock().await,
        };

        // Broadcast update
        let _ = self.sync_channels.update_sender.send(update);

        Ok(())
    }

    /// Apply text insert operation using Y-Octo
    async fn apply_text_insert(&self, position: usize, text: &str) -> Result<()> {
        let mut doc = self.y_doc.write().unwrap();
        let mut txn = doc.transact_mut();
        
        // Get or create text object
        let text_obj = txn.get_or_insert_text("main_text");
        text_obj.insert(&mut txn, position, text);
        
        Ok(())
    }

    /// Apply text delete operation using Y-Octo
    async fn apply_text_delete(&self, position: usize, length: usize) -> Result<()> {
        let mut doc = self.y_doc.write().unwrap();
        let mut txn = doc.transact_mut();
        
        let text_obj = txn.get_or_insert_text("main_text");
        text_obj.remove_range(&mut txn, position, length);
        
        Ok(())
    }

    /// Apply shape create operation using Loro
    async fn apply_shape_create(&self, shape: &Shape) -> Result<()> {
        let mut doc = self.loro_doc.write().unwrap();
        let shapes_map = doc.get_map("shapes");
        
        let shape_data = serde_json::to_string(shape)?;
        shapes_map.insert(&shape.id, shape_data)?;
        
        Ok(())
    }

    /// Apply shape update operation using Loro
    async fn apply_shape_update(&self, shape_id: &str, properties: &ShapeProperties) -> Result<()> {
        let mut doc = self.loro_doc.write().unwrap();
        let shapes_map = doc.get_map("shapes");
        
        // Get existing shape and update properties
        if let Some(existing_shape_data) = shapes_map.get(shape_id) {
            let mut shape: Shape = serde_json::from_str(&existing_shape_data.to_string())?;
            shape.properties = properties.clone();
            
            let updated_data = serde_json::to_string(&shape)?;
            shapes_map.insert(shape_id, updated_data)?;
        }
        
        Ok(())
    }

    /// Apply shape delete operation using Loro
    async fn apply_shape_delete(&self, shape_id: &str) -> Result<()> {
        let mut doc = self.loro_doc.write().unwrap();
        let shapes_map = doc.get_map("shapes");
        shapes_map.delete(shape_id)?;
        
        Ok(())
    }

    /// Apply data insert operation using LWW table
    async fn apply_data_insert(&self, table: &str, row: &str, column: &str, value: &serde_json::Value) -> Result<()> {
        let mut db = self.lww_db.write().unwrap();
        
        // Convert JSON value to appropriate type
        match value {
            serde_json::Value::String(s) => db.set(table, row, column, s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    db.set(table, row, column, i);
                } else if let Some(f) = n.as_f64() {
                    db.set(table, row, column, f);
                }
            },
            serde_json::Value::Bool(b) => db.set(table, row, column, *b),
            _ => db.set(table, row, column, value.to_string()),
        }
        
        Ok(())
    }

    /// Apply data update operation using LWW table
    async fn apply_data_update(&self, table: &str, row: &str, column: &str, value: &serde_json::Value) -> Result<()> {
        // Same as insert for LWW - last write wins
        self.apply_data_insert(table, row, column, value).await
    }

    /// Apply data delete operation using LWW table
    async fn apply_data_delete(&self, table: &str, row: &str, column: &Option<String>) -> Result<()> {
        let mut db = self.lww_db.write().unwrap();
        
        match column {
            Some(col) => {
                // Delete specific cell
                db.set(table, row, col, serde_json::Value::Null);
            },
            None => {
                // Delete entire row
                db.delete_row(table, row);
            }
        }
        
        Ok(())
    }

    /// Update cursor position
    pub async fn update_cursor(&self, session_id: &str, position: CursorPosition, tool: String) -> Result<()> {
        let user_id = {
            let sessions = self.sessions.read().unwrap();
            let session = sessions.get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.user_id.clone()
        };

        // Update cursor state
        {
            let mut cursors = self.cursor_tracker.active_cursors.write().unwrap();
            cursors.insert(session_id.to_string(), CursorState {
                position: position.clone(),
                tool: tool.clone(),
                selection: None,
                last_update: Instant::now(),
                is_active: true,
            });
        }

        // Update session
        {
            let mut sessions = self.sessions.write().unwrap();
            if let Some(session) = sessions.get_mut(session_id) {
                session.cursor_position = Some(position.clone());
                session.current_tool = tool.clone();
                session.last_activity = Utc::now();
            }
        }

        // Broadcast cursor update
        let cursor_update = CursorUpdate {
            session_id: session_id.to_string(),
            user_id,
            position,
            tool,
            timestamp: Utc::now(),
        };

        let _ = self.sync_channels.cursor_sender.send(cursor_update);

        Ok(())
    }

    /// Update user presence
    pub async fn update_presence(&self, user_id: String, status: PresenceStatus) -> Result<()> {
        {
            let mut presence = self.presence_manager.user_presence.write().unwrap();
            presence.insert(user_id.clone(), PresenceInfo {
                status: status.clone(),
                last_seen: Utc::now(),
                current_canvas: None,
                active_tool: None,
            });
        }

        Ok(())
    }

    /// Get current vector clock
    async fn get_vector_clock(&self) -> VectorClock {
        let sessions = self.sessions.read().unwrap();
        let mut clocks = HashMap::new();
        
        for (session_id, session) in sessions.iter() {
            clocks.insert(session.user_id.clone(), 1); // Simplified clock
        }
        
        VectorClock { clocks }
    }

    /// Generate a unique color for a user
    fn generate_user_color(&self, user_id: &str) -> (u8, u8, u8) {
        // Generate deterministic color based on user ID
        let hash = user_id.chars().fold(0u32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u32));
        let r = ((hash & 0xFF0000) >> 16) as u8;
        let g = ((hash & 0x00FF00) >> 8) as u8;
        let b = (hash & 0x0000FF) as u8;
        
        // Ensure colors are vibrant
        (r.max(100), g.max(100), b.max(100))
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<CollaborationSession> {
        let sessions = self.sessions.read().unwrap();
        sessions.values().cloned().collect()
    }

    /// Get active cursors
    pub fn get_active_cursors(&self) -> HashMap<String, CursorState> {
        let cursors = self.cursor_tracker.active_cursors.read().unwrap();
        cursors.clone()
    }

    /// Export CRDT state for synchronization
    pub fn export_state(&self) -> Result<CollaborationState> {
        let y_doc = self.y_doc.read().unwrap();
        let loro_doc = self.loro_doc.read().unwrap();
        let lww_db = self.lww_db.read().unwrap();

        let state = CollaborationState {
            y_doc_state: y_doc.get_update_v1(&y_doc.get_state_vector()),
            loro_doc_state: loro_doc.export_snapshot(),
            lww_db_state: lww_db.export_updates(lww_db.version().clone()),
            timestamp: Utc::now(),
        };

        Ok(state)
    }

    /// Import CRDT state from synchronization
    pub fn import_state(&self, state: &CollaborationState) -> Result<()> {
        {
            let mut y_doc = self.y_doc.write().unwrap();
            y_doc.apply_update_v1(&state.y_doc_state)?;
        }

        {
            let mut loro_doc = self.loro_doc.write().unwrap();
            loro_doc.import_snapshot(&state.loro_doc_state)?;
        }

        {
            let mut lww_db = self.lww_db.write().unwrap();
            lww_db.import_updates(&state.lww_db_state);
        }

        Ok(())
    }

    /// Start real-time synchronization loop
    pub async fn start_sync_loop(&self) -> Result<()> {
        let mut update_receiver = self.sync_channels.update_sender.subscribe();
        let mut cursor_receiver = self.sync_channels.cursor_sender.subscribe();
        let mut presence_receiver = self.sync_channels.presence_sender.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(update) = update_receiver.recv() => {
                        // Process collaboration update
                        tracing::info!("Received collaboration update: {:?}", update);
                    },
                    Ok(cursor_update) = cursor_receiver.recv() => {
                        // Process cursor update
                        tracing::debug!("Received cursor update: {:?}", cursor_update);
                    },
                    Ok(presence_update) = presence_receiver.recv() => {
                        // Process presence update
                        tracing::info!("Received presence update: {:?}", presence_update);
                    },
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Periodic cleanup
                    }
                }
            }
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationState {
    pub y_doc_state: Vec<u8>,
    pub loro_doc_state: Vec<u8>,
    pub lww_db_state: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

impl Default for CollaborationEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collaboration_engine() {
        let engine = CollaborationEngine::new().unwrap();
        
        let session_id = engine.start_session("user1".to_string(), "Test User".to_string()).await.unwrap();
        assert!(!session_id.is_empty());
        
        let sessions = engine.get_active_sessions();
        assert_eq!(sessions.len(), 1);
        
        engine.end_session(&session_id).await.unwrap();
        
        let sessions = engine.get_active_sessions();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_text_operations() {
        let engine = CollaborationEngine::new().unwrap();
        let session_id = engine.start_session("user1".to_string(), "Test User".to_string()).await.unwrap();
        
        let operation = Operation::TextInsert {
            position: 0,
            text: "Hello, World!".to_string(),
        };
        
        engine.apply_operation(&session_id, operation).await.unwrap();
        
        let operation = Operation::TextDelete {
            position: 7,
            length: 6,
        };
        
        engine.apply_operation(&session_id, operation).await.unwrap();
    }

    #[tokio::test]
    async fn test_cursor_tracking() {
        let engine = CollaborationEngine::new().unwrap();
        let session_id = engine.start_session("user1".to_string(), "Test User".to_string()).await.unwrap();
        
        let position = CursorPosition {
            x: 100.0,
            y: 200.0,
            canvas_id: "canvas1".to_string(),
            element_id: None,
            timestamp: Utc::now(),
        };
        
        engine.update_cursor(&session_id, position, "pen".to_string()).await.unwrap();
        
        let cursors = engine.get_active_cursors();
        assert_eq!(cursors.len(), 1);
    }

    #[tokio::test]
    async fn test_data_operations() {
        let engine = CollaborationEngine::new().unwrap();
        let session_id = engine.start_session("user1".to_string(), "Test User".to_string()).await.unwrap();
        
        let operation = Operation::DataInsert {
            table: "test_table".to_string(),
            row: "row1".to_string(),
            column: "col1".to_string(),
            value: serde_json::Value::String("test_value".to_string()),
        };
        
        engine.apply_operation(&session_id, operation).await.unwrap();
        
        let operation = Operation::DataUpdate {
            table: "test_table".to_string(),
            row: "row1".to_string(),
            column: "col1".to_string(),
            value: serde_json::Value::String("updated_value".to_string()),
        };
        
        engine.apply_operation(&session_id, operation).await.unwrap();
    }
} 