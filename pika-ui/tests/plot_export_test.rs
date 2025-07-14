//! Tests for plot export functionality

// NOTE: Export functionality is temporarily disabled in pika-ui
// These tests will be re-enabled once the export module is restored

/*
#[cfg(test)]
mod tests {
    use pika_ui::export::plot_export::{PlotExporter, PlotExportConfig, PlotExportFormat, export_plot_to_file};
    use pika_ui::theme::ThemeMode;
    use pika_core::plots::{PlotConfig, PlotType, PlotDataConfig};
    use arrow::record_batch::RecordBatch;
    use arrow::array::{Float64Array, StringArray, ArrayRef};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio;
    
    /// Create test data for plots
    fn create_test_data() -> RecordBatch {
        let x_values = Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let y_values = Float64Array::from(vec![2.0, 4.0, 3.0, 5.0, 7.0, 6.0, 8.0, 9.0, 10.0, 11.0]);
        let categories = StringArray::from(vec!["A", "B", "A", "B", "A", "B", "A", "B", "A", "B"]);
        
        let schema = Schema::new(vec![
            Field::new("x", DataType::Float64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("category", DataType::Utf8, false),
        ]);
        
        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(x_values) as ArrayRef,
                Arc::new(y_values) as ArrayRef,
                Arc::new(categories) as ArrayRef,
            ],
        ).unwrap()
    }
    
    #[tokio::test]
    async fn test_export_png() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_plot.png");
        
        let plot_config = PlotConfig {
            plot_type: PlotType::Scatter,
            title: Some("Test Scatter Plot".to_string()),
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            data_config: PlotDataConfig::Scatter {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                color_by: None,
                size_by: None,
            },
            ..Default::default()
        };
        
        let data = create_test_data();
        let export_config = PlotExportConfig {
            width: 800,
            height: 600,
            format: PlotExportFormat::Png,
            theme_mode: ThemeMode::Light,
            dpi: 96.0,
            background_color: None,
        };
        
        let exporter = PlotExporter::new(export_config);
        let result = exporter.export_plot(&plot_config, &data, &output_path).await;
        
        assert!(result.is_ok());
        assert!(output_path.exists());
        
        // Verify it's a valid PNG
        let file_data = std::fs::read(&output_path).unwrap();
        assert!(file_data.len() > 0);
        assert_eq!(&file_data[0..8], b"\x89PNG\r\n\x1a\n"); // PNG header
    }
    
    #[tokio::test]
    async fn test_export_svg() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_plot.svg");
        
        let plot_config = PlotConfig {
            plot_type: PlotType::Line,
            title: Some("Test Line Plot".to_string()),
            data_config: PlotDataConfig::Line {
                x_column: "x".to_string(),
                y_columns: vec!["y".to_string()],
                interpolation: None,
            },
            ..Default::default()
        };
        
        let data = create_test_data();
        let export_config = PlotExportConfig {
            format: PlotExportFormat::Svg,
            ..Default::default()
        };
        
        let exporter = PlotExporter::new(export_config);
        let result = exporter.export_plot(&plot_config, &data, &output_path).await;
        
        assert!(result.is_ok());
        assert!(output_path.exists());
        
        // Verify it's a valid SVG
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("</svg>"));
    }
    
    #[tokio::test]
    async fn test_export_different_themes() {
        let temp_dir = TempDir::new().unwrap();
        
        let plot_config = PlotConfig {
            plot_type: PlotType::Bar,
            title: Some("Theme Test Plot".to_string()),
            data_config: PlotDataConfig::Bar {
                category_column: "category".to_string(),
                value_columns: vec!["y".to_string()],
                horizontal: false,
                stacked: false,
            },
            ..Default::default()
        };
        
        let data = create_test_data();
        
        // Test light theme
        let light_path = temp_dir.path().join("light_theme.png");
        let light_config = PlotExportConfig {
            theme_mode: ThemeMode::Light,
            ..Default::default()
        };
        let light_exporter = PlotExporter::new(light_config);
        assert!(light_exporter.export_plot(&plot_config, &data, &light_path).await.is_ok());
        
        // Test dark theme
        let dark_path = temp_dir.path().join("dark_theme.png");
        let dark_config = PlotExportConfig {
            theme_mode: ThemeMode::Dark,
            ..Default::default()
        };
        let dark_exporter = PlotExporter::new(dark_config);
        assert!(dark_exporter.export_plot(&plot_config, &data, &dark_path).await.is_ok());
        
        // Both files should exist and be different
        assert!(light_path.exists());
        assert!(dark_path.exists());
        
        let light_size = std::fs::metadata(&light_path).unwrap().len();
        let dark_size = std::fs::metadata(&dark_path).unwrap().len();
        assert!(light_size > 0);
        assert!(dark_size > 0);
    }
    
    #[tokio::test]
    async fn test_export_different_sizes() {
        let temp_dir = TempDir::new().unwrap();
        
        let plot_config = PlotConfig {
            plot_type: PlotType::Histogram,
            data_config: PlotDataConfig::Histogram {
                column: "y".to_string(),
                num_bins: Some(10),
                bin_strategy: None,
            },
            ..Default::default()
        };
        
        let data = create_test_data();
        
        // Test different sizes
        let sizes = vec![(400, 300), (800, 600), (1200, 900)];
        
        for (width, height) in sizes {
            let output_path = temp_dir.path().join(format!("size_{}x{}.png", width, height));
            let config = PlotExportConfig {
                width,
                height,
                ..Default::default()
            };
            
            let exporter = PlotExporter::new(config);
            assert!(exporter.export_plot(&plot_config, &data, &output_path).await.is_ok());
            assert!(output_path.exists());
        }
    }
    
    #[tokio::test]
    async fn test_export_with_helper_function() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("helper_test.png");
        
        let plot_config = PlotConfig {
            plot_type: PlotType::Scatter,
            title: Some("Helper Function Test".to_string()),
            data_config: PlotDataConfig::Scatter {
                x_column: "x".to_string(),
                y_column: "y".to_string(),
                color_by: Some("category".to_string()),
                size_by: None,
            },
            ..Default::default()
        };
        
        let data = create_test_data();
        
        // Use the helper function
        let result = export_plot_to_file(&plot_config, &data, &output_path, ThemeMode::Light).await;
        
        assert!(result.is_ok());
        assert!(output_path.exists());
    }
    
    #[test]
    fn test_format_detection() {
        assert_eq!(PlotExportFormat::from_extension("png"), Some(PlotExportFormat::Png));
        assert_eq!(PlotExportFormat::from_extension("PNG"), Some(PlotExportFormat::Png));
        assert_eq!(PlotExportFormat::from_extension("svg"), Some(PlotExportFormat::Svg));
        assert_eq!(PlotExportFormat::from_extension("pdf"), Some(PlotExportFormat::Pdf));
        assert_eq!(PlotExportFormat::from_extension("jpg"), None);
        
        assert_eq!(PlotExportFormat::Png.extension(), "png");
        assert_eq!(PlotExportFormat::Svg.extension(), "svg");
        assert_eq!(PlotExportFormat::Pdf.extension(), "pdf");
    }
}
*/ 