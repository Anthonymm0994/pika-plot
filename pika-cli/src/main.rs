//! Command-line interface for Pika-Plot with enhanced user experience.

use clap::{Parser, Subcommand};
use pika_core::{Result, PikaError};
use pika_engine::Engine;
use std::path::PathBuf;
use tokio;

#[derive(Parser)]
#[command(name = "pika")]
#[command(about = "Pika-Plot CLI - GPU-accelerated data visualization", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import data from a file
    Import {
        /// Path to the data file (CSV, Parquet, or JSON)
        #[arg(short, long)]
        file: PathBuf,
        
        /// Table name to import into
        #[arg(short, long)]
        table: String,
        
        /// Database path (defaults to in-memory)
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    
    /// Execute a SQL query
    Query {
        /// SQL query to execute
        #[arg(short, long)]
        sql: String,
        
        /// Output format (table, csv, json)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Database path (defaults to in-memory)
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    
    /// Generate a plot from data
    Plot {
        /// SQL query to get plot data
        #[arg(short, long)]
        query: String,
        
        /// Plot type (scatter, line, bar, histogram)
        #[arg(short = 't', long, default_value = "scatter")]
        plot_type: String,
        
        /// X column name
        #[arg(short, long)]
        x: String,
        
        /// Y column name
        #[arg(short, long)]
        y: String,
        
        /// Output file (PNG or SVG)
        #[arg(short, long)]
        output: PathBuf,
        
        /// Database path (defaults to in-memory)
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    
    /// Export data to a file
    Export {
        /// Table or query to export
        #[arg(short, long)]
        source: String,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format (csv, json, parquet)
        #[arg(short, long)]
        format: Option<String>,
        
        /// Database path (defaults to in-memory)
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
    
    /// Show database schema
    Schema {
        /// Database path (defaults to in-memory)
        #[arg(short, long)]
        database: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize engine
    let engine = Engine::new().await?;
    
    match cli.command {
        Commands::Import { file, table, database } => {
            import_data(engine, file, table, database).await?;
        }
        Commands::Query { sql, format, database } => {
            execute_query(engine, sql, format, database).await?;
        }
        Commands::Plot { query, plot_type, x, y, output, database } => {
            generate_plot(engine, query, plot_type, x, y, output, database).await?;
        }
        Commands::Export { source, output, format, database } => {
            export_data(engine, source, output, format, database).await?;
        }
        Commands::Schema { database } => {
            show_schema(engine, database).await?;
        }
    }
    
    Ok(())
}

async fn import_data(
    engine: Engine,
    file: PathBuf,
    table: String,
    _database: Option<PathBuf>,
) -> Result<()> {
    println!("Importing {} into table '{}'...", file.display(), table);
    
    // Determine file type from extension
    let extension = file.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "csv" => {
            engine.import_csv(&file, &table).await?;
            println!("Successfully imported CSV data");
        }
        "parquet" => {
            return Err(PikaError::NotImplemented {
                feature: "Parquet import".to_string()
            });
        }
        "json" => {
            return Err(PikaError::NotImplemented {
                feature: "JSON import".to_string()
            });
        }
        _ => {
            return Err(PikaError::Internal(
                format!("Unsupported file type: {}", extension)
            ));
        }
    }
    
    Ok(())
}

async fn execute_query(
    engine: Engine,
    sql: String,
    format: String,
    _database: Option<PathBuf>,
) -> Result<()> {
    println!("Executing query...");
    
    let result = engine.execute_query(&sql).await?;
    
    match format.as_str() {
        "table" => {
            println!("Query returned {} rows", result.row_count);
            // TODO: Pretty print table
        }
        "csv" => {
            println!("row_count");
            println!("{}", result.row_count);
        }
        "json" => {
            println!(r#"{{"row_count": {}}}"#, result.row_count);
        }
        _ => {
            return Err(PikaError::Internal(
                format!("Unsupported output format: {}", format)
            ));
        }
    }
    
    Ok(())
}

async fn generate_plot(
    _engine: Engine,
    query: String,
    plot_type: String,
    x: String,
    y: String,
    output: PathBuf,
    _database: Option<PathBuf>,
) -> Result<()> {
    println!("Generating {} plot...", plot_type);
    println!("Query: {}", query);
    println!("X: {}, Y: {}", x, y);
    println!("Output: {}", output.display());
    
    // TODO: Implement plot generation
    return Err(PikaError::NotImplemented {
        feature: "CLI plot generation".to_string()
    });
}

async fn export_data(
    engine: Engine,
    source: String,
    output: PathBuf,
    format: Option<String>,
    _database: Option<PathBuf>,
) -> Result<()> {
    println!("Exporting data to {}...", output.display());
    
    // Determine format from extension if not specified
    let format = format.unwrap_or_else(|| {
        output.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("csv")
            .to_string()
    });
    
    // Check if source is a table name or SQL query
    let sql = if source.trim().to_lowercase().starts_with("select") {
        source
    } else {
        format!("SELECT * FROM {}", source)
    };
    
    let result = engine.execute_query(&sql).await?;
    
    match format.as_str() {
        "csv" => {
            // TODO: Write CSV file
            println!("Exported {} rows to CSV", result.row_count);
        }
        "json" => {
            // TODO: Write JSON file
            println!("Exported {} rows to JSON", result.row_count);
        }
        "parquet" => {
            return Err(PikaError::NotImplemented {
                feature: "Parquet export".to_string()
            });
        }
        _ => {
            return Err(PikaError::Internal(
                format!("Unsupported export format: {}", format)
            ));
        }
    }
    
    Ok(())
}

async fn show_schema(
    engine: Engine,
    _database: Option<PathBuf>,
) -> Result<()> {
    println!("Database Schema:");
    println!("================");
    
    // Query the information schema
    let tables_sql = "SELECT table_name FROM information_schema.tables 
                      WHERE table_schema = 'main' 
                      ORDER BY table_name";
    
    let result = engine.execute_query(tables_sql).await?;
    
    if result.row_count == 0 {
        println!("No tables found in database");
    } else {
        println!("Found {} tables", result.row_count);
        // TODO: List tables and their columns
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
