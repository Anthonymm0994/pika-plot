//! Main application implementation.

use crate::{
    panels::{DataPanel, CanvasPanel, PropertiesPanel, StatusBar},
    state::{AppState, ViewMode},
    widgets::{file_import_dialog::FileImportDialog, drag_drop},
};
use eframe::{egui, CreationContext};
use pika_core::events::AppEvent;
use pika_engine::Engine;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::runtime::Handle;
use tokio::sync::broadcast::{Sender, Receiver};
use tracing::{info, error};

/// Main application struct that implements the eframe App trait.
pub struct PikaApp {
    /// Application state
    state: AppState,
    
    /// Shared engine reference
    engine: Arc<RwLock<Engine>>,
    
    /// Tokio runtime handle
    runtime: Handle,
    
    /// Event channels
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,
    
    /// UI panels
    data_panel: DataPanel,
    canvas_panel: CanvasPanel,
    properties_panel: PropertiesPanel,
    status_bar: StatusBar,
    
    /// Dialogs
    file_import_dialog: Option<FileImportDialog>,
    
    /// Context for repainting
    egui_ctx: egui::Context,
}

impl PikaApp {
    /// Create a new app instance.
    pub fn new(
        cc: &CreationContext,
        engine: Arc<RwLock<Engine>>,
        runtime: Handle,
        event_tx: Sender<AppEvent>,
        event_rx: Receiver<AppEvent>,
    ) -> Self {
        info!("Initializing Pika-Plot UI");
        
        // Save context for async repaints
        let egui_ctx = cc.egui_ctx.clone();
        
        // Initialize state
        let state = AppState::new();
        
        // Create UI panels
        let data_panel = DataPanel::new();
        let canvas_panel = CanvasPanel::new(&cc.egui_ctx);
        let properties_panel = PropertiesPanel::new();
        let status_bar = StatusBar::new();
        
        Self {
            state,
            engine,
            runtime,
            event_tx,
            event_rx,
            data_panel,
            canvas_panel,
            properties_panel,
            status_bar,
            file_import_dialog: None,
            egui_ctx,
        }
    }
    
    /// Process events from the engine
    fn process_engine_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::ImportComplete { path, table_info } => {
                    info!("Import complete: {:?}", path);
                    self.state.add_data_node(table_info);
                    self.status_bar.set_message(format!("Imported: {}", path.display()));
                }
                
                AppEvent::ImportProgress { path, progress } => {
                    self.status_bar.set_progress(Some(progress));
                }
                
                AppEvent::ImportError { path, error } => {
                    error!("Import failed for {:?}: {}", path, error);
                    self.status_bar.set_error(format!("Import failed: {}", error));
                }
                
                AppEvent::QueryComplete { id, result } => {
                    match result {
                        Ok(query_result) => {
                            self.state.update_query_result(id, query_result);
                        }
                        Err(e) => {
                            error!("Query failed: {}", e);
                            self.status_bar.set_error(format!("Query failed: {}", e));
                        }
                    }
                }
                
                AppEvent::MemoryWarning { used_mb, available_mb, threshold } => {
                    self.status_bar.update_memory_usage(used_mb, available_mb);
                    match threshold {
                        pika_core::events::MemoryThreshold::Critical |
                        pika_core::events::MemoryThreshold::Severe => {
                            // Show warning dialog
                            self.state.show_memory_warning = true;
                        }
                        _ => {}
                    }
                }
                
