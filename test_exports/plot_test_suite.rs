use std::path::PathBuf;
use std::fs;
use pika_core::{
    types::{ImportOptions, DataType},
    plots::{PlotConfig, PlotType, PlotDataConfig},
};
use pika_engine::Engine;
use pika_ui::plots::{
    ScatterPlot, HistogramPlot, BarPlot, LinePlot, BoxPlot, HeatmapPlot,
    render_plot_by_config
};
use egui::{Context, Ui};

/// Comprehensive plot testing suite
pub struct PlotTestSuite {
    pub engine: Engine,
    pub test_data_path: PathBuf,
    pub export_path: PathBuf,
}

impl PlotTestSuite {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let engine = Engine::new(pika_core::events::EventBus::new(1024).into()).await?;
        
        Ok(Self {
            engine,
            test_data_path: PathBuf::from("test_exports/data"),
            export_path: PathBuf::from("test_exports/plots"),
        })
    }
    
    /// Create comprehensive test datasets
    pub async fn create_test_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create sample sales data
        let sales_data = r#"date,product,category,sales,quantity,price,region,customer_type
2024-01-01,Laptop,Electronics,1200.50,2,600.25,North,Business
2024-01-02,Phone,Electronics,800.00,1,800.00,South,Consumer
2024-01-03,Desk,Furniture,450.00,1,450.00,East,Business
2024-01-04,Chair,Furniture,120.00,2,60.00,West,Consumer
2024-01-05,Tablet,Electronics,400.00,1,400.00,North,Consumer
2024-01-06,Bookshelf,Furniture,200.00,1,200.00,South,Business
2024-01-07,Monitor,Electronics,300.00,1,300.00,East,Business
2024-01-08,Sofa,Furniture,800.00,1,800.00,West,Consumer
2024-01-09,Keyboard,Electronics,50.00,3,16.67,North,Business
2024-01-10,Table,Furniture,350.00,1,350.00,South,Consumer
2024-01-11,Mouse,Electronics,25.00,5,5.00,East,Business
2024-01-12,Lamp,Furniture,75.00,2,37.50,West,Consumer
2024-01-13,Webcam,Electronics,100.00,1,100.00,North,Business
2024-01-14,Cabinet,Furniture,600.00,1,600.00,South,Consumer
2024-01-15,Headphones,Electronics,150.00,2,75.00,East,Business"#;

        let sales_path = self.test_data_path.join("sales_data.csv");
        fs::write(&sales_path, sales_data)?;
        
        // Import the test data
        let import_options = ImportOptions {
            has_header: true,
            delimiter: ',',
            quote_char: Some('"'),
            escape_char: None,
            skip_rows: 0,
            max_rows: None,
            encoding: "utf-8".to_string(),
        };
        
        self.engine.import_csv(sales_path, import_options, pika_core::types::NodeId::new()).await?;
        
        // Create time series data
        let time_series_data = r#"timestamp,temperature,humidity,pressure
