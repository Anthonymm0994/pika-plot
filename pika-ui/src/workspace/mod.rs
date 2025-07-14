//! Workspace functionality.

pub mod notebook;
pub mod reporting;
pub mod save_load;

pub use notebook::NotebookCell;

use crate::state::AppState;
use egui::Ui;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceMode {
    Canvas,
    Notebook,
}

pub struct WorkspaceManager {
    pub mode: WorkspaceMode,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            mode: WorkspaceMode::Canvas,
        }
    }
    
    pub fn switch_mode(&mut self, mode: WorkspaceMode) {
        self.mode = mode;
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState) {
        match self.mode {
            WorkspaceMode::Canvas => {
                ui.label("Canvas Mode");
            }
            WorkspaceMode::Notebook => {
                ui.label("Notebook Mode");
            }
        }
    }
} 