//! Export functionality for plots and data.

mod plot_export;

use pika_core::{Result, PikaError};
use std::path::Path;

pub use plot_export::{
    export_plot_to_image,
    export_plot_data_to_csv,
    export_plot_data_to_json,
};

/// Export types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    PlotImage,
    Data,
    Workspace,
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    // Image formats
    Png,
    Svg,
    // Data formats
    Csv,
    Json,
    Parquet,
    // Workspace format
    PikaWorkspace,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub png_options: Option<PngExportOptions>,
    pub csv_options: Option<CsvExportOptions>,
    pub json_options: Option<JsonExportOptions>,
}

#[derive(Debug, Clone)]
pub struct PngExportOptions {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub transparent: bool,
}

#[derive(Debug, Clone)]
pub struct CsvExportOptions {
    pub delimiter: u8,
    pub include_header: bool,
}

#[derive(Debug, Clone)]
pub struct JsonExportOptions {
    pub pretty: bool,
    pub indent: usize,
}

impl Default for PngExportOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            dpi: 96,
            transparent: false,
        }
    }
}

impl Default for CsvExportOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            include_header: true,
        }
    }
}

impl Default for JsonExportOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            indent: 2,
        }
    }
}

/// Export manager
pub struct ExportManager {
    current_export: Option<ExportType>,
}

impl ExportManager {
    pub fn new() -> Self {
        Self {
            current_export: None,
        }
    }
    
    /// Start an export
    pub fn start_export(&mut self, export_type: ExportType) {
        self.current_export = Some(export_type);
    }
    
    /// Cancel current export
    pub fn cancel_export(&mut self) {
        self.current_export = None;
    }
    
    /// Get current export type
    pub fn current_export(&self) -> Option<ExportType> {
        self.current_export
    }
    
    /// Detect format from file extension
    pub fn detect_format(path: &Path) -> Option<ExportFormat> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        
        match ext.as_str() {
            "png" => Some(ExportFormat::Png),
            "svg" => Some(ExportFormat::Svg),
            "csv" => Some(ExportFormat::Csv),
            "json" => Some(ExportFormat::Json),
            "parquet" => Some(ExportFormat::Parquet),
            "pika" => Some(ExportFormat::PikaWorkspace),
            _ => None,
        }
    }
    
    /// Export data to file
    pub fn export_data(
        &self,
        data: &arrow::record_batch::RecordBatch,
        path: &Path,
        options: &ExportOptions,
    ) -> Result<()> {
        match options.format {
            ExportFormat::Csv => {
                let csv_opts = options.csv_options.as_ref()
                    .cloned()
                    .unwrap_or_default();
                
                export_plot_data_to_csv(
                    data,
                    path,
                    csv_opts.delimiter,
                    csv_opts.include_header,
                )
            }
            ExportFormat::Json => {
                let json_opts = options.json_options.as_ref()
                    .cloned()
                    .unwrap_or_default();
                
                export_plot_data_to_json(
                    data,
                    path,
                    json_opts.pretty,
                )
            }
            ExportFormat::Parquet => {
                Err(PikaError::NotImplemented {
                    feature: "Parquet export".to_string(),
                })
            }
            _ => Err(PikaError::Internal("Invalid format for data export".to_string())),
        }
    }
    
    /// Export plot as image
    pub fn export_plot(
        &self,
        ctx: &egui::Context,
        plot_data: &arrow::record_batch::RecordBatch,
        config: &pika_core::plots::PlotConfig,
        path: &Path,
        options: &ExportOptions,
    ) -> Result<()> {
        match options.format {
            ExportFormat::Png => {
                let png_opts = options.png_options.as_ref()
                    .cloned()
                    .unwrap_or_default();
                
                export_plot_to_image(
                    ctx,
                    plot_data,
                    config,
                    path,
                    png_opts.width,
                    png_opts.height,
                    png_opts.dpi,
                )
            }
            ExportFormat::Svg => {
                export_plot_to_image(
                    ctx,
                    plot_data,
                    config,
                    path,
                    1920, // Default SVG size
                    1080,
                    96,
                )
            }
            _ => Err(PikaError::Internal("Invalid format for plot export".to_string())),
        }
    }
} 