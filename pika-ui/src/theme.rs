//! UI theme configuration.

use egui::{Context, Visuals, Color32, Rounding, Vec2};

/// Apply the Pika-Plot dark theme.
pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    
    // Background colors
    visuals.window_fill = Color32::from_gray(30);
    visuals.panel_fill = Color32::from_gray(25);
    visuals.faint_bg_color = Color32::from_gray(20);
    visuals.extreme_bg_color = Color32::from_gray(15);
    
    // Selection and hover
    visuals.selection.bg_fill = Color32::from_rgb(76, 140, 200);
    visuals.widgets.hovered.bg_fill = Color32::from_gray(45);
    visuals.widgets.active.bg_fill = Color32::from_gray(55);
    
    // Rounding
    visuals.window_rounding = Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);
    
    // Shadows
    visuals.window_shadow.extrusion = 8.0;
    visuals.popup_shadow.extrusion = 4.0;
    
    // Button style
    visuals.widgets.inactive.bg_fill = Color32::from_gray(40);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(35);
    
    // Text
    visuals.override_text_color = Some(Color32::from_gray(210));
    
    ctx.set_visuals(visuals);
    
    // Spacing
    let mut spacing = ctx.style().spacing.clone();
    spacing.item_spacing = Vec2::new(8.0, 4.0);
    spacing.button_padding = Vec2::new(8.0, 4.0);
    spacing.menu_margin = 12.0.into();
    spacing.indent = 20.0;
    
    let mut style = (*ctx.style()).clone();
    style.spacing = spacing;
    ctx.set_style(style);
} 