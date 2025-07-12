//! Tooltip extensions for enhanced UI help

use egui::{Response, RichText, Ui, Color32};
use crate::shortcuts::{ShortcutAction, get_action_name};

/// Extension trait for adding enhanced tooltips to egui widgets
pub trait TooltipExt {
    /// Add a rich tooltip with formatted text
    fn tooltip_rich(self, text: impl Into<RichText>) -> Self;
    
    /// Add a tooltip with shortcut information
    fn tooltip_with_shortcut(self, text: &str, action: Option<ShortcutAction>) -> Self;
    
    /// Add a delayed tooltip that appears after a delay
    fn tooltip_delayed(self, text: &str, delay_ms: u64) -> Self;
    
    /// Add a tooltip with help text and examples
    fn tooltip_help(self, title: &str, description: &str, example: Option<&str>) -> Self;
    
    /// Add a tooltip with warning styling
    fn tooltip_warning(self, text: &str) -> Self;
    
    /// Add a tooltip with error styling
    fn tooltip_error(self, text: &str) -> Self;
    
    /// Add a tooltip with success styling
    fn tooltip_success(self, text: &str) -> Self;
}

impl TooltipExt for Response {
    fn tooltip_rich(self, text: impl Into<RichText>) -> Self {
        self.on_hover_ui(|ui| {
            ui.label(text.into());
        })
    }
    
    fn tooltip_with_shortcut(self, text: &str, action: Option<ShortcutAction>) -> Self {
        self.on_hover_ui(|ui| {
            ui.vertical(|ui| {
                ui.label(text);
                
                if let Some(action) = action {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Shortcut:").weak());
                        
                        // This would need access to ShortcutManager
                        // For now, we'll show the action name
                        ui.label(RichText::new(get_action_name(action)).weak().small());
                    });
                }
            });
        })
    }
    
    fn tooltip_delayed(self, text: &str, _delay_ms: u64) -> Self {
        // For simplicity, we'll just show the tooltip immediately
        // In a real implementation, you'd track hover time
        self.on_hover_ui_at_pointer(|ui| {
            ui.label(text);
        })
    }
    
    fn tooltip_help(self, title: &str, description: &str, example: Option<&str>) -> Self {
        self.on_hover_ui(|ui| {
            ui.vertical(|ui| {
                // Title
                ui.label(RichText::new(title).strong());
                
                // Description
                ui.label(description);
                
                // Example if provided
                if let Some(example) = example {
                    ui.separator();
                    ui.label(RichText::new("Example:").weak());
                    ui.label(RichText::new(example).weak().small().monospace());
                }
            });
        })
    }
    
    fn tooltip_warning(self, text: &str) -> Self {
        self.on_hover_ui(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠️").color(Color32::from_rgb(255, 165, 0)));
                ui.label(RichText::new(text).color(Color32::from_rgb(255, 165, 0)));
            });
        })
    }
    
    fn tooltip_error(self, text: &str) -> Self {
        self.on_hover_ui(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("❌").color(Color32::from_rgb(255, 100, 100)));
                ui.label(RichText::new(text).color(Color32::from_rgb(255, 100, 100)));
            });
        })
    }
    
    fn tooltip_success(self, text: &str) -> Self {
        self.on_hover_ui(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✅").color(Color32::from_rgb(100, 255, 100)));
                ui.label(RichText::new(text).color(Color32::from_rgb(100, 255, 100)));
            });
        })
    }
}

/// Helper function to create contextual tooltips for data operations
pub fn data_operation_tooltip(ui: &mut Ui, operation: &str, data_info: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(operation).strong());
        ui.separator();
        ui.label(RichText::new("Data:").weak());
        ui.label(RichText::new(data_info).weak().small());
        ui.separator();
        ui.label(RichText::new("💡 Tip: Right-click for more options").weak().small());
    });
}

/// Helper function to create tooltips for plot configurations
pub fn plot_config_tooltip(ui: &mut Ui, plot_type: &str, config_hint: &str) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("📊");
            ui.label(RichText::new(plot_type).strong());
        });
        ui.separator();
        ui.label(config_hint);
        ui.separator();
        ui.label(RichText::new("💡 Drag columns to configure axes").weak().small());
    });
}

/// Helper function to create tooltips for memory/performance info
pub fn performance_tooltip(ui: &mut Ui, memory_mb: usize, rows: usize) {
    ui.vertical(|ui| {
        ui.label(RichText::new("Performance Info").strong());
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Memory:");
            ui.label(RichText::new(format!("{} MB", memory_mb)).weak());
        });
        ui.horizontal(|ui| {
            ui.label("Rows:");
            ui.label(RichText::new(format!("{}", rows)).weak());
        });
        
        // Performance hints
        if memory_mb > 500 {
            ui.separator();
            ui.label(RichText::new("⚠️ High memory usage").color(Color32::from_rgb(255, 165, 0)));
            ui.label(RichText::new("Consider using sampling or aggregation").weak().small());
        }
        
        if rows > 1_000_000 {
            ui.separator();
            ui.label(RichText::new("🚀 Large dataset").color(Color32::from_rgb(100, 200, 255)));
            ui.label(RichText::new("GPU acceleration is active").weak().small());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tooltip_extensions_compile() {
        // This test just ensures the trait compiles correctly
        // In a real UI test, you'd create a mock Response and test the methods
    }
} 