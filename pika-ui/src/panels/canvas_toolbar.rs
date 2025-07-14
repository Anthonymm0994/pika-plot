//! Canvas toolbar panel for tool selection and canvas controls.

use egui::{Ui, Button, Color32};
use crate::state::{AppState, ToolMode};

pub struct CanvasToolbar;

impl CanvasToolbar {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&mut self, ui: &mut Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            // Canvas label
            ui.label("Pika-Plot Canvas");
            ui.separator();
            
            // Tool buttons
            let tool_button = |ui: &mut Ui, label: &str, tool: ToolMode, current: &ToolMode| -> bool {
                let selected = *current == tool;
                let button = Button::new(label)
                    .fill(if selected { 
                        ui.style().visuals.selection.bg_fill 
                    } else { 
                        Color32::TRANSPARENT 
                    });
                ui.add(button).clicked()
            };
            
            if tool_button(ui, "🔲 Select", ToolMode::Select, &state.tool_mode) {
                state.tool_mode = ToolMode::Select;
            }
            
            if tool_button(ui, "▭ Rectangle", ToolMode::Rectangle, &state.tool_mode) {
                state.tool_mode = ToolMode::Rectangle;
            }
            
            if tool_button(ui, "⭕ Circle", ToolMode::Circle, &state.tool_mode) {
                state.tool_mode = ToolMode::Circle;
            }
            
            if tool_button(ui, "╱ Line", ToolMode::Line, &state.tool_mode) {
                state.tool_mode = ToolMode::Line;
            }
            
            if tool_button(ui, "✏️ Draw", ToolMode::Draw, &state.tool_mode) {
                state.tool_mode = ToolMode::Draw;
            }
            
            if tool_button(ui, "📝 Text", ToolMode::Text, &state.tool_mode) {
                state.tool_mode = ToolMode::Text;
            }
            
            // Spacer to push info to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Elements: {}", state.canvas_nodes.len()));
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", state.canvas_state.zoom));
            });
        });
    }
} 