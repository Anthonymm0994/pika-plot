//! Pika-Plot main application - GPU-accelerated data canvas.

use pika_core::error::Result;
use pika_engine::Engine;
use pika_ui::PikaApp;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, error};

fn main() -> Result<()> {
    // Initialize logging (similar to frog-viz)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pika=debug".parse().unwrap())
                .add_directive("info".parse().unwrap())
        )
        .init();
    
    info!("Starting Pika-Plot...");
    
    // Create tokio runtime (following frog-viz pattern)
    let runtime = tokio::runtime::Runtime::new()?;
    
    // Create engine with runtime handle
    let engine = runtime.block_on(async {
        Engine::new(
            Some(4 * 1024 * 1024 * 1024), // 4GB default limit
            runtime.handle().clone(),
        ).await
    })?;
    
    let engine = Arc::new(RwLock::new(engine));
    
    // Get event bus and create channels before moving engine
    let (event_tx, event_rx) = {
        let engine = engine.read();
        let event_bus = engine.event_bus();
        // Use the app_events_sender method
        let tx = event_bus.app_events_sender();
        let rx = event_bus.subscribe_app_events();
        (tx, rx)
    };
    
    // Spawn engine background tasks (like frog-viz)
    let engine_handle = engine.clone();
    runtime.spawn(async move {
        loop {
            // Process engine events
            let mut engine = engine_handle.write();
            if let Err(e) = engine.process_events().await {
                error!("Engine error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    
    // Create eframe native options (similar to both frog-viz and pebble)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1024.0, 768.0])
            .with_title("Pika-Plot - GPU-Accelerated Data Canvas")
            .with_icon(create_pika_icon()),
        ..Default::default()
    };
    
    // Run the UI (blocking, like frog-viz)
    eframe::run_native(
        "Pika-Plot",
        native_options,
        Box::new(move |cc| {
            // Apply theme (from pebble pattern)
            pika_ui::theme::apply_theme(&cc.egui_ctx);
            
            Ok(Box::new(PikaApp::new(
                cc,
                engine,
                runtime.handle().clone(),
                event_tx,
                event_rx,
            )))
        }),
    )?;
    
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
