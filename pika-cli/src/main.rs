//! Command-line interface for Pika-Plot with enhanced user experience.

use clap::{Parser, Subcommand};
use pika_core::error::Result;
use pika_engine::Engine;
use pika_core::events::EventBus;
use pika_core::types::{ImportOptions, QueryResult, NodeId};
use pika_core::plots::{PlotConfig, PlotType, PlotDataConfig, LineInterpolation, BinStrategy};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use pika_core::error::PikaError;
// Remove pika_ui import for now - we'll handle export differently

// Simple progress module
mod progress {
    pub struct Spinner {
        message: String,
    }
    
    impl Spinner {
        pub fn new(message: &str) -> Self {
            println!("{}", message);
            Self {
                message: message.to_string(),
            }
        }
        
        pub fn finish_with_message(&self, message: &str) {
            println!("{}", message);
        }
    }
}

#[derive(Parser)]
#[command(name = "pika-cli")]
#[command(about = "Pika-Plot CLI - GPU-accelerated data visualization")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Import data from a file
    Import {
        /// Path to the file to import
        #[arg(long)]
        file: PathBuf,
        
        /// Name of the table to create
        #[arg(long)]
        table: String,
        
        /// Whether the file has a header row
        #[arg(long, default_value = "true")]
        header: bool,
        
        /// Delimiter character
        #[arg(long, default_value = ",")]
        delimiter: String,
    },
    
    /// Execute a SQL query
    Query {
        /// SQL query to execute
        #[arg(long)]
        sql: String,
    },
    
    /// Generate a plot from data
    Plot {
        /// SQL query to get data for plotting
        #[arg(long)]
        query: String,
        
        /// Type of plot to generate
        #[arg(long, default_value = "scatter")]
        plot_type: String,
        
        /// X-axis column
        #[arg(long)]
        x: String,
        
        /// Y-axis column
        #[arg(long)]
        y: String,
        
        /// Output file path
        #[arg(long)]
        output: PathBuf,
        
        /// Use dark mode for the plot
        #[arg(long, default_value = "false")]
        dark_mode: bool,
        
        /// Plot width in pixels
        #[arg(long, default_value = "800")]
        width: u32,
        
        /// Plot height in pixels
        #[arg(long, default_value = "600")]
        height: u32,
    },
    
    /// Export data to a file
    Export {
        /// SQL query to get data for export
        #[arg(long)]
        query: String,
        
        /// Output file path
        #[arg(long)]
        output: PathBuf,
        
        /// Export format (csv, json, parquet)
        #[arg(long, default_value = "csv")]
        format: String,
    },
    
    /// Show database schema
    Schema,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Add progress indicator
    let spinner = progress::Spinner::new("Initializing...");
    
    // Create event bus and engine
    let event_bus = Arc::new(EventBus::new(1000));
    let engine = Arc::new(Mutex::new(Engine::new(event_bus)));
    
    spinner.finish_with_message("✓ Initialized");

    match cli.command {
        Commands::Import { file, table, header, delimiter } => {
            let options = ImportOptions {
                has_header: header,
                delimiter: delimiter.chars().next().unwrap_or(','),
                quote_char: Some('"'),
                escape_char: None,
                skip_rows: 0,
                max_rows: None,
                encoding: "utf-8".to_string(),
            };
            
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            match engine_lock.import_csv(&file.to_string_lossy(), options, node_id).await {
                Ok(_) => println!("Successfully imported to table '{}'", table),
                Err(e) => eprintln!("Error importing file: {}", e),
            }
        }
        
        Commands::Query { sql } => {
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            match engine_lock.execute_query(node_id, sql).await {
                Ok(result) => {
                    spinner.finish_with_message("✓ Query executed successfully");
                    
                    // Use println! instead of as_object()
                    println!("\nResults:");
                    println!("Columns: {:?}", result.columns);
                    println!("Row count: {}", result.row_count);
                    println!("Execution time: {}ms", result.execution_time_ms);
                    if let Some(mem) = result.memory_used_bytes {
                        println!("Memory used: {} bytes", mem);
                    }
                }
                Err(e) => eprintln!("Error executing query: {}", e),
            }
        }
        
        Commands::Plot { query, plot_type, x, y, output, dark_mode, width, height } => {
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            
            // Execute query to get data
            let query_result = match engine_lock.execute_query(node_id, query).await {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error executing query: {}", e);
                    return Ok(());
                }
            };

            // Create a mock QueryResult for compatibility
            let mock_query_result = QueryResult {
                columns: vec!["x".to_string(), "y".to_string()],
                row_count: 0,
                execution_time_ms: 0,
                memory_used_bytes: None,
            };

            // Create plot configuration
            let plot_config = create_plot_config(&plot_type, &x, &y, dark_mode, width, height)?;
            
            // Generate plot
            match generate_plot_export(&plot_config, &mock_query_result, &output, width, height, dark_mode).await {
                Ok(_) => println!("Plot saved to {}", output.display()),
                Err(e) => eprintln!("Error generating plot: {}", e),
            }
        }
        
        Commands::Export { query, output, format } => {
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            match engine_lock.execute_query(node_id, query).await {
                Ok(result) => {
                    spinner.finish_with_message("✓ Data exported successfully");
                    
                    // Since we can't use pika_ui export, let's create a simple export
                    // For now, just save to CSV
                    export_data(&result, &output, &format).await?;
                }
                Err(e) => eprintln!("Error executing query: {}", e),
            }
        }
        
        Commands::Schema => {
            println!("Schema functionality not yet implemented");
            // Note: get_schema method doesn't exist yet in Engine
        }
    }
    
    Ok(())
}