                _ => {} // Handle other events as needed
            }
        }
    }
    
    /// Handle file drops
    fn handle_file_drops(&mut self, ctx: &egui::Context) {
        // Check for dropped files
        let dropped_files = drag_drop::get_dropped_files(ctx);
        
        if !dropped_files.is_empty() {
            // Filter for supported file types
            let supported_files: Vec<_> = dropped_files.into_iter()
                .filter(|path| {
                    if let Some(ext) = path.extension() {
                        matches!(ext.to_str().map(|s| s.to_lowercase()).as_deref(),
                            Some("csv") | Some("parquet") | Some("json"))
                    } else {
                        false
                    }
                })
                .collect();
            
            if !supported_files.is_empty() {
                // Import the files
                let options = pika_core::types::ImportOptions::default();
                for path in supported_files {
                    self.event_tx.send(AppEvent::ImportCsv { 
                        path: path.clone(), 
                        options: options.clone() 
                    }).ok();
                    
                    self.status_bar.set_message(format!("Importing: {}", path.display()));
                }
            }
        }
        
        // Show visual feedback when hovering with files
        if drag_drop::can_drop_files(ctx) {
            // This will be handled by the canvas or welcome screen
        }
    }
    
    /// Show the main menu bar
    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(28.0)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    // File menu
                    ui.menu_button("File", |ui| {
                        if ui.button("Import Data...").clicked() {
                            self.file_import_dialog = Some(FileImportDialog::new());
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        if ui.button("Save Workspace...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Pika Workspace", &["pika"])
                                .save_file()
                            {
                                if let Err(e) = crate::workspace::save_workspace(&self.state, &path) {
                                    self.status_bar.set_error(format!("Failed to save workspace: {}", e));
                                } else {
                                    self.status_bar.set_success(format!("Workspace saved"));
                                }
                            }
                            ui.close_menu();
                        }
                        
                        if ui.button("Load Workspace...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Pika Workspace", &["pika"])
                                .pick_file()
                            {
                                match crate::workspace::load_workspace(&path) {
                                    Ok(snapshot) => {
                                        if let Err(e) = crate::workspace::apply_snapshot(&mut self.state, snapshot) {
                                            self.status_bar.set_error(format!("Failed to load workspace: {}", e));
                                        } else {
                                            self.status_bar.set_success(format!("Workspace loaded"));
                                        }
                                    }
                                    Err(e) => {
                                        self.status_bar.set_error(format!("Failed to load workspace: {}", e));
                                    }
                                }
                            }
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        if ui.button("Exit").clicked() {
                            std::process::exit(0);
                        }
                    });
                    
                    // View menu
                    ui.menu_button("View", |ui| {
                        ui.radio_value(&mut self.state.view_mode, ViewMode::Canvas, "Canvas Mode");
                        ui.radio_value(&mut self.state.view_mode, ViewMode::Grid, "Grid Mode");
                        ui.separator();
                        ui.checkbox(&mut self.state.show_properties, "Properties Panel");
                        ui.checkbox(&mut self.state.show_data_panel, "Data Panel");
                    });
                    
                    // Tools menu
                    ui.menu_button("Tools", |ui| {
                        if ui.button("Clear Cache").clicked() {
                            self.event_tx.send(AppEvent::ClearCache {
                                query_cache: true,
                                gpu_cache: true,
                            }).ok();
                            ui.close_menu();
                        }
                        
                        if ui.button("Memory Usage...").clicked() {
                            self.state.show_memory_dialog = true;
                            ui.close_menu();
                        }
                    });
                    
                    // Help menu
                    ui.menu_button("Help", |ui| {
                        if ui.button("Documentation").clicked() {
                            // TODO: Open documentation
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        if ui.button("About").clicked() {
                            self.state.show_about_dialog = true;
                            ui.close_menu();
                        }
                    });
                    
                    // Right-aligned status
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mem_info = self.engine.read().memory_coordinator().get_memory_info();
                        ui.label(format!("Memory: {}/{} MB", 
                            mem_info.used_mb, 
                            mem_info.total_mb
                        ));
                    });
                });
            });
    }
    
    /// Show welcome screen when no data is loaded
    fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        // Handle file drops on welcome screen
        let screen_rect = ui.ctx().screen_rect();
        if let Some(files) = drag_drop::DragDropHandler::new(egui::Id::new("welcome_drop"))
            .handle_drop_area(ui, screen_rect) 
        {
            // Import dropped files
            let options = pika_core::types::ImportOptions::default();
            for path in files {
                self.event_tx.send(AppEvent::ImportCsv { 
                    path: path.clone(), 
                    options: options.clone() 
                }).ok();
            }
        }
        
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            // Logo/Title
            ui.heading(egui::RichText::new("⚡ Pika-Plot").size(48.0));
            ui.label(egui::RichText::new("GPU-Accelerated Data Canvas").size(18.0));
            
            ui.add_space(40.0);
            
            // Quick actions
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("📁 Import Data").size(16.0))
                    .on_hover_text("Import CSV, Parquet, or JSON files")
                    .clicked() 
                {
                    self.file_import_dialog = Some(FileImportDialog::new());
                }
                
                ui.add_space(20.0);
                
                if ui.button(egui::RichText::new("📂 Open Workspace").size(16.0))
                    .on_hover_text("Load a saved workspace")
                    .clicked() 
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Pika Workspace", &["pika"])
                        .pick_file()
                    {
                        match crate::workspace::load_workspace(&path) {
                            Ok(snapshot) => {
                                if let Err(e) = crate::workspace::apply_snapshot(&mut self.state, snapshot) {
                                    self.status_bar.set_error(format!("Failed to load workspace: {}", e));
                                } else {
                                    self.status_bar.set_success(format!("Workspace loaded"));
                                }
                            }
                            Err(e) => {
                                self.status_bar.set_error(format!("Failed to load workspace: {}", e));
                            }
                        }
                    }
                }
            });
            
            ui.add_space(60.0);
            
            // Drag and drop hint
            if drag_drop::can_drop_files(ui.ctx()) {
                ui.group(|ui| {
                    ui.set_min_size(egui::Vec2::new(400.0, 100.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.heading("📥 Drop files here to import");
                        ui.label("Supports CSV, Parquet, and JSON files");
                        ui.add_space(20.0);
                    });
                });
            } else {
                // Tips
                ui.group(|ui| {
                    ui.label(egui::RichText::new("💡 Tips:").strong());
                    ui.label("• Drag and drop files onto the canvas to import");
                    ui.label("• Use Ctrl+Scroll to zoom in/out");
                    ui.label("• Right-click nodes for context menu");
                    ui.label("• Connect nodes to create data pipelines");
                });
            }
        });
    }
}