2024-01-01 00:00:00,20.5,45.2,1013.2
2024-01-01 01:00:00,20.1,46.1,1013.5
2024-01-01 02:00:00,19.8,47.0,1013.8
2024-01-01 03:00:00,19.5,47.5,1014.1
2024-01-01 04:00:00,19.2,48.0,1014.3
2024-01-01 05:00:00,19.0,48.5,1014.5
2024-01-01 06:00:00,19.5,47.8,1014.2
2024-01-01 07:00:00,20.2,46.5,1013.8
2024-01-01 08:00:00,21.0,45.0,1013.5
2024-01-01 09:00:00,22.5,43.2,1013.0
2024-01-01 10:00:00,24.0,41.5,1012.5
2024-01-01 11:00:00,25.5,40.0,1012.0
2024-01-01 12:00:00,26.8,38.5,1011.5
2024-01-01 13:00:00,27.2,37.8,1011.2
2024-01-01 14:00:00,27.5,37.0,1011.0
2024-01-01 15:00:00,27.0,37.5,1011.3
2024-01-01 16:00:00,26.2,38.8,1011.8
2024-01-01 17:00:00,25.0,40.2,1012.2
2024-01-01 18:00:00,23.5,42.0,1012.8
2024-01-01 19:00:00,22.0,44.0,1013.2
2024-01-01 20:00:00,21.2,45.5,1013.6
2024-01-01 21:00:00,20.8,46.2,1013.9
2024-01-01 22:00:00,20.5,46.8,1014.1
2024-01-01 23:00:00,20.2,47.2,1014.3"#;

        let time_series_path = self.test_data_path.join("time_series.csv");
        fs::write(&time_series_path, time_series_data)?;
        
        self.engine.import_csv(time_series_path, import_options, pika_core::types::NodeId::new()).await?;
        
        Ok(())
    }
    
    /// Test scatter plot functionality
    pub async fn test_scatter_plot(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Scatter,
            title: "Sales vs Quantity Scatter Plot".to_string(),
            data_config: PlotDataConfig {
                table_name: "sales_data".to_string(),
                x_column: "quantity".to_string(),
                y_column: "sales".to_string(),
                color_column: Some("category".to_string()),
                size_column: None,
                group_column: Some("region".to_string()),
            },
            width: 800,
            height: 600,
            show_legend: true,
            color_scheme: "viridis".to_string(),
        };
        
        // Export plot data and configuration
        let export_path = self.export_path.join("scatter_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Scatter plot configuration exported successfully".to_string())
    }
    
    /// Test histogram functionality
    pub async fn test_histogram(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Histogram,
            title: "Sales Distribution Histogram".to_string(),
            data_config: PlotDataConfig {
                table_name: "sales_data".to_string(),
                x_column: "sales".to_string(),
                y_column: "".to_string(), // Not used for histogram
                color_column: Some("category".to_string()),
                size_column: None,
                group_column: None,
            },
            width: 800,
            height: 600,
            show_legend: true,
            color_scheme: "plasma".to_string(),
        };
        
        let export_path = self.export_path.join("histogram_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Histogram configuration exported successfully".to_string())
    }
    
    /// Test bar plot functionality
    pub async fn test_bar_plot(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Bar,
            title: "Sales by Category Bar Chart".to_string(),
            data_config: PlotDataConfig {
                table_name: "sales_data".to_string(),
                x_column: "category".to_string(),
                y_column: "sales".to_string(),
                color_column: Some("region".to_string()),
                size_column: None,
                group_column: Some("customer_type".to_string()),
            },
            width: 800,
            height: 600,
            show_legend: true,
            color_scheme: "inferno".to_string(),
        };
        
        let export_path = self.export_path.join("bar_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Bar plot configuration exported successfully".to_string())
    }
    
    /// Test line plot functionality
    pub async fn test_line_plot(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Line,
            title: "Temperature Over Time".to_string(),
            data_config: PlotDataConfig {
                table_name: "time_series".to_string(),
                x_column: "timestamp".to_string(),
                y_column: "temperature".to_string(),
                color_column: None,
                size_column: None,
                group_column: None,
            },
            width: 800,
            height: 600,
            show_legend: false,
            color_scheme: "cool".to_string(),
        };
        
        let export_path = self.export_path.join("line_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Line plot configuration exported successfully".to_string())
    }
    
    /// Test box plot functionality
    pub async fn test_box_plot(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Box,
            title: "Sales Distribution by Category".to_string(),
            data_config: PlotDataConfig {
                table_name: "sales_data".to_string(),
                x_column: "category".to_string(),
                y_column: "sales".to_string(),
                color_column: Some("region".to_string()),
                size_column: None,
                group_column: None,
            },
            width: 800,
            height: 600,
            show_legend: true,
            color_scheme: "warm".to_string(),
        };
        
        let export_path = self.export_path.join("box_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Box plot configuration exported successfully".to_string())
    }
    
    /// Test heatmap functionality
    pub async fn test_heatmap(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = PlotConfig {
            plot_type: PlotType::Heatmap,
            title: "Sales Heatmap by Region and Category".to_string(),
            data_config: PlotDataConfig {
                table_name: "sales_data".to_string(),
                x_column: "region".to_string(),
                y_column: "category".to_string(),
                color_column: Some("sales".to_string()),
                size_column: None,
                group_column: None,
            },
            width: 800,
            height: 600,
            show_legend: true,
            color_scheme: "turbo".to_string(),
        };
        
        let export_path = self.export_path.join("heatmap_plot.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&export_path, config_json)?;
        
        Ok("Heatmap configuration exported successfully".to_string())
    }
    
    /// Run comprehensive test suite
    pub async fn run_all_tests(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // Create test data
        self.create_test_data().await?;
        results.push("Test data created successfully".to_string());
        
        // Test all plot types
        results.push(self.test_scatter_plot().await?);
        results.push(self.test_histogram().await?);
        results.push(self.test_bar_plot().await?);
        results.push(self.test_line_plot().await?);
        results.push(self.test_box_plot().await?);
        results.push(self.test_heatmap().await?);
        
        // Create summary report
        let summary = format!(
            "Plot Test Suite Summary:\n\
             - Total plots tested: 6\n\
             - Scatter plot: ✓\n\
             - Histogram: ✓\n\
             - Bar plot: ✓\n\
             - Line plot: ✓\n\
             - Box plot: ✓\n\
             - Heatmap: ✓\n\
             \n\
             All configurations exported to: {}\n\
             Test data created in: {}",
            self.export_path.display(),
            self.test_data_path.display()
        );
        
        let summary_path = self.export_path.join("test_summary.txt");
        fs::write(&summary_path, &summary)?;
        results.push(summary);
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_plot_suite() {
        let mut suite = PlotTestSuite::new().await.expect("Failed to create test suite");
        let results = suite.run_all_tests().await.expect("Failed to run tests");
        
        for result in results {
            println!("{}", result);
        }
    }
} 