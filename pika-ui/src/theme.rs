//! Theme configuration for the application.

use egui::{Color32, Context, FontId, FontFamily, Visuals, Style, Stroke};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub fn set_modern_theme(ctx: &Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();
    
    // Modern dark theme colors
    visuals.window_fill = Color32::from_rgb(30, 30, 35);
    visuals.panel_fill = Color32::from_rgb(35, 35, 40);
    visuals.faint_bg_color = Color32::from_rgb(40, 40, 45);
    visuals.extreme_bg_color = Color32::from_rgb(25, 25, 30);
    
    // Selection and interaction colors
    visuals.selection.bg_fill = Color32::from_rgb(70, 90, 120);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(100, 120, 150));
    
    // Widget colors
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(45, 45, 50);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 65));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(180, 180, 185));
    
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(50, 50, 55);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(70, 70, 75));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(160, 160, 165));
    
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 60);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 100, 130));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 200, 205));
    
    visuals.widgets.active.bg_fill = Color32::from_rgb(60, 80, 110);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 120, 150));
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
    
    // Apply visuals
    ctx.set_visuals(visuals);
    
    // Font configuration
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(13.0, FontFamily::Proportional)
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(13.0, FontFamily::Proportional)
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(16.0, FontFamily::Proportional)
    );
    
    ctx.set_style(style);
}

/// Configuration for plot theming
#[derive(Clone)]
pub struct PlotTheme {
    pub background_color: Color32,
    pub grid_color: Color32,
    pub axis_color: Color32,
    pub text_color: Color32,
    pub primary_colors: Vec<Color32>,
    pub secondary_colors: Vec<Color32>,
}

impl PlotTheme {
    /// Create a dark mode plot theme
    pub fn dark() -> Self {
        Self {
            background_color: Color32::from_rgb(35, 35, 40),
            grid_color: Color32::from_rgb(60, 60, 65),
            axis_color: Color32::from_rgb(180, 180, 185),
            text_color: Color32::from_rgb(220, 220, 225),
            primary_colors: vec![
                Color32::from_rgb(100, 150, 250), // Blue
                Color32::from_rgb(250, 150, 100), // Orange
                Color32::from_rgb(150, 250, 100), // Green
                Color32::from_rgb(250, 100, 150), // Pink
                Color32::from_rgb(150, 100, 250), // Purple
                Color32::from_rgb(100, 250, 250), // Cyan
                Color32::from_rgb(250, 250, 100), // Yellow
            ],
            secondary_colors: vec![
                Color32::from_rgb(70, 100, 180),  // Darker Blue
                Color32::from_rgb(180, 100, 70),  // Darker Orange
                Color32::from_rgb(100, 180, 70),  // Darker Green
                Color32::from_rgb(180, 70, 100),  // Darker Pink
                Color32::from_rgb(100, 70, 180),  // Darker Purple
                Color32::from_rgb(70, 180, 180),  // Darker Cyan
                Color32::from_rgb(180, 180, 70),  // Darker Yellow
            ],
        }
    }
    
    /// Create a light mode plot theme
    pub fn light() -> Self {
        Self {
            background_color: Color32::from_rgb(248, 248, 253),
            grid_color: Color32::from_rgb(220, 220, 225),
            axis_color: Color32::from_rgb(60, 60, 65),
            text_color: Color32::from_rgb(40, 40, 45),
            primary_colors: vec![
                Color32::from_rgb(50, 100, 200),  // Blue
                Color32::from_rgb(200, 100, 50),  // Orange
                Color32::from_rgb(100, 200, 50),  // Green
                Color32::from_rgb(200, 50, 100),  // Pink
                Color32::from_rgb(100, 50, 200),  // Purple
                Color32::from_rgb(50, 200, 200),  // Cyan
                Color32::from_rgb(200, 200, 50),  // Yellow
            ],
            secondary_colors: vec![
                Color32::from_rgb(30, 70, 150),   // Darker Blue
                Color32::from_rgb(150, 70, 30),   // Darker Orange
                Color32::from_rgb(70, 150, 30),   // Darker Green
                Color32::from_rgb(150, 30, 70),   // Darker Pink
                Color32::from_rgb(70, 30, 150),   // Darker Purple
                Color32::from_rgb(30, 150, 150),  // Darker Cyan
                Color32::from_rgb(150, 150, 30),  // Darker Yellow
            ],
        }
    }
    
    /// Get a categorical color by index
    pub fn categorical_color(&self, index: usize) -> Color32 {
        self.primary_colors[index % self.primary_colors.len()]
    }
    
    /// Get a secondary categorical color by index
    pub fn secondary_categorical_color(&self, index: usize) -> Color32 {
        self.secondary_colors[index % self.secondary_colors.len()]
    }
    
    /// Create a theme for the given mode
    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }
}

/// Get the current theme mode
pub fn get_theme_mode() -> ThemeMode {
    // For now, always return dark mode
    // In the future, this could be stored in preferences
    ThemeMode::Dark
}