use anyhow::Result;
use eframe::egui;
use pika_ui::app::PikaApp;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🎨 Starting Pika-Plot: Excalidraw-style Data Visualization Tool");
    println!("📊 Features: Canvas + Notebook interface, CSV import, Interactive plots");
    
    // Configure the native options for the desktop app
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Pika-Plot - Excalidraw-style Data Visualization"),
        ..Default::default()
    };
    
    // Run the application
    let result = eframe::run_native(
        "Pika-Plot",
        native_options,
        Box::new(|cc| {
            // Set dark theme
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.window_rounding = egui::Rounding::same(6.0);
            cc.egui_ctx.set_style(style);
            
            Ok(Box::new(PikaApp::new(cc)))
        }),
    );
    
    match result {
        Ok(_) => {
            println!("✅ Pika-Plot closed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error running Pika-Plot: {}", e);
            Err(anyhow::anyhow!("Failed to run application: {}", e))
        }
    }
}
