# Architecture Patterns in Pika-Plot

## Overview
This document describes the key architectural patterns used throughout the Pika-Plot codebase. Understanding these patterns is essential for maintaining consistency and extending the application.

## 1. Event-Driven Architecture

### Pattern Description
The entire application uses an event-driven architecture with a central event bus for communication between components.

### Implementation
```rust
// Event definition (pika-core/src/events.rs)
pub enum Event {
    NodeCreated { id: NodeId, node_type: String },
    DataImported { table_id: String, info: TableInfo },
    PlotRequested { config: PlotConfig },
    // ... more events
}

// Event bus using broadcast channels
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}
```

### Benefits
- **Loose Coupling**: Components don't need direct references
- **Scalability**: Easy to add new event handlers
- **Testability**: Components can be tested in isolation
- **Async Support**: Natural fit for Rust's async ecosystem

### Usage Example
```rust
// Publishing an event
event_bus.publish(Event::NodeCreated { 
    id: node_id, 
    node_type: "Table".to_string() 
});

// Subscribing to events
let mut rx = event_bus.subscribe();
while let Ok(event) = rx.recv().await {
    match event {
        Event::NodeCreated { id, node_type } => {
            // Handle node creation
        }
        _ => {}
    }
}
```

## 2. State Management Pattern

### Pattern Description
Centralized state management with immutable updates and reactive UI.

### Implementation
```rust
pub struct AppState {
    // Canvas state
    pub canvas_nodes: Vec<CanvasNode>,
    pub connections: Vec<NodeConnection>,
    pub selected_nodes: HashSet<NodeId>,
    
    // Tool state
    pub current_tool: ToolMode,
    pub drawing_state: Option<DrawingState>,
    
    // Data state
    pub data_sources: Vec<TableInfo>,
    pub view_mode: ViewMode,
}
```

### State Update Flow
1. User action triggers event
2. Event handler updates state
3. UI re-renders based on new state
4. Side effects (if any) are triggered

### Best Practices
- Never mutate state directly outside of event handlers
- Use `Arc<Mutex<>>` for shared state across threads
- Keep state minimal and normalized
- Derive UI state from core state

## 3. Node-Based Architecture

### Pattern Description
Everything in the canvas is a node with common properties and behaviors.

### Node Hierarchy
```
CanvasNode (trait)
├── TableNode (data source)
├── PlotNode (visualization)
├── ShapeNode (drawing)
└── TextNode (annotation)
```

### Common Node Interface
```rust
pub trait Node {
    fn id(&self) -> &NodeId;
    fn position(&self) -> Point2;
    fn size(&self) -> Size2;
    fn ports(&self) -> &[Port];
    fn render(&self, painter: &Painter);
    fn handle_event(&mut self, event: &NodeEvent) -> bool;
}
```

### Connection System
- Nodes have ports (input/output)
- Connections link compatible ports
- Data flows from output to input ports
- Bezier curves for visual connections

## 4. Command Pattern (Future)

### Pattern Description
All user actions as reversible commands for undo/redo support.

### Structure
```rust
pub trait Command {
    fn execute(&mut self, state: &mut AppState) -> Result<()>;
    fn undo(&mut self, state: &mut AppState) -> Result<()>;
    fn description(&self) -> &str;
}
```

### Benefits
- Full undo/redo support
- Action logging and replay
- Macro recording capabilities
- Collaborative editing foundation

## 5. Builder Pattern

### Pattern Description
Complex object construction with fluent API.

### Examples
```rust
// Plot configuration
let plot = PlotConfig::builder()
    .plot_type(PlotType::Scatter)
    .title("Sales Analysis")
    .x_column("date")
    .y_column("revenue")
    .theme(PlotTheme::Dark)
    .build();

// Workspace snapshot
let snapshot = SnapshotBuilder::new()
    .with_description("My workspace")
    .add_node(table_node)
    .add_connection(connection)
    .build();
```

## 6. Type State Pattern

### Pattern Description
Compile-time state validation using Rust's type system.

### Example: File Import States
```rust
pub struct FileImport<S> {
    path: PathBuf,
    state: PhantomData<S>,
}

pub struct Unvalidated;
pub struct Validated;
pub struct Configured;

impl FileImport<Unvalidated> {
    pub fn validate(self) -> Result<FileImport<Validated>> {
        // Validation logic
    }
}

impl FileImport<Validated> {
    pub fn configure(self, config: ImportConfig) -> FileImport<Configured> {
        // Configuration logic
    }
}

impl FileImport<Configured> {
    pub fn import(self) -> Result<TableInfo> {
        // Only configured imports can be executed
    }
}
```

## 7. Error Handling Pattern

### Pattern Description
Consistent error handling using Result types and error chaining.

### Error Hierarchy
```rust
#[derive(Error, Debug)]
pub enum PikaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV parsing error: {0}")]
    CsvParse(String),
    
    #[error("Invalid plot configuration: {0}")]
    InvalidPlot(String),
    
    // ... more variants
}

pub type Result<T> = std::result::Result<T, PikaError>;
```

### Best Practices
- Use `?` operator for error propagation
- Provide context with error messages
- Convert external errors to internal types
- Log errors at appropriate levels

## 8. Async/Await Pattern

### Pattern Description
Non-blocking operations for I/O and long-running tasks.

### Usage
```rust
// Data import
pub async fn import_csv(path: PathBuf) -> Result<TableInfo> {
    let data = tokio::fs::read_to_string(path).await?;
    let table = parse_csv_async(data).await?;
    Ok(table)
}

// Event handling
tokio::spawn(async move {
    while let Some(event) = event_stream.next().await {
        process_event(event).await;
    }
});
```

## 9. Trait-Based Extensibility

### Pattern Description
Define behavior contracts through traits for extensibility.

### Key Traits
```rust
// Data source trait
pub trait DataSource: Send + Sync {
    async fn schema(&self) -> Result<Schema>;
    async fn query(&self, sql: &str) -> Result<QueryResult>;
}

// Plot renderer trait
pub trait PlotRenderer {
    fn render(&self, data: &PlotData, config: &PlotConfig) -> Result<ImageBuffer>;
    fn supported_types(&self) -> &[PlotType];
}

// Export format trait
pub trait ExportFormat {
    fn extension(&self) -> &str;
    fn mime_type(&self) -> &str;
    fn export(&self, data: &ExportData) -> Result<Vec<u8>>;
}
```

## 10. Module Organization Pattern

### Pattern Description
Logical grouping of related functionality into modules.

### Structure
```
src/
├── lib.rs          # Public API
├── state.rs        # State management
├── events.rs       # Event definitions
├── panels/         # UI panels
│   ├── mod.rs
│   ├── canvas.rs
│   └── data.rs
├── widgets/        # Reusable widgets
│   ├── mod.rs
│   └── table.rs
└── nodes/          # Node implementations
    ├── mod.rs
    └── plot.rs
```

### Best Practices
- Keep modules focused on a single concern
- Use `mod.rs` for public interface
- Private implementation details in submodules
- Re-export commonly used items

## Conclusion

These patterns work together to create a maintainable, extensible, and performant application. When adding new features:

1. Use the event bus for communication
2. Update state immutably
3. Follow the node interface for canvas items
4. Handle errors consistently
5. Leverage async for I/O operations
6. Define traits for extensibility 