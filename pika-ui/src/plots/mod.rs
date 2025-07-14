//! Plot rendering UI components

pub mod bar_plot;
pub mod box_plot;
pub mod correlation_plot;
pub mod enhanced_scatter_plot;
pub mod heatmap_plot;
pub mod histogram_plot;
pub mod line_plot;
pub mod plot_renderer;
pub mod radar_plot;
pub mod scatter_plot;
pub mod violin_plot;

pub use bar_plot::BarPlot;
pub use box_plot::BoxPlot;
pub use correlation_plot::CorrelationPlot;
pub use enhanced_scatter_plot::EnhancedScatterPlot;
pub use heatmap_plot::HeatmapPlot;
pub use histogram_plot::HistogramPlot;
pub use line_plot::LinePlot;
pub use plot_renderer::render_plot;
pub use radar_plot::RadarPlot;
pub use scatter_plot::ScatterPlot;
pub use violin_plot::ViolinPlot;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{PlotTheme, ThemeMode};
    use egui::Color32;
    
    #[test]
    fn test_plot_theme_dark_mode() {
        let dark_theme = PlotTheme::dark();
        
        // Test dark theme colors
        assert_eq!(dark_theme.background_color, Color32::from_rgb(35, 35, 40));
        assert_eq!(dark_theme.grid_color, Color32::from_rgb(60, 60, 65));
        assert_eq!(dark_theme.axis_color, Color32::from_rgb(180, 180, 185));
        assert_eq!(dark_theme.text_color, Color32::from_rgb(220, 220, 225));
        
        // Test that we have enough colors
        assert!(!dark_theme.primary_colors.is_empty());
        assert!(!dark_theme.secondary_colors.is_empty());
        assert_eq!(dark_theme.primary_colors.len(), dark_theme.secondary_colors.len());
        
        // Test color retrieval
        let first_color = dark_theme.categorical_color(0);
        let second_color = dark_theme.categorical_color(1);
        assert_ne!(first_color, second_color);
        
        // Test color cycling
        let cycled_color = dark_theme.categorical_color(dark_theme.primary_colors.len());
        assert_eq!(cycled_color, first_color);
    }
    
    #[test]
    fn test_plot_theme_light_mode() {
        let light_theme = PlotTheme::light();
        
        // Test light theme colors
        assert_eq!(light_theme.background_color, Color32::from_rgb(248, 248, 253));
        assert_eq!(light_theme.grid_color, Color32::from_rgb(220, 220, 225));
        assert_eq!(light_theme.axis_color, Color32::from_rgb(60, 60, 65));
        assert_eq!(light_theme.text_color, Color32::from_rgb(40, 40, 45));
        
        // Test that we have enough colors
        assert!(!light_theme.primary_colors.is_empty());
        assert!(!light_theme.secondary_colors.is_empty());
        assert_eq!(light_theme.primary_colors.len(), light_theme.secondary_colors.len());
    }
    
    #[test]
    fn test_theme_mode_for_mode() {
        let dark_theme = PlotTheme::for_mode(ThemeMode::Dark);
        let light_theme = PlotTheme::for_mode(ThemeMode::Light);
        
        // Themes should be different
        assert_ne!(dark_theme.background_color, light_theme.background_color);
        assert_ne!(dark_theme.grid_color, light_theme.grid_color);
        assert_ne!(dark_theme.axis_color, light_theme.axis_color);
        assert_ne!(dark_theme.text_color, light_theme.text_color);
    }
    
    #[test]
    fn test_categorical_color_consistency() {
        let dark_theme = PlotTheme::dark();
        let light_theme = PlotTheme::light();
        
        // Test that colors are consistent for the same index
        for i in 0..10 {
            let dark_color1 = dark_theme.categorical_color(i);
            let dark_color2 = dark_theme.categorical_color(i);
            assert_eq!(dark_color1, dark_color2);
            
            let light_color1 = light_theme.categorical_color(i);
            let light_color2 = light_theme.categorical_color(i);
            assert_eq!(light_color1, light_color2);
        }
    }
    
    #[test]
    fn test_secondary_colors() {
        let theme = PlotTheme::dark();
        
        // Test that secondary colors are different from primary
        for i in 0..theme.primary_colors.len() {
            let primary = theme.categorical_color(i);
            let secondary = theme.secondary_categorical_color(i);
            assert_ne!(primary, secondary);
        }
    }
    
    #[test]
    fn test_color_accessibility() {
        let dark_theme = PlotTheme::dark();
        let light_theme = PlotTheme::light();
        
        // Test that colors have sufficient contrast for readability
        // This is a basic test - in practice you'd want more sophisticated contrast checking
        
        // Dark theme should have bright colors on dark background
        let dark_bg = dark_theme.background_color;
        let dark_color = dark_theme.categorical_color(0);
        
        // Light theme should have dark colors on light background
        let light_bg = light_theme.background_color;
        let light_color = light_theme.categorical_color(0);
        
        // Basic brightness check (not a full contrast ratio calculation)
        let dark_brightness = (dark_color.r() as u32 + dark_color.g() as u32 + dark_color.b() as u32) / 3;
        let dark_bg_brightness = (dark_bg.r() as u32 + dark_bg.g() as u32 + dark_bg.b() as u32) / 3;
        
        let light_brightness = (light_color.r() as u32 + light_color.g() as u32 + light_color.b() as u32) / 3;
        let light_bg_brightness = (light_bg.r() as u32 + light_bg.g() as u32 + light_bg.b() as u32) / 3;
        
        // Dark theme: colors should be brighter than background
        assert!(dark_brightness > dark_bg_brightness, "Dark theme colors should be brighter than background");
        
        // Light theme: colors should be darker than background
        assert!(light_brightness < light_bg_brightness, "Light theme colors should be darker than background");
    }
    
    #[test]
    fn test_theme_mode_enum() {
        assert_eq!(ThemeMode::Dark, ThemeMode::Dark);
        assert_eq!(ThemeMode::Light, ThemeMode::Light);
        assert_ne!(ThemeMode::Dark, ThemeMode::Light);
    }
} 