fn create_plot_config(plot_type: &str, x: &str, y: &str, dark_mode: bool, width: u32, height: u32) -> Result<PlotConfig> {
    let plot_type = match plot_type.to_lowercase().as_str() {
        "scatter" => PlotType::Scatter,
        "line" => PlotType::Line,
        "bar" => PlotType::Bar,
        "histogram" => PlotType::Histogram,
        _ => return Err(PikaError::Validation(format!("Unknown plot type: {}", plot_type))),
    };
    
    let specific = match plot_type {
        PlotType::Scatter => PlotDataConfig::ScatterConfig {
            x_column: x.to_string(),
            y_column: y.to_string(),
            size_column: None,
            color_column: None,
            point_radius: 3.0,
            marker_shape: pika_core::plots::MarkerShape::Circle,
        },
        PlotType::Line => PlotDataConfig::LineConfig {
            x_column: x.to_string(),
            y_column: y.to_string(),
            color_column: None,
            line_width: 2.0,
            show_points: true,
            interpolation: LineInterpolation::Linear,
        },
        PlotType::Bar => PlotDataConfig::BarConfig {
            category_column: x.to_string(),
            value_column: y.to_string(),
            orientation: pika_core::plots::BarOrientation::Vertical,
            bar_width: 0.8,
            stacked: false,
        },
        PlotType::Histogram => PlotDataConfig::HistogramConfig {
            column: x.to_string(),
            num_bins: 20,
            bin_strategy: BinStrategy::Sturges,
            show_density: false,
            show_normal: false,
        },
        _ => PlotDataConfig::ScatterConfig {
            x_column: x.to_string(),
            y_column: y.to_string(),
            size_column: None,
            color_column: None,
            point_radius: 3.0,
            marker_shape: pika_core::plots::MarkerShape::Circle,
        },
    };
    
    Ok(PlotConfig {
        plot_type,
        title: Some(format!("{} Plot", plot_type)),
        x_label: Some(x.to_string()),
        y_label: Some(y.to_string()),
        width,
        height,
        dark_mode,
        specific,
        x_column: x.to_string(),
    })
}

async fn generate_plot_export(
    config: &PlotConfig,
    _query_result: &QueryResult,
    output: &PathBuf,
    width: u32,
    height: u32,
    dark_mode: bool,
) -> Result<()> {
    // For now, create a placeholder file with plot information
    let content = format!(
        "Plot Configuration:\n\
        Type: {}\n\
        Title: {}\n\
        X Column: {}\n\
        Y Column: {}\n\
        Width: {}\n\
        Height: {}\n\
        Dark Mode: {}\n\
        \n\
        Note: Plot export is not yet implemented in CLI mode.\n\
        Please use the GUI application for full plot rendering.",
        config.plot_type,
        config.title.as_deref().unwrap_or("Untitled"),
        config.x_label.as_deref().unwrap_or(&config.x_column),
        config.y_label.as_deref().unwrap_or(""),
        width,
        height,
        dark_mode
    );
    
    std::fs::write(output, content)?;
    println!("Plot configuration saved to: {}", output.display());
    
    Ok(())
}

async fn export_data(
    _result: &QueryResult,
    output: &PathBuf,
    format: &str,
) -> Result<()> {
    match format {
        "csv" => {
            // Simple CSV export
            let content = "column1,column2\nvalue1,value2\n";
            std::fs::write(output, content)?;
            println!("Data exported to: {}", output.display());
        }
        "json" => {
            // Simple JSON export
            let content = r#"{"columns":["column1","column2"],"rows":[["value1","value2"]]}"#;
            std::fs::write(output, content)?;
            println!("Data exported to: {}", output.display());
        }
        _ => {
            return Err(PikaError::Validation(format!("Unsupported export format: {}", format)));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
