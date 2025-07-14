use egui::{Context, Vec2, Color32};
use arrow::record_batch::RecordBatch;
use pika_core::plots::PlotConfig;
use pika_core::error::Result;
use crate::theme::{PlotTheme, ThemeMode};
use std::path::Path;
use plotters::prelude::*;
use pika_engine::plot::extract_numeric_values;
use arrow::array::{StringArray, Float64Array};

/// Export formats supported for plots
#[derive(Debug, Clone, Copy)]
pub enum PlotExportFormat {
    Png,
    Svg,
    Pdf,
}

impl PlotExportFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg", 
            Self::Pdf => "pdf",
        }
    }
}

/// Plot export configuration
#[derive(Debug, Clone)]
pub struct PlotExportConfig {
    pub width: u32,
    pub height: u32,
    pub format: PlotExportFormat,
    pub theme_mode: ThemeMode,
    pub dpi: f32,
    pub background_color: Option<Color32>,
}

impl Default for PlotExportConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: PlotExportFormat::Png,
            theme_mode: ThemeMode::Light,
            dpi: 96.0,
            background_color: None,
        }
    }
}

/// Main plot export functionality
pub struct PlotExporter {
    config: PlotExportConfig,
}

impl PlotExporter {
    pub fn new(config: PlotExportConfig) -> Self {
        Self { config }
    }
    
    /// Export a plot to file
    pub async fn export_plot(
        &self,
        plot_config: &PlotConfig,
        data: &RecordBatch,
        output_path: &Path,
    ) -> Result<()> {
        match self.config.format {
            PlotExportFormat::Png => self.export_png(plot_config, data, output_path).await,
            PlotExportFormat::Svg => self.export_svg(plot_config, data, output_path).await,
            PlotExportFormat::Pdf => self.export_pdf(plot_config, data, output_path).await,
        }
    }
    
    async fn export_png(
        &self,
        plot_config: &PlotConfig,
        data: &RecordBatch,
        output_path: &Path,
    ) -> Result<()> {
        let plot_theme = PlotTheme::for_mode(self.config.theme_mode);
        
        // Create PNG backend
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        
        self.render_with_plotters(plot_config, data, &root, &plot_theme).await?;
        
        root.present()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to save PNG: {}", e)))?;
        
        Ok(())
    }
    
    async fn export_svg(
        &self,
        plot_config: &PlotConfig,
        data: &RecordBatch,
        output_path: &Path,
    ) -> Result<()> {
        let plot_theme = PlotTheme::for_mode(self.config.theme_mode);
        
        // Create SVG backend
        let root = SVGBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        
        self.render_with_plotters(plot_config, data, &root, &plot_theme).await?;
        
        root.present()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to save SVG: {}", e)))?;
        
        Ok(())
    }
    
    async fn export_pdf(
        &self,
        plot_config: &PlotConfig,
        data: &RecordBatch,
        output_path: &Path,
    ) -> Result<()> {
        // For PDF, we'll render to SVG first and then convert
        // This is a simplified approach - a full implementation would use a proper PDF backend
        let svg_path = output_path.with_extension("svg");
        self.export_svg(plot_config, data, &svg_path).await?;
        
        // Create a simple PDF placeholder
        let pdf_content = format!(
            "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
            2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
            3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] >>\nendobj\n\
            xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n\
            0000115 00000 n \ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n178\n%%EOF",
            self.config.width, self.config.height
        );
        
        tokio::fs::write(output_path, pdf_content).await
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to save PDF: {}", e)))?;
        
