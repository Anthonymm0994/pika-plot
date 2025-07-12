//! UI widgets module.

pub mod file_import_dialog;
pub mod memory_dialog;

pub use file_import_dialog::FileImportDialog;
pub use memory_dialog::{MemoryDialog, MemoryWarningDialog, MemoryAction};

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};

/// Common widget helpers and utilities.
pub mod utils {
    use super::*;
    
    /// Draw a grid background for canvas-like widgets.
    pub fn draw_grid(ui: &mut Ui, rect: Rect, grid_size: f32, color: Color32) {
        let painter = ui.painter();
        
        // Vertical lines
        let mut x = rect.left() + grid_size;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                (1.0, color),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = rect.top() + grid_size;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                (1.0, color),
            );
            y += grid_size;
        }
    }
    
    /// Handle pan and zoom for canvas widgets.
    pub struct PanZoom {
        pub offset: Vec2,
        pub zoom: f32,
    }
    
    impl Default for PanZoom {
        fn default() -> Self {
            Self {
                offset: Vec2::ZERO,
                zoom: 1.0,
            }
        }
    }
    
    impl PanZoom {
        pub fn handle_input(&mut self, ui: &mut Ui, rect: Rect) -> Response {
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            
            // Pan with drag
            if response.dragged() {
                self.offset += response.drag_delta();
            }
            
            // Zoom with scroll
            if response.hovered() {
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 {
                    let zoom_delta = 1.0 + scroll_delta * 0.001;
                    self.zoom *= zoom_delta;
                    self.zoom = self.zoom.clamp(0.1, 10.0);
                    
                    // Zoom towards cursor
                    if let Some(cursor_pos) = response.hover_pos() {
                        let relative_pos = cursor_pos - rect.center();
                        self.offset -= relative_pos * (zoom_delta - 1.0);
                    }
                }
            }
            
            response
        }
        
        /// Convert screen position to canvas position.
        pub fn screen_to_canvas(&self, screen_pos: Pos2, canvas_center: Pos2) -> Pos2 {
            let relative = screen_pos - canvas_center;
            let canvas_pos = (relative - self.offset) / self.zoom;
            canvas_center + canvas_pos
        }
        
        /// Convert canvas position to screen position.
        pub fn canvas_to_screen(&self, canvas_pos: Pos2, canvas_center: Pos2) -> Pos2 {
            let relative = canvas_pos - canvas_center;
            let screen_pos = relative * self.zoom + self.offset;
            canvas_center + screen_pos
        }
    }
} 