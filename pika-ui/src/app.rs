//! Main application implementation.

use crate::{
    panels::{CanvasPanel, DataPanel, PropertiesPanel, StatusBar},
    state::AppState,
    widgets::{FileImportDialog, MemoryDialog},
    shortcuts::ShortcutManager,
    notifications::NotificationManager,
};
use eframe::{egui, App, Frame};
use pika_core::{
    events::{EventBus, AppEvent},
    types::{NodeId, ImportOptions},
};
use pika_engine::Engine;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct PikaApp {
    state: AppState,
    engine: Arc<RwLock<Engine>>,
    event_bus: Arc<EventBus>,
    event_tx: broadcast::Sender<AppEvent>,
    event_rx: broadcast::Receiver<AppEvent>,
    
    // UI components
    canvas_panel: CanvasPanel,
    data_panel: DataPanel,
    properties_panel: PropertiesPanel,
    status_bar: StatusBar,
    
    // Dialogs
    file_import_dialog: Option<FileImportDialog>,
    memory_dialog: Option<MemoryDialog>,
    
    // Managers
    shortcut_manager: ShortcutManager,
    notification_manager: NotificationManager,
    
    // UI state
    show_data_panel: bool,
    show_properties_panel: bool,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl PikaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure custom fonts (simple fallback if font loading fails)
        let ctx = &cc.egui_ctx;
        let mut fonts = egui::FontDefinitions::default();
        
        // Try to load custom font, fallback to default if it fails
        // For now, we'll skip custom fonts to avoid file path issues
        
        ctx.set_fonts(fonts);

        // Create Tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime");

        let event_bus = Arc::new(EventBus::new(1024));

        // Create engine with proper async handling
        let engine = runtime.block_on(async {
            Engine::new(event_bus.clone()).await.expect("Failed to create engine")
        });

        let (event_tx, event_rx) = broadcast::channel(1024);
        
        let canvas_panel = CanvasPanel::new(event_bus.clone());
        
        Self {
            state: AppState::new(),
            engine: Arc::new(RwLock::new(engine)),
            event_bus,
            event_tx,
            event_rx,
            canvas_panel,
            data_panel: DataPanel::new(),
            properties_panel: PropertiesPanel::new(),
            status_bar: StatusBar::new(),
            file_import_dialog: None,
            memory_dialog: None,
            shortcut_manager: ShortcutManager::new(),
            notification_manager: NotificationManager::new(),
            show_data_panel: true,
            show_properties_panel: true,
            runtime: Arc::new(runtime),
        }
    }
    
    fn handle_events(&mut self, ctx: &egui::Context) {
        // Handle shortcuts
        if let Some(action) = self.shortcut_manager.handle_input(ctx) {
            self.handle_shortcut_action(action);
        }
        
        // Handle application events
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::FileOpened(path) => {
                    self.status_bar.set_message(format!("Opened: {}", path));
                }
                AppEvent::ImportCsv { path, options } => {
                    self.handle_csv_import(path, options);
                }
                AppEvent::ImportComplete { path, table_info } => {
                    self.status_bar.set_message(format!("Imported: {}", path));
                    self.state.add_table(table_info);
                }
                AppEvent::QueryComplete { id: _node_id, result } => {
                    // Handle query completion
                    self.status_bar.set_message(format!("Query completed: {} rows", result.row_count));
                }
                AppEvent::ExecuteQuery { node_id, sql: _sql } => {
                    // Execute query
                    self.status_bar.set_message(format!("Executing query for node: {}", node_id));
                }
                AppEvent::MemoryWarning { used, available } => {
                    let used_mb = used / 1024 / 1024;
                    let available_mb = available / 1024 / 1024;
                    self.notification_manager.show_warning(
                        format!("Memory Warning: {}MB used, {}MB available", used_mb, available_mb),
                        Some("Clear Cache".to_string()),
                        Some(Box::new(move || {
                            // Clear cache action
                        }))
                    );
                }
                AppEvent::ClearCache { node_id: _node_id } => {
                    self.status_bar.set_message("Cache cleared".to_string());
                }
                _ => {}
            }
        }
    }
    
    fn handle_shortcut_action(&mut self, action: crate::shortcuts::ShortcutAction) {
        use crate::shortcuts::ShortcutAction;
        
        match action {
            ShortcutAction::ImportData => {
                self.file_import_dialog = Some(FileImportDialog::new());
            }
            ShortcutAction::ToggleDataPanel => {
                self.show_data_panel = !self.show_data_panel;
            }
            ShortcutAction::TogglePropertiesPanel => {
                self.show_properties_panel = !self.show_properties_panel;
            }
            ShortcutAction::ShowMemoryDialog => {
                self.memory_dialog = Some(MemoryDialog::new());
            }
            ShortcutAction::ClearCache => {
                let _ = self.event_tx.send(AppEvent::ClearCache { node_id: None });
            }
            _ => {
                self.status_bar.set_message(format!("Action: {:?}", action));
            }
        }
    }
    
    fn handle_csv_import(&mut self, path: String, options: ImportOptions) {
        let engine = self.engine.clone();
        let event_tx = self.event_tx.clone();
        let path_buf = std::path::PathBuf::from(&path);
        
        tokio::spawn(async move {
            let engine_guard = engine.read().await;
            match engine_guard.import_csv(path_buf, options, NodeId::new()).await {
                Ok(table_info) => {
                    let _ = event_tx.send(AppEvent::ImportComplete {
                        path: path.clone(),
                        table_info,
                    });
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::ImportError {
                        path: path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        });
    }
}

impl App for PikaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.handle_events(ctx);
        
        // Update notification manager
        self.notification_manager.update(ctx);
        
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import CSV").clicked() {
                        self.file_import_dialog = Some(FileImportDialog::new());
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_data_panel, "Data Panel");
                    ui.checkbox(&mut self.show_properties_panel, "Properties Panel");
                });
                
                ui.menu_button("Tools", |ui| {
                    if ui.button("Memory Info").clicked() {
                        self.memory_dialog = Some(MemoryDialog::new());
                        ui.close_menu();
                    }
                    if ui.button("Clear Cache").clicked() {
                        let _ = self.event_tx.send(AppEvent::ClearCache { node_id: None });
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                self.status_bar.show(ui);
            });
        
        // Side panels
        if self.show_data_panel {
            egui::SidePanel::left("data_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.data_panel.show(ui, &mut self.state, &self.event_tx);
                });
        }
        
        if self.show_properties_panel {
            egui::SidePanel::right("properties_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.properties_panel.show(ui, &mut self.state, &self.event_tx);
                });
        }
        
        // Central canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas_panel.show(ctx, ui);
        });
        
        // Handle dialogs
        if let Some(ref mut dialog) = self.file_import_dialog {
            if let Some((file_path, options)) = dialog.show(ctx) {
                let file_name = file_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown");
                
                // Add notification
                self.notification_manager.show_info(
                    format!("Importing {}", file_name),
                    None,
                    None,
                );
                
                // Send import event
                let _ = self.event_tx.send(AppEvent::ImportCsv {
                    path: file_path.to_string_lossy().to_string(),
                    options,
                });
                
                self.file_import_dialog = None;
            }
        }
        
        if let Some(ref mut dialog) = self.memory_dialog {
            let mut open = true;
            dialog.show(ctx, &mut open);
            if !open {
                self.memory_dialog = None;
            }
        }
    }
} 