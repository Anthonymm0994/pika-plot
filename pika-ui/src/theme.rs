//! Theme configuration for the UI.

use egui::{Style, Visuals, Color32, Rounding, Stroke};

/// Apply dark theme to the UI
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();
    
    // Colors
    visuals.panel_fill = Color32::from_rgb(40, 40, 45);
    visuals.window_fill = Color32::from_rgb(35, 35, 40);
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 25);
    
    // Accent colors
    visuals.selection.bg_fill = Color32::from_rgb(70, 120, 200);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(100, 150, 255));
    
    // Borders and outlines
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 65));
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 80, 85));
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 105));
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(120, 120, 125));
    
    // Shadows
    visuals.window_shadow = egui::Shadow {
        offset: egui::Vec2::new(2.0, 8.0),
        blur: 16.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(128),
    };
    visuals.popup_shadow = egui::Shadow {
        offset: egui::Vec2::new(1.0, 4.0),
        blur: 8.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(96),
    };
    
    // Rounding
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);
    
    style.visuals = visuals;
    ctx.set_style(style);
}

/// Apply light theme to the UI
pub fn apply_light_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::light();
    
    // Colors
    visuals.panel_fill = Color32::from_rgb(250, 250, 255);
    visuals.window_fill = Color32::from_rgb(248, 248, 253);
    visuals.extreme_bg_color = Color32::from_rgb(240, 240, 245);
    
    // Accent colors
    visuals.selection.bg_fill = Color32::from_rgb(70, 120, 200);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(50, 100, 180));
    
    style.visuals = visuals;
    ctx.set_style(style);
} 