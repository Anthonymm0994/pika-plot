//! Workspace management module.

mod notebook;
mod save_load;

pub use notebook::{NotebookMode, NotebookCell};
pub use save_load::{save_workspace, load_workspace, apply_snapshot, AutoSave};

use pika_core::types::NodeId;
use egui::{Ui, Color32};

/// Workspace modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    Canvas,
    Notebook,
}

/// Main workspace structure
pub struct Workspace {
    mode: WorkspaceMode,
    notebook: NotebookMode,
}

impl Workspace {
    /// Create a new workspace
    pub fn new() -> Self {
        Self {
            mode: WorkspaceMode::Canvas,
            notebook: NotebookMode::new(),
        }
    }
    
    /// Get current mode
    pub fn mode(&self) -> WorkspaceMode {
        self.mode
    }
    
    /// Set workspace mode
    pub fn set_mode(&mut self, mode: WorkspaceMode) {
        self.mode = mode;
    }
    
    /// Show mode switcher
    pub fn show_mode_switcher(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Mode:");
            
            if ui.selectable_label(self.mode == WorkspaceMode::Canvas, "🎨 Canvas").clicked() {
                self.mode = WorkspaceMode::Canvas;
            }
            
            if ui.selectable_label(self.mode == WorkspaceMode::Notebook, "📓 Notebook").clicked() {
                self.mode = WorkspaceMode::Notebook;
            }
        });
    }
    
    /// Convert canvas nodes to notebook cells
    pub fn canvas_to_notebook(&mut self, nodes: Vec<NodeId>) {
        // TODO: Implement conversion logic
        self.notebook.clear();
        
        // For each node, create a corresponding cell
        for (i, node_id) in nodes.iter().enumerate() {
            let cell = NotebookCell {
                id: *node_id,
                cell_type: notebook::CellType::Query,
                content: format!("-- Node {} converted to cell", i),
                execution_count: None,
                output: None,
                is_collapsed: false,
            };
            self.notebook.add_cell(cell);
        }
    }
    
    /// Convert notebook cells to canvas nodes
    pub fn notebook_to_canvas(&mut self) -> Vec<(NodeId, egui::Pos2)> {
        let mut positions = Vec::new();
        
        // Arrange nodes in a grid
        let grid_spacing = 200.0;
        let nodes_per_row = 4;
        
        for (i, cell) in self.notebook.cells().iter().enumerate() {
            let row = i / nodes_per_row;
            let col = i % nodes_per_row;
            
            let x = 100.0 + (col as f32) * grid_spacing;
            let y = 100.0 + (row as f32) * grid_spacing;
            
            positions.push((cell.id, egui::pos2(x, y)));
        }
        
        positions
    }
    
    /// Render the workspace
    pub fn render(&mut self, ui: &mut Ui) {
        match self.mode {
            WorkspaceMode::Canvas => {
                // Canvas mode is rendered elsewhere
                ui.label("Canvas mode active");
            }
            WorkspaceMode::Notebook => {
                self.notebook.render(ui);
            }
        }
    }
} 