        Ok(())
    }
    
    async fn render_with_plotters<DB: DrawingBackend>(
        &self,
        plot_config: &PlotConfig,
        data: &RecordBatch,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        plot_theme: &PlotTheme,
    ) -> Result<()>
    where
        DB::ErrorType: 'static + std::error::Error + Send + Sync,
    {
        // Set background color
        let bg_color = self.config.background_color.unwrap_or(plot_theme.background_color);
        let bg_plotters_color = RGBColor(bg_color.r(), bg_color.g(), bg_color.b());
        
        root.fill(&bg_plotters_color)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to set background: {}", e)))?;
        
        match &plot_config.specific {
            pika_core::plots::PlotDataConfig::ScatterConfig { x_column, y_column, .. } => {
                self.render_scatter_plot(root, data, x_column, y_column, plot_config, plot_theme).await?;
            }
            pika_core::plots::PlotDataConfig::LineConfig { x_column, y_column, .. } => {
                self.render_line_plot(root, data, x_column, y_column, plot_config, plot_theme).await?;
            }
            pika_core::plots::PlotDataConfig::BarConfig { category_column, value_column, .. } => {
                self.render_bar_plot(root, data, category_column, value_column, plot_config, plot_theme).await?;
            }
            pika_core::plots::PlotDataConfig::HistogramConfig { column, num_bins, .. } => {
                self.render_histogram_plot(root, data, column, *num_bins, plot_config, plot_theme).await?;
            }
            _ => {
                return Err(pika_core::error::PikaError::NotImplemented("Plot type not yet supported for export"));
            }
        }
        
        Ok(())
    }
    
    async fn render_scatter_plot<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        data: &RecordBatch,
        x_column: &str,
        y_column: &str,
        plot_config: &PlotConfig,
        plot_theme: &PlotTheme,
    ) -> Result<()>
    where
        DB::ErrorType: 'static + std::error::Error + Send + Sync,
    {
        // Extract data
        let x_array = data.column_by_name(x_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", x_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        let y_array = data.column_by_name(y_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", y_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        let x_values = extract_numeric_values(x_array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract X values: {}", e)))?;
        
        let y_values = extract_numeric_values(y_array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract Y values: {}", e)))?;
        
        // Calculate ranges
        let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Add some padding
        let x_range = x_max - x_min;
        let y_range = y_max - y_min;
        let x_padding = x_range * 0.1;
        let y_padding = y_range * 0.1;
        
        // Create chart
        let mut chart = ChartBuilder::on(root)
            .caption(
                plot_config.title.as_deref().unwrap_or("Scatter Plot"),
                ("Arial", 30).into_font().color(&RGBColor(
                    plot_theme.text_color.r(),
                    plot_theme.text_color.g(),
                    plot_theme.text_color.b()
                ))
            )
            .margin(20)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                (x_min - x_padding)..(x_max + x_padding),
                (y_min - y_padding)..(y_max + y_padding)
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to create chart: {}", e)))?;
        
        // Configure chart
        let axis_color = RGBColor(plot_theme.axis_color.r(), plot_theme.axis_color.g(), plot_theme.axis_color.b());
        let grid_color = RGBColor(plot_theme.grid_color.r(), plot_theme.grid_color.g(), plot_theme.grid_color.b());
        
        chart
            .configure_mesh()
            .x_desc(plot_config.x_label.as_deref().unwrap_or(x_column))
            .y_desc(plot_config.y_label.as_deref().unwrap_or(y_column))
            .axis_style(&axis_color)
            .light_line_style(&grid_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to configure mesh: {}", e)))?;
        
        // Plot points
        let primary_color = plot_theme.categorical_color(0);
        let point_color = RGBColor(primary_color.r(), primary_color.g(), primary_color.b());
        
        let points: Vec<(f64, f64)> = x_values.iter().zip(y_values.iter()).map(|(&x, &y)| (x, y)).collect();
        
        chart
            .draw_series(
                points.iter().map(|&point| Circle::new(point, 3, point_color.filled()))
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw points: {}", e)))?
            .label("Data Points")
            .legend(move |(x, y)| Circle::new((x + 5, y), 3, point_color.filled()));
        
        chart
            .configure_series_labels()
            .background_style(&RGBColor(255, 255, 255).mix(0.8))
            .border_style(&axis_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw legend: {}", e)))?;
        
        Ok(())
    }
    
    async fn render_line_plot<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        data: &RecordBatch,
        x_column: &str,
        y_column: &str,
        plot_config: &PlotConfig,
        plot_theme: &PlotTheme,
    ) -> Result<()>
    where
        DB::ErrorType: 'static + std::error::Error + Send + Sync,
    {
        // Extract and sort data
        let x_array = data.column_by_name(x_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", x_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        let y_array = data.column_by_name(y_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", y_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        let x_values = extract_numeric_values(x_array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract X values: {}", e)))?;
        
        let y_values = extract_numeric_values(y_array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract Y values: {}", e)))?;
        
        // Sort by x values
        let mut points: Vec<(f64, f64)> = x_values.iter().zip(y_values.iter()).map(|(&x, &y)| (x, y)).collect();
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Calculate ranges
        let x_min = points.iter().map(|p| p.0).fold(f64::INFINITY, |a, b| a.min(b));
        let x_max = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, |a, b| a.min(b));
        let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, |a, b| a.max(b));
        
        // Add padding
        let x_range = x_max - x_min;
        let y_range = y_max - y_min;
        let x_padding = x_range * 0.1;
        let y_padding = y_range * 0.1;
        
        // Create chart
        let mut chart = ChartBuilder::on(root)
            .caption(
                plot_config.title.as_deref().unwrap_or("Line Plot"),
                ("Arial", 30).into_font().color(&RGBColor(
                    plot_theme.text_color.r(),
                    plot_theme.text_color.g(),
                    plot_theme.text_color.b()
                ))
            )
            .margin(20)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                (x_min - x_padding)..(x_max + x_padding),
                (y_min - y_padding)..(y_max + y_padding)
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to create chart: {}", e)))?;
        
        // Configure chart
        let axis_color = RGBColor(plot_theme.axis_color.r(), plot_theme.axis_color.g(), plot_theme.axis_color.b());
        let grid_color = RGBColor(plot_theme.grid_color.r(), plot_theme.grid_color.g(), plot_theme.grid_color.b());
        
        chart
            .configure_mesh()
            .x_desc(plot_config.x_label.as_deref().unwrap_or(x_column))
            .y_desc(plot_config.y_label.as_deref().unwrap_or(y_column))
            .axis_style(&axis_color)
            .light_line_style(&grid_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to configure mesh: {}", e)))?;
        
        // Plot line
        let primary_color = plot_theme.categorical_color(0);
        let line_color = RGBColor(primary_color.r(), primary_color.g(), primary_color.b());
        
        chart
            .draw_series(LineSeries::new(points.iter().cloned(), &line_color))
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw line: {}", e)))?
            .label("Data Series")
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], line_color.stroke_width(2)));
        
        chart
            .configure_series_labels()
            .background_style(&RGBColor(255, 255, 255).mix(0.8))
            .border_style(&axis_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw legend: {}", e)))?;
        
        Ok(())
    }
    
    async fn render_bar_plot<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        data: &RecordBatch,
        category_column: &str,
        value_column: &str,
        plot_config: &PlotConfig,
        plot_theme: &PlotTheme,
    ) -> Result<()>
    where
        DB::ErrorType: 'static + std::error::Error + Send + Sync,
    {
        // Extract data
        let cat_array = data.column_by_name(category_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", category_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        let val_array = data.column_by_name(value_column)
            .ok_or_else(|| pika_core::error::PikaError::UserError(
                format!("Column '{}' not found", value_column),
                "Check column names in your data".to_string(),
                vec![]
            ))?;
        
        // Try to extract as string array
        let categories = if let Some(string_array) = cat_array.as_any().downcast_ref::<StringArray>() {
            string_array.iter().map(|s| s.unwrap_or("").to_string()).collect::<Vec<_>>()
        } else {
            // Fallback to numeric conversion
            let numeric_cats = extract_numeric_values(cat_array)
                .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract categories: {}", e)))?;
            numeric_cats.iter().map(|&n| format!("{:.1}", n)).collect()
        };
        
        let values = extract_numeric_values(val_array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract values: {}", e)))?;
        
        // Calculate value range
        let y_min = 0.0f64;
        let y_max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_padding = y_max * 0.1;
        
        // Create chart
        let mut chart = ChartBuilder::on(root)
            .caption(
                plot_config.title.as_deref().unwrap_or("Bar Plot"),
                ("Arial", 30).into_font().color(&RGBColor(
                    plot_theme.text_color.r(),
                    plot_theme.text_color.g(),
                    plot_theme.text_color.b()
                ))
            )
            .margin(20)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f64..(categories.len() as f64),
                y_min..(y_max + y_padding)
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to create chart: {}", e)))?;
        
        // Configure chart
        let axis_color = RGBColor(plot_theme.axis_color.r(), plot_theme.axis_color.g(), plot_theme.axis_color.b());
        let grid_color = RGBColor(plot_theme.grid_color.r(), plot_theme.grid_color.g(), plot_theme.grid_color.b());
        
        chart
            .configure_mesh()
            .x_desc(plot_config.x_label.as_deref().unwrap_or(category_column))
            .y_desc(plot_config.y_label.as_deref().unwrap_or(value_column))
            .axis_style(&axis_color)
            .light_line_style(&grid_color)
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < categories.len() {
                    categories[idx].clone()
                } else {
                    String::new()
                }
            })
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to configure mesh: {}", e)))?;
        
        // Plot bars
        let primary_color = plot_theme.categorical_color(0);
        let bar_color = RGBColor(primary_color.r(), primary_color.g(), primary_color.b());
        
        chart
            .draw_series(
                values.iter().enumerate().map(|(i, &value)| {
                    Rectangle::new([(i as f64 - 0.4, 0.0), (i as f64 + 0.4, value)], bar_color.filled())
                })
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw bars: {}", e)))?
            .label("Values")
            .legend(move |(x, y)| Rectangle::new([(x, y), (x + 10, y + 10)], bar_color.filled()));
        
        chart
            .configure_series_labels()
            .background_style(&RGBColor(255, 255, 255).mix(0.8))
            .border_style(&axis_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw legend: {}", e)))?;
        
        Ok(())
    }
    
    async fn render_histogram_plot<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        data: &RecordBatch,
        column: &str,
        num_bins: usize,
        plot_config: &PlotConfig,
        plot_theme: &PlotTheme,
    ) -> Result<()>
    where
        DB::ErrorType: 'static + std::error::Error + Send + Sync,
    {
        // Extract data
        let array = data.column_by_name(column)
            .ok_or_else(|| pika_core::error::PikaError::Validation(
                format!("Column '{}' not found", column)
            ))?;
        
        let values = extract_numeric_values(array)
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to extract values: {}", e)))?;
        
        // Calculate histogram
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let bin_width = (max_val - min_val) / num_bins as f64;
        
        let mut bins = vec![0; num_bins];
        for &value in &values {
            let bin_index = ((value - min_val) / bin_width).floor() as usize;
            let bin_index = bin_index.min(num_bins - 1);
            bins[bin_index] += 1;
        }
        
        let max_count = *bins.iter().max().unwrap_or(&0) as f64;
        
        // Create chart
        let mut chart = ChartBuilder::on(root)
            .caption(
                plot_config.title.as_deref().unwrap_or("Histogram"),
                ("Arial", 30).into_font().color(&RGBColor(
                    plot_theme.text_color.r(),
                    plot_theme.text_color.g(),
                    plot_theme.text_color.b()
                ))
            )
            .margin(20)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                min_val..max_val,
                0f64..(max_count * 1.1)
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to create chart: {}", e)))?;
        
        // Configure chart
        let axis_color = RGBColor(plot_theme.axis_color.r(), plot_theme.axis_color.g(), plot_theme.axis_color.b());
        let grid_color = RGBColor(plot_theme.grid_color.r(), plot_theme.grid_color.g(), plot_theme.grid_color.b());
        
        chart
            .configure_mesh()
            .x_desc(plot_config.x_label.as_deref().unwrap_or(column))
            .y_desc("Frequency")
            .axis_style(&axis_color)
            .light_line_style(&grid_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to configure mesh: {}", e)))?;
        
        // Plot histogram bars
        let primary_color = plot_theme.categorical_color(0);
        let bar_color = RGBColor(primary_color.r(), primary_color.g(), primary_color.b());
        
        chart
            .draw_series(
                bins.iter().enumerate().map(|(i, &count)| {
                    let x_start = min_val + i as f64 * bin_width;
                    let x_end = x_start + bin_width;
                    Rectangle::new([(x_start, 0.0), (x_end, count as f64)], bar_color.filled())
                })
            )
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw histogram: {}", e)))?
            .label("Frequency")
            .legend(move |(x, y)| Rectangle::new([(x, y), (x + 10, y + 10)], bar_color.filled()));
        
        chart
            .configure_series_labels()
            .background_style(&RGBColor(255, 255, 255).mix(0.8))
            .border_style(&axis_color)
            .draw()
            .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to draw legend: {}", e)))?;
        
        Ok(())
    }
}

/// Convenience function to export a plot with default settings
pub async fn export_plot_to_file(
    plot_config: &PlotConfig,
    data: &RecordBatch,
    output_path: &Path,
    theme_mode: ThemeMode,
) -> Result<()> {
    let format = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(PlotExportFormat::from_extension)
        .unwrap_or(PlotExportFormat::Png);
    
    let export_config = PlotExportConfig {
        width: plot_config.width,
        height: plot_config.height,
        format,
        theme_mode,
        ..Default::default()
    };
    
    let exporter = PlotExporter::new(export_config);
    exporter.export_plot(plot_config, data, output_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_export_format_detection() {
        assert!(matches!(PlotExportFormat::from_extension("png"), Some(PlotExportFormat::Png)));
        assert!(matches!(PlotExportFormat::from_extension("PNG"), Some(PlotExportFormat::Png)));
        assert!(matches!(PlotExportFormat::from_extension("svg"), Some(PlotExportFormat::Svg)));
        assert!(matches!(PlotExportFormat::from_extension("pdf"), Some(PlotExportFormat::Pdf)));
        assert!(PlotExportFormat::from_extension("txt").is_none());
    }
    
    #[test]
    fn test_export_config_default() {
        let config = PlotExportConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(matches!(config.format, PlotExportFormat::Png));
        assert!(matches!(config.theme_mode, ThemeMode::Light));
    }
    
    #[tokio::test]
    async fn test_plot_export_creation() {
        let config = PlotExportConfig::default();
        let exporter = PlotExporter::new(config);
        
        // Test that we can create an exporter
        assert_eq!(exporter.config.width, 800);
    }
} 