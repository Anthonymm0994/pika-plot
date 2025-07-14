//! Theme configuration for the UI.

use egui::Color32;

/// Basic theme configuration for Pika-Plot
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary_color: Color32,
    pub secondary_color: Color32,
    pub background_color: Color32,
    pub text_color: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: Color32::from_rgb(70, 130, 180),
            secondary_color: Color32::from_rgb(100, 149, 237),
            background_color: Color32::from_rgb(40, 40, 40),
            text_color: Color32::WHITE,
        }
    }
} 