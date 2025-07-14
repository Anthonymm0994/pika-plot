//! Export functionality for plots and data.

pub mod plot_export;

use egui::Context;
use pika_core::{plots::PlotConfig, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Svg,
    Json,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Json => "json",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Png => "PNG Image",
            Self::Svg => "SVG Vector",
            Self::Json => "JSON Data",
        }
    }
}

pub struct ExportOptions {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub background_color: egui::Color32,
    pub title: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            width: 800,
            height: 600,
            dpi: 96,
            background_color: egui::Color32::WHITE,
            title: None,
        }
    }
}

/// Export a plot to file
pub fn export_plot(
    config: &PlotConfig,
    options: &ExportOptions,
    output_path: &PathBuf,
) -> Result<()> {
    // Placeholder implementation
    match options.format {
        ExportFormat::Png => {
            // Export as PNG
            Ok(())
        }
        ExportFormat::Svg => {
            // Export as SVG
            Ok(())
        }
        ExportFormat::Json => {
            // Export as JSON
            Ok(())
        }
    }
}

/// Show export dialog
pub fn show_export_dialog(ctx: &Context, config: &PlotConfig) -> Option<(PathBuf, ExportOptions)> {
    // Placeholder - would show file dialog
    None
} 