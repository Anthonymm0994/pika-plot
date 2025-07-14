# Code Quality Guide

## Overview
This guide outlines code quality standards, best practices, and conventions used throughout the Pika-Plot codebase.

## Rust Best Practices

### 1. Error Handling

#### Use Custom Error Types
```rust
// Good: Custom error with context
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Failed to read CSV file: {0}")]
    FileRead(#[from] std::io::Error),
    
    #[error("Invalid CSV format at line {line}: {message}")]
    InvalidFormat { line: usize, message: String },
}

// Use Result type alias
pub type Result<T> = std::result::Result<T, ImportError>;
```

#### Propagate Errors with Context
```rust
// Good: Add context to errors
pub fn import_csv(path: &Path) -> Result<TableInfo> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ImportError::FileRead(e))?;
    
    parse_csv(&content)
        .map_err(|e| ImportError::InvalidFormat {
            line: e.line,
            message: format!("in file {}", path.display()),
        })?
}
```

### 2. Memory Management

#### Use Arc for Shared State
```rust
// Good: Shared ownership with Arc
pub struct App {
    state: Arc<Mutex<AppState>>,
    event_bus: Arc<EventBus>,
}

// Clone is cheap for Arc
let state_clone = self.state.clone();
tokio::spawn(async move {
    let mut state = state_clone.lock().await;
    state.update();
});
```

#### Avoid Unnecessary Clones
```rust
// Bad: Unnecessary clone
let name = table.name.clone();
process_name(name);

// Good: Borrow when possible
process_name(&table.name);

// Good: Move when ownership is transferred
let name = std::mem::take(&mut table.name);
process_name(name);
```

### 3. Async Best Practices

#### Use Async for I/O
```rust
// Good: Async file operations
pub async fn load_data(path: &Path) -> Result<Vec<u8>> {
    tokio::fs::read(path).await
        .map_err(|e| DataError::FileRead(e))
}

// Good: Concurrent operations
pub async fn load_multiple(paths: &[PathBuf]) -> Result<Vec<TableInfo>> {
    let futures = paths.iter()
        .map(|p| load_table(p))
        .collect::<Vec<_>>();
    
    futures::future::try_join_all(futures).await
}
```

#### Avoid Blocking in Async Context
```rust
// Bad: Blocking in async
async fn process() {
    let data = std::fs::read("file.csv").unwrap(); // Blocks!
}

// Good: Use async versions
async fn process() {
    let data = tokio::fs::read("file.csv").await.unwrap();
}

// Good: Use spawn_blocking for CPU-intensive work
async fn compute() {
    let result = tokio::task::spawn_blocking(|| {
        expensive_computation()
    }).await.unwrap();
}
```

### 4. Type Safety

#### Use NewType Pattern
```rust
// Good: Type-safe IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableId(String);

// Prevents mixing different ID types
fn connect(from: NodeId, to: NodeId) { /* ... */ }
```

#### Use Enums for State
```rust
// Good: Explicit states
pub enum ImportState {
    Idle,
    Loading { progress: f32 },
    Configuring { data: PreviewData },
    Complete { table: TableInfo },
    Failed { error: ImportError },
}

// Exhaustive matching enforced
match state {
    ImportState::Idle => { /* ... */ }
    ImportState::Loading { progress } => { /* ... */ }
    // Must handle all cases
}
```

### 5. Performance

#### Use Iterators Efficiently
```rust
// Bad: Collect then iterate
let items: Vec<_> = data.iter().map(process).collect();
for item in items {
    handle(item);
}

// Good: Chain iterators
data.iter()
    .map(process)
    .for_each(handle);

// Good: Early termination
let found = data.iter()
    .find(|item| item.matches(query))
    .cloned();
```

#### Minimize Allocations
```rust
// Bad: Allocate in loop
for i in 0..1000 {
    let mut vec = Vec::new();
    vec.push(i);
}

// Good: Reuse allocation
let mut vec = Vec::with_capacity(1);
for i in 0..1000 {
    vec.clear();
    vec.push(i);
}
```

## Code Organization

### 1. Module Structure

```rust
// Good: Clear module organization
pub mod panels {
    mod canvas;      // Private implementation
    mod properties;  // Private implementation
    
    pub use canvas::CanvasPanel;      // Public API
    pub use properties::PropertiesPanel; // Public API
}
```

### 2. Trait Design

```rust
// Good: Small, focused traits
pub trait Renderable {
    fn render(&self, painter: &Painter);
}

pub trait Selectable {
    fn hit_test(&self, pos: Pos2) -> bool;
    fn select(&mut self);
    fn deselect(&mut self);
}

// Combine with trait bounds
impl<T> Canvas<T> 
where 
    T: Renderable + Selectable
{
    // Implementation
}
```

