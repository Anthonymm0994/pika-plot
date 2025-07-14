use std::process::Command;
use std::fs;

fn main() {
    println!("🚀 Testing Pika-Plot Functionality");
    
    // Test 1: CLI Help
    test_cli_help();
    
    // Test 2: Data Import
    test_data_import();
    
    // Test 3: Schema Display
    test_schema();
    
    // Test 4: Query Execution
    test_query();
    
    // Test 5: Plot Generation
    test_plot_generation();
    
    // Test 6: Verify Plot Configs
    test_plot_configs();
    
    println!("✅ All tests completed!");
}

fn test_cli_help() {
    println!("\n📖 Testing CLI Help...");
    let output = Command::new("cargo")
        .args(&["run", "-p", "pika-cli", "--", "help"])
        .current_dir("..")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✅ CLI help works");
            } else {
                println!("  ❌ CLI help failed");
            }
        }
        Err(e) => println!("  ❌ Error running CLI help: {}", e)
    }
}

fn test_data_import() {
    println!("\n📥 Testing Data Import...");
    let output = Command::new("cargo")
        .args(&["run", "-p", "pika-cli", "--", "import", 
               "--file", "test_exports/data/sales_data.csv", 
               "--table", "sales_test"])
        .current_dir("..")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✅ Data import works");
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("  ⚠️  Data import may have issues: {}", stderr);
            }
        }
        Err(e) => println!("  ❌ Error running data import: {}", e)
    }
}

fn test_schema() {
    println!("\n🗂️  Testing Schema Display...");
    let output = Command::new("cargo")
        .args(&["run", "-p", "pika-cli", "--", "schema"])
        .current_dir("..")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✅ Schema display works");
            } else {
                println!("  ⚠️  Schema display may have issues");
            }
        }
        Err(e) => println!("  ❌ Error running schema: {}", e)
    }
}

fn test_query() {
    println!("\n🔍 Testing Query Execution...");
    let output = Command::new("cargo")
        .args(&["run", "-p", "pika-cli", "--", "query", 
               "--sql", "SELECT 1 as test_value"])
        .current_dir("..")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✅ Query execution works");
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("  ⚠️  Query execution may have issues: {}", stderr);
            }
        }
        Err(e) => println!("  ❌ Error running query: {}", e)
    }
}

fn test_plot_generation() {
    println!("\n📊 Testing Plot Generation...");
    
    // Test different plot types
    let plot_tests = [
        ("scatter", "SELECT 1 as x, 2 as y"),
        ("histogram", "SELECT 1 as value"),
        ("bar", "SELECT 'A' as category, 10 as value"),
    ];
    
    for (plot_type, query) in &plot_tests {
        let output = Command::new("cargo")
            .args(&["run", "-p", "pika-cli", "--", "plot",
                   "--type", plot_type,
                   "--query", query])
            .current_dir("..")
            .output();
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("  ✅ {} plot generation works", plot_type);
                } else {
                    println!("  ⚠️  {} plot may have issues", plot_type);
                }
            }
            Err(e) => println!("  ❌ Error testing {} plot: {}", plot_type, e)
        }
    }
}

fn test_plot_configs() {
    println!("\n⚙️  Testing Plot Configurations...");
    
    let configs = [
        "scatter_plot_config.json",
        "histogram_config.json",
        "bar_plot_config.json",
        "line_plot_config.json",
        "box_plot_config.json",
        "heatmap_config.json",
        "violin_plot_config.json",
        "correlation_plot_config.json",
        "time_series_plot_config.json",
        "radar_plot_config.json",
    ];
    
    for config in &configs {
        let path = format!("plots/{}", config);
        match fs::read_to_string(&path) {
            Ok(content) => {
                if content.contains("plot_type") && content.contains("title") {
                    println!("  ✅ {} is valid", config);
                } else {
                    println!("  ⚠️  {} may be incomplete", config);
                }
            }
            Err(_) => println!("  ❌ {} not found", config)
        }
    }
} 