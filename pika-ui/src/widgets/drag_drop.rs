use egui::{Context, CursorIcon, Id, LayerId, Order, Pos2, Rect, Response, Sense, Ui, Vec2};
use std::path::PathBuf;

/// Drag and drop state
#[derive(Clone, Debug)]
pub struct DragDropState {
    /// Files being dragged
    pub dragged_files: Vec<PathBuf>,
    /// Current drag position
    pub drag_pos: Option<Pos2>,
    /// Whether we're currently dragging
    pub is_dragging: bool,
}

impl Default for DragDropState {
    fn default() -> Self {
        Self {
            dragged_files: Vec::new(),
            drag_pos: None,
            is_dragging: false,
        }
    }
}

/// Drag and drop handler
pub struct DragDropHandler {
    id: Id,
    state: DragDropState,
}

impl DragDropHandler {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            state: DragDropState::default(),
        }
    }
    
    /// Handle drag and drop for a UI area
    pub fn handle_drop_area(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
    ) -> Option<Vec<PathBuf>> {
        let response = ui.allocate_rect(rect, Sense::hover());
        
        // Check for file drops
        let mut dropped_files = None;
        
        ui.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                dropped_files = Some(
                    i.raw.dropped_files.iter()
                        .filter_map(|f| f.path.clone())
                        .collect()
                );
            }
        });
        
        // Visual feedback when hovering with files
        if response.hovered() && self.state.is_dragging {
            ui.painter().rect(
                rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            );
            
            ui.ctx().set_cursor_icon(CursorIcon::Copy);
        }
        
        dropped_files
    }
    
    /// Show drop zone overlay
    pub fn show_drop_overlay(&self, ctx: &Context) {
        if !self.state.is_dragging {
            return;
        }
        
        let layer_id = LayerId::new(Order::Tooltip, self.id);
        let painter = ctx.layer_painter(layer_id);
        
        if let Some(pos) = self.state.drag_pos {
            // Draw file count badge
            let text = format!("{} file(s)", self.state.dragged_files.len());
            let galley = painter.layout_no_wrap(
                text,
                egui::FontId::default(),
                egui::Color32::WHITE,
            );
            
            let rect = Rect::from_min_size(
                pos + Vec2::new(10.0, 10.0),
                galley.size() + Vec2::new(16.0, 8.0),
            );
            
            painter.rect_filled(
                rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(50, 50, 50, 200),
            );
            
            painter.galley(
                rect.center() - galley.size() / 2.0,
                galley,
                egui::Color32::WHITE,
            );
        }
    }
}

/// Draggable file widget
pub struct DraggableFile {
    path: PathBuf,
}

impl DraggableFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    
    pub fn show(&self, ui: &mut Ui) -> Response {
        let text = self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown file");
        
        let response = ui.add(
            egui::Label::new(format!("📄 {}", text))
                .sense(Sense::drag())
        );
        
        if response.drag_started() {
            ui.ctx().memory_mut(|mem| {
                mem.data.insert_temp(
                    Id::new("dragged_file"),
                    self.path.clone(),
                );
            });
        }
        
        response
    }
}

/// Drop target widget
pub struct DropTarget {
    id: Id,
    accepts: Vec<String>, // File extensions
}

impl DropTarget {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            accepts: vec!["csv".to_string(), "parquet".to_string(), "json".to_string()],
        }
    }
    
    pub fn accepts(mut self, extensions: Vec<&str>) -> Self {
        self.accepts = extensions.into_iter().map(|s| s.to_string()).collect();
        self
    }
    
    pub fn show(&self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Option<PathBuf> {
        let response = ui.group(|ui| {
            content(ui);
        });
        
        let mut dropped_file = None;
        
        // Check if file was dropped
        if response.response.hovered() {
            ui.ctx().memory(|mem| {
                if let Some(path) = mem.data.get_temp::<PathBuf>(Id::new("dragged_file")) {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if self.accepts.contains(&ext.to_lowercase()) {
                            dropped_file = Some(path.clone());
                        }
                    }
                }
            });
            
            if dropped_file.is_some() {
                ui.ctx().memory_mut(|mem| {
                    mem.data.remove::<PathBuf>(Id::new("dragged_file"));
                });
            }
        }
        
        // Visual feedback
        if response.response.hovered() && ui.input(|i| i.pointer.any_down()) {
            ui.painter().rect_stroke(
                response.response.rect,
                4.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
            );
        }
        
        dropped_file
    }
}

/// Helper to check if files can be dropped
pub fn can_drop_files(ctx: &Context) -> bool {
    ctx.input(|i| {
        !i.raw.hovered_files.is_empty() || !i.raw.dropped_files.is_empty()
    })
}

/// Get hovered files
pub fn get_hovered_files(ctx: &Context) -> Vec<PathBuf> {
    ctx.input(|i| {
        i.raw.hovered_files.iter()
            .filter_map(|f| f.path.clone())
            .collect()
    })
}

/// Get dropped files
pub fn get_dropped_files(ctx: &Context) -> Vec<PathBuf> {
    ctx.input(|i| {
        i.raw.dropped_files.iter()
            .filter_map(|f| f.path.clone())
            .collect()
    })
} 