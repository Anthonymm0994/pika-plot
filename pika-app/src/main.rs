//! Pika-Plot main application - GPU-accelerated data canvas.

use pika_core::error::{Result, PikaError};
use pika_ui::PikaApp;
use tracing::{info, error};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pika=debug".parse().unwrap())
                .add_directive("info".parse().unwrap())
        )
        .init();
    
    info!("Starting Pika-Plot...");
    
    // Create eframe native options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1024.0, 768.0])
            .with_title("Pika-Plot - GPU-Accelerated Data Canvas")
            .with_icon(create_pika_icon()),
        ..Default::default()
    };
    
    // Run the UI
    eframe::run_native(
        "Pika-Plot",
        native_options,
        Box::new(|cc| {
            // Create the app
            Ok(Box::new(PikaApp::new(cc)))
        }),
    ).map_err(|e| PikaError::internal(format!("Failed to run application: {}", e)))?;
    
    Ok(())
}

fn create_pika_icon() -> egui::IconData {
    // Create a simple pika icon (yellow lightning bolt style)
    let size = 32u32;
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    
    // Draw a lightning bolt shape
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            
            // Define lightning bolt shape
            let in_bolt = 
                (x >= 10 && x <= 20 && y >= 4 && y <= 12) ||  // Top horizontal
                (x >= 12 && x <= 18 && y >= 12 && y <= 20) || // Middle diagonal
                (x >= 14 && x <= 24 && y >= 20 && y <= 28);   // Bottom horizontal
            
            if in_bolt {
                // Electric yellow
                pixels[idx] = 255;     // R
                pixels[idx + 1] = 235; // G
                pixels[idx + 2] = 59;  // B
                pixels[idx + 3] = 255; // A
            }
        }
    }
    
    egui::IconData {
        rgba: pixels,
        width: size,
        height: size,
    }
}
