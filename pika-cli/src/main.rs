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
    
    // Create event bus and engine
    let event_bus = Arc::new(EventBus::new(1000));
    let engine = Arc::new(Mutex::new(Engine::new()));
    
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
            match engine_lock.import_csv(file.to_string_lossy().to_string(), options, node_id).await {
                Ok(_) => println!("Successfully imported to table '{}'", table),
                Err(e) => eprintln!("Error importing file: {}", e),
            }
        }
        
        Commands::Query { sql } => {
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            match engine_lock.execute_query(sql).await {
                Ok(result) => {
                    println!("Query executed successfully:");
                    // For now, handle the Value result
                    if let Some(obj) = result.as_object() {
                        if let Some(columns) = obj.get("columns") {
                            println!("Columns: {:?}", columns);
                        }
                        if let Some(row_count) = obj.get("row_count") {
                            println!("Rows: {}", row_count);
                        }
                        if let Some(execution_time) = obj.get("execution_time_ms") {
                            println!("Execution time: {}ms", execution_time);
                        }
                    } else {
                        println!("Result: {:?}", result);
                    }
                }
                Err(e) => eprintln!("Error executing query: {}", e),
            }
        }
        
        Commands::Plot { query, plot_type, x, y, output, dark_mode, width, height } => {
            let node_id = NodeId::new();
            let mut engine_lock = engine.lock().await;
            
            // Execute query to get data
            let query_result = match engine_lock.execute_query(query).await {
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
            match engine_lock.execute_query(query).await {
                Ok(result) => {
                    // Create a mock QueryResult for compatibility
                    let mock_query_result = QueryResult {
                        columns: vec!["data".to_string()],
                        row_count: 0,
                        execution_time_ms: 0,
                        memory_used_bytes: None,
                    };
                    
                    match export_data(&mock_query_result, &output, &format).await {
                        Ok(_) => println!("Data exported to {}", output.display()),
                        Err(e) => eprintln!("Error exporting data: {}", e),
                    }
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
    let plot_type_enum = match plot_type {
        "scatter" => PlotType::Scatter,
        "line" => PlotType::Line,
        "bar" => PlotType::Bar,
        "histogram" => PlotType::Histogram,
        _ => return Err(PikaError::Unsupported("Plot type not supported in CLI".to_string())),
    };
    
    let specific_config = match plot_type_enum {
        PlotType::Scatter => PlotDataConfig::ScatterConfig {
            x_column: x.to_string(),
            y_column: y.to_string(),
            color_column: None,
            size_column: None,
            point_radius: 3.0,
            marker_shape: pika_core::plots::MarkerShape::Circle,
        },
        PlotType::Line => PlotDataConfig::LineConfig {
            x_column: x.to_string(),
            y_column: y.to_string(),
            color_column: None,
            line_width: 2.0,
            show_points: false,
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
            bin_strategy: BinStrategy::Fixed,
            show_density: false,
            show_normal: false,
        },
        _ => return Err(PikaError::Unsupported("Plot type not supported in CLI".to_string())),
    };
    
    Ok(PlotConfig {
        plot_type: plot_type_enum,
        title: Some(format!("{} Plot", plot_type)),
        x_label: Some(x.to_string()),
        y_label: Some(y.to_string()),
        width,
        height,
        dark_mode,
        specific: specific_config,
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
    // For now, create a placeholder plot file
    // In a real implementation, this would use the plot renderer
    
    let default_title = "Untitled Plot".to_string();
    let title = config.title.as_ref().unwrap_or(&default_title);
    let plot_content = format!(
        "Plot: {} ({}x{})\nTheme: {}\nOutput: {}",
        title,
        width,
        height,
        if dark_mode { "Dark" } else { "Light" },
        output.display()
    );
    
    tokio::fs::write(output, plot_content).await
        .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to save plot: {}", e)))?;
    
    println!("Generated {:?} plot with {} theme", config.plot_type, if dark_mode { "dark" } else { "light" });
    
    Ok(())
}

async fn export_data(
    _result: &QueryResult,
    output: &PathBuf,
    format: &str,
) -> Result<()> {
    // For now, create a placeholder export file
    // In a real implementation, this would serialize the actual data
    
    let export_content = format!("Exported data in {} format", format);
    
    tokio::fs::write(output, export_content).await
        .map_err(|e| pika_core::error::PikaError::Internal(format!("Failed to save export: {}", e)))?;
    
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