### 3. Documentation

```rust
/// Imports CSV data from the specified path.
/// 
/// # Arguments
/// * `path` - Path to the CSV file
/// * `config` - Import configuration
/// 
/// # Returns
/// Returns `TableInfo` on success, or an error if:
/// - File cannot be read
/// - CSV format is invalid
/// - Column types cannot be inferred
/// 
/// # Example
/// ```
/// let config = ImportConfig::default();
/// let table = import_csv("data.csv", config).await?;
/// ```
pub async fn import_csv(path: &Path, config: ImportConfig) -> Result<TableInfo> {
    // Implementation
}
```

## Testing

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_creation() {
        let node = CanvasNode::new(NodeType::Table);
        assert_eq!(node.node_type(), NodeType::Table);
        assert!(!node.is_selected());
    }
    
    #[tokio::test]
    async fn test_async_import() {
        let data = import_csv("test.csv").await;
        assert!(data.is_ok());
    }
}
```

### 2. Integration Tests

```rust
// tests/integration_test.rs
#[test]
fn test_full_workflow() {
    let app = create_test_app();
    
    // Import data
    app.import_csv("test_data.csv");
    assert_eq!(app.table_count(), 1);
    
    // Create plot
    let plot_id = app.create_plot(PlotType::Scatter);
    assert!(app.has_node(plot_id));
    
    // Verify connection
    assert!(app.has_connection_to(plot_id));
}
```

## Common Patterns

### 1. Event Publishing

```rust
// Consistent event publishing pattern
impl DataPanel {
    fn handle_import(&mut self, path: PathBuf) {
        match import_csv(&path) {
            Ok(table) => {
                self.event_bus.publish(Event::DataImported {
                    table_id: table.id.clone(),
                    info: table,
                });
            }
            Err(e) => {
                self.event_bus.publish(Event::Error {
                    message: format!("Import failed: {}", e),
                });
            }
        }
    }
}
```

### 2. State Updates

```rust
// Consistent state update pattern
impl AppState {
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::NodeCreated { id, node_type } => {
                let node = create_node(id, node_type);
                self.canvas_nodes.push(node);
                self.mark_dirty();
            }
            Event::NodeDeleted { id } => {
                self.canvas_nodes.retain(|n| n.id() != &id);
                self.connections.retain(|c| !c.involves(&id));
                self.mark_dirty();
            }
            // Handle all events
        }
    }
}
```

### 3. UI Rendering

```rust
// Consistent UI pattern
impl Panel for DataSourcesPanel {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Data Sources");
        
        ui.horizontal(|ui| {
            if ui.button("➕ Import CSV...").clicked() {
                self.show_import_dialog = true;
            }
        });
        
        ScrollArea::vertical().show(ui, |ui| {
            for table in &self.tables {
                self.render_table_item(ui, table);
            }
        });
        
        // Handle modals
        if self.show_import_dialog {
            self.render_import_dialog(ui);
        }
    }
}
```

## Performance Guidelines

### 1. Measure First
- Use `cargo bench` for performance testing
- Profile with `perf` or `flamegraph`
- Optimize hot paths only

### 2. Common Optimizations
- Pre-allocate collections with `with_capacity`
- Use `SmallVec` for small collections
- Cache expensive computations
- Batch UI updates

### 3. Avoid Premature Optimization
- Write clear code first
- Optimize based on measurements
- Document performance-critical sections

## Security Considerations

### 1. Input Validation
```rust
// Validate all external input
pub fn parse_csv(content: &str) -> Result<CsvData> {
    // Check file size
    if content.len() > MAX_FILE_SIZE {
        return Err(ImportError::FileTooLarge);
    }
    
    // Validate format
    let reader = csv::Reader::from_reader(content.as_bytes());
    // ... validation logic
}
```

### 2. Path Handling
```rust
// Sanitize file paths
pub fn safe_path(base: &Path, user_input: &str) -> Result<PathBuf> {
    let path = base.join(user_input);
    
    // Ensure path is within base directory
    if !path.starts_with(base) {
        return Err(SecurityError::PathTraversal);
    }
    
    Ok(path)
}
```

## Code Review Checklist

- [ ] Error handling uses Result types
- [ ] No unwrap() in production code
- [ ] Async functions for I/O operations
- [ ] Documentation for public APIs
- [ ] Tests for new functionality
- [ ] No sensitive data in logs
- [ ] Performance impact considered
- [ ] Follows existing patterns
- [ ] No compiler warnings
- [ ] Dependencies are minimal 