impl eframe::App for PikaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process engine events first
        self.process_engine_events();
        
        // Handle file drops
        self.handle_file_drops(ctx);
        
        // Show menu bar
        self.show_menu_bar(ctx);
        
        // Show status bar
        self.status_bar.show(ctx);
        
        // Show data panel if enabled
        if self.state.show_data_panel {
            egui::SidePanel::left("data_panel")
                .default_width(250.0)
                .resizable(true)
                .show(ctx, |ui| {
                    self.data_panel.show(ui, &mut self.state, &self.event_tx);
                });
        }
        
        // Show properties panel if enabled
        if self.state.show_properties && self.state.selected_node.is_some() {
            egui::SidePanel::right("properties_panel")
                .default_width(300.0)
                .resizable(true)
                .show(ctx, |ui| {
                    self.properties_panel.show(ui, &mut self.state, &self.event_tx);
                });
        }
        
        // Main canvas/content area
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.data_nodes.is_empty() {
                self.show_welcome_screen(ui);
            } else {
                match self.state.view_mode {
                    ViewMode::Canvas => {
                        self.canvas_panel.show(ui, &mut self.state, &self.event_tx);
                    }
                    ViewMode::Grid => {
                        // TODO: Implement grid view
                        ui.label("Grid view coming soon!");
                    }
                }
            }
        });
        
        // Show dialogs
        if let Some(dialog) = &mut self.file_import_dialog {
            if let Some((paths, options)) = dialog.show(ctx) {
                // Send import events to engine
                for path in paths {
                    self.event_tx.send(AppEvent::ImportCsv { 
                        path: path.clone(), 
                        options: options.clone() 
                    }).ok();
                }
                self.file_import_dialog = None;
            }
        }
        
        // Memory warning dialog
        if self.state.show_memory_warning {
            egui::Window::new("Memory Warning")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("⚠️ Memory usage is critically high!");
                    ui.label("Consider clearing cache or closing unused data nodes.");
                    ui.separator();
                    if ui.button("Clear Cache").clicked() {
                        self.event_tx.send(AppEvent::ClearCache {
                            query_cache: true,
                            gpu_cache: true,
                        }).ok();
                        self.state.show_memory_warning = false;
                    }
                    if ui.button("Dismiss").clicked() {
                        self.state.show_memory_warning = false;
                    }
                });
        }
        
        // About dialog
        if self.state.show_about_dialog {
            egui::Window::new("About Pika-Plot")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("⚡ Pika-Plot");
                    ui.label("GPU-Accelerated Data Canvas");
                    ui.label("Version 0.1.0");
                    ui.separator();
                    ui.label("A notebook-style interface for exploring gigabytes of data");
                    ui.label("with GPU acceleration and intelligent caching.");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.state.show_about_dialog = false;
                    }
                });
        }
        
        // Request repaint if we have ongoing operations
        if self.status_bar.has_progress() {
            ctx.request_repaint();
        }
    }
} 