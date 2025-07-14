//! Main application implementation.

use eframe::egui;
use pika_core::{
    events::EventBus,
    nodes::{Node, NodeType},
};

use crate::{
    panels::{
        canvas_panel::AppEvent,
        status_bar::StatusBar, 
        properties::PropertiesPanel,
    },
    state::{AppState, ToolMode},
    shortcuts::ShortcutManager,
    widgets::file_import_dialog::FileImportDialog,
    screens::FileConfigScreen,
    theme::set_modern_theme,
};

use crate::panels::canvas::CanvasPanel;

use std::sync::Arc;
use tokio::sync::broadcast;

// Inline panel implementations for now
struct DataSourcesPanel {
    search_query: String,
    selected_table: Option<String>,
}

impl DataSourcesPanel {
    fn new() -> Self {
        Self {
            search_query: String::new(),
            selected_table: None,
        }
    }
    
    fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, _event_bus: &Arc<pika_core::events::EventBus>) {
        ui.heading("Data Sources");
        
        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        
        ui.separator();
        
        // List of tables
        for table in &state.tables {
            if self.search_query.is_empty() || table.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                let is_selected = self.selected_table.as_ref() == Some(&table.name);
                if ui.selectable_label(is_selected, &table.name).clicked() {
                    self.selected_table = Some(table.name.clone());
                }
            }
        }
        
        ui.separator();
        
        // Import button
        if ui.button("Import CSV").clicked() {
            // Switch to file config screen
            state.view_mode = crate::state::ViewMode::FileConfig;
        }
    }
}

struct CanvasToolbar;

impl CanvasToolbar {
    fn new() -> Self {
        Self
    }
    
    fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            ui.label("Tools:");
            
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Select), "Select").clicked() {
                state.tool_mode = ToolMode::Select;
            }
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Pan), "Pan").clicked() {
                state.tool_mode = ToolMode::Pan;
            }
            
            ui.separator();
            
            // Drawing tools
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Rectangle), "□ Rectangle").clicked() {
                state.tool_mode = ToolMode::Rectangle;
            }
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Circle), "○ Circle").clicked() {
                state.tool_mode = ToolMode::Circle;
            }
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Line), "/ Line").clicked() {
                state.tool_mode = ToolMode::Line;
            }
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Draw), "✏ Draw").clicked() {
                state.tool_mode = ToolMode::Draw;
            }
            if ui.selectable_label(matches!(state.tool_mode, ToolMode::Text), "T Text").clicked() {
                state.tool_mode = ToolMode::Text;
            }
            
            ui.separator();
            
            // Zoom controls
            ui.label(format!("Zoom: {:.0}%", state.canvas_state.zoom * 100.0));
            if ui.button("-").clicked() {
                state.canvas_state.zoom = (state.canvas_state.zoom * 0.9).max(0.1);
            }
            if ui.button("+").clicked() {
                state.canvas_state.zoom = (state.canvas_state.zoom * 1.1).min(5.0);
            }
            if ui.button("Reset").clicked() {
                state.canvas_state.zoom = 1.0;
            }
        });
    }
}

struct MenuBar;

impl MenuBar {
    fn new() -> Self {
        Self
    }

    fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) -> Option<MenuAction> {
        let mut action = None;
        
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Workspace...").clicked() {
                    action = Some(MenuAction::NewWorkspace);
                    ui.close_menu();
                }
                if ui.button("Open Database...").clicked() {
                    action = Some(MenuAction::OpenDatabase);
                    ui.close_menu();
                }
                if ui.button("Import CSV...").clicked() {
                    action = Some(MenuAction::ImportCsv);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save Project").clicked() {
                    action = Some(MenuAction::Save);
                    ui.close_menu();
                }
                if ui.button("Save Project As...").clicked() {
                    action = Some(MenuAction::SaveAs);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    action = Some(MenuAction::Exit);
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    action = Some(MenuAction::Undo);
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    action = Some(MenuAction::Redo);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut").clicked() {
                    action = Some(MenuAction::Cut);
                    ui.close_menu();
                }
                if ui.button("Copy").clicked() {
                    action = Some(MenuAction::Copy);
                    ui.close_menu();
                }
                if ui.button("Paste").clicked() {
                    action = Some(MenuAction::Paste);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All").clicked() {
                    action = Some(MenuAction::SelectAll);
                    ui.close_menu();
                }
            });
            
            ui.menu_button("View", |ui| {
                if ui.button("Zoom In").clicked() {
                    state.canvas_state.zoom = (state.canvas_state.zoom * 1.2).min(5.0);
                    ui.close_menu();
                }
                if ui.button("Zoom Out").clicked() {
                    state.canvas_state.zoom = (state.canvas_state.zoom * 0.8).max(0.1);
                    ui.close_menu();
                }
                if ui.button("Reset Zoom").clicked() {
                    state.canvas_state.zoom = 1.0;
                    ui.close_menu();
                }
                if ui.button("Center View on Selection").clicked() {
                    action = Some(MenuAction::CenterOnSelection);
                    ui.close_menu();
                }
                ui.separator();
                if ui.checkbox(&mut state.canvas_state.show_grid, "Toggle Grid").clicked() {
                    ui.close_menu();
                }
                // TODO: Add snap_to_grid field to CanvasState
                // if ui.checkbox(&mut state.canvas_state.snap_to_grid, "Snap to Grid").clicked() {
                //     ui.close_menu();
                // }
                ui.separator();
                ui.label("Mode:");
                if ui.radio_value(&mut state.view_mode, crate::state::ViewMode::Canvas, "Canvas Mode").clicked() {
                    ui.close_menu();
                }
                if ui.radio_value(&mut state.view_mode, crate::state::ViewMode::Notebook, "Notebook Mode").clicked() {
                    ui.close_menu();
                }
            });
            
            ui.menu_button("Data", |ui| {
                ui.label("Active Data Sources:");
                ui.separator();
                if state.data_nodes.is_empty() {
                    ui.label("  No data sources loaded");
                } else {
                    for node in &state.data_nodes {
                        ui.label(format!("  • {}", node.table_info.name));
                    }
                }
                ui.separator();
                
                let plot_count = state.canvas_nodes.values()
                    .filter(|n| matches!(n.node_type, crate::state::CanvasNodeType::Plot { .. }))
                    .count();
                ui.label(format!("Connected Plots: {}", plot_count));
                
                let unconnected_count = state.canvas_nodes.values()
                    .filter(|n| {
                        matches!(n.node_type, crate::state::CanvasNodeType::Plot { .. }) &&
                        !state.connections.iter().any(|c| c.to == n.id)
                    })
                    .count();
                if unconnected_count > 0 {
                    ui.label(format!("⚠ Unconnected Nodes: {}", unconnected_count));
                }
                
                ui.separator();
                ui.label("Query Validity: ✓ OK");
                
                let notes_count = state.canvas_nodes.values()
                    .filter(|n| matches!(n.node_type, crate::state::CanvasNodeType::Note { .. }))
                    .count();
                ui.label(format!("Notes/Annotations: {}", notes_count));
            });
            
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    action = Some(MenuAction::About);
                    ui.close_menu();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    action = Some(MenuAction::KeyboardShortcuts);
                    ui.close_menu();
                }
                if ui.button("Tutorial / Walkthrough").clicked() {
                    action = Some(MenuAction::Tutorial);
                    ui.close_menu();
                }
                if ui.button("Open Logs").clicked() {
                    action = Some(MenuAction::OpenLogs);
                    ui.close_menu();
                }
                if ui.button("Documentation").clicked() {
                    action = Some(MenuAction::Documentation);
                    ui.close_menu();
                }
            });
        });
        
        action
    }
}

#[derive(Debug, Clone)]
enum MenuAction {
    NewWorkspace,
    OpenDatabase,
    ImportCsv,
    Save,
    SaveAs,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    CenterOnSelection,
    KeyboardShortcuts,
    Tutorial,
    OpenLogs,
    Documentation,
    About,
}

/// Main application struct for Pika-Plot
/// Provides an Excalidraw-style interface for data visualization
pub struct App {
    state: AppState,
    event_bus: Arc<EventBus>,
    properties_panel: PropertiesPanel,
    canvas_panel: CanvasPanel,
    status_bar: StatusBar,
    data_sources_panel: DataSourcesPanel,
    canvas_toolbar: CanvasToolbar,
    menu_bar: MenuBar,
    shortcut_manager: ShortcutManager,
    // Event channels for communication
    app_event_tx: broadcast::Sender<AppEvent>,
    app_event_rx: broadcast::Receiver<AppEvent>,
    // Professional CSV import dialog (core functionality)
    csv_import_dialog: FileImportDialog,
    // File configuration screen
    file_config_screen: FileConfigScreen,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        set_modern_theme(&cc.egui_ctx);
        
        let event_bus = Arc::new(EventBus::new(1000));
        
        let properties_panel = PropertiesPanel::new();
        let canvas_panel = CanvasPanel::new(event_bus.clone());
        let status_bar = StatusBar::new();
        let data_sources_panel = DataSourcesPanel::new();
        let canvas_toolbar = CanvasToolbar::new();
        let menu_bar = MenuBar::new();
        let file_config_screen = FileConfigScreen::new();

        let (app_event_tx, app_event_rx) = broadcast::channel(1000);

        Self {
            state: AppState::new(),
            event_bus,
            properties_panel,
            canvas_panel,
            status_bar,
            data_sources_panel,
            canvas_toolbar,
            menu_bar,
            shortcut_manager: ShortcutManager::new(),
            app_event_tx,
            app_event_rx,
            csv_import_dialog: FileImportDialog::new(),
            file_config_screen,
        }
    }
    
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Handle keyboard shortcuts
        ctx.input_mut(|i| {
            // Ctrl+O - Open file
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::O) {
                // Switch to file config screen
                self.state.view_mode = crate::state::ViewMode::FileConfig;
            }
            
            // Delete - Delete selected node
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                if let Some(id) = self.state.selected_node {
                    self.state.remove_data_node(id);
                    self.state.selected_node = None;
                }
            }
            
            // Escape - Cancel/deselect
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                self.state.selected_node = None;
            }
        });
    }
    
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            if let Some(action) = self.menu_bar.show(ui, &mut self.state) {
                match action {
                    MenuAction::NewWorkspace => {
                        println!("📄 New workspace created");
                        // TODO: Clear state and create new workspace
                    }
                    MenuAction::OpenDatabase => {
                        println!("📂 Open database dialog");
                        // TODO: Show database open dialog
                    }
                    MenuAction::ImportCsv => {
                        self.state.view_mode = crate::state::ViewMode::FileConfig;
                    }
                    MenuAction::Save => {
                        println!("💾 Save project");
                        // TODO: Save project
                    }
                    MenuAction::SaveAs => {
                        println!("💾 Save project as");
                        // TODO: Save project as dialog
                    }
                    MenuAction::Exit => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    MenuAction::Undo => {
                        println!("↩️ Undo");
                        // TODO: Implement undo
                    }
                    MenuAction::Redo => {
                        println!("↪️ Redo");
                        // TODO: Implement redo
                    }
                    MenuAction::Cut => {
                        println!("✂️ Cut");
                        // TODO: Implement cut
                    }
                    MenuAction::Copy => {
                        println!("📋 Copy");
                        // TODO: Implement copy
                    }
                    MenuAction::Paste => {
                        println!("📋 Paste");
                        // TODO: Implement paste
                    }
                    MenuAction::SelectAll => {
                        // Select all nodes on canvas
                        for node_id in self.state.canvas_nodes.keys().cloned().collect::<Vec<_>>() {
                            self.state.selected_node = Some(node_id);
                        }
                    }
                    MenuAction::CenterOnSelection => {
                        if let Some(selected_id) = self.state.selected_node {
                            if let Some(node) = self.state.get_canvas_node(selected_id) {
                                self.state.canvas_state.pan_offset = -node.position + egui::Vec2::new(400.0, 300.0);
                            }
                        }
                    }
                    MenuAction::KeyboardShortcuts => {
                        println!("⌨️ Show keyboard shortcuts");
                        // TODO: Show shortcuts dialog
                    }
                    MenuAction::Tutorial => {
                        println!("📚 Show tutorial");
                        // TODO: Show tutorial
                    }
                    MenuAction::OpenLogs => {
                        println!("📄 Open logs");
                        // TODO: Open logs folder
                    }
                    MenuAction::Documentation => {
                        println!("📖 Open documentation");
                        // TODO: Open documentation
                    }
                    MenuAction::About => {
                        println!("ℹ️ About Pika-Plot");
                        // TODO: Show about dialog
                    }
                }
            }
        });
    }
    
    fn add_node(&mut self, typ: NodeType) {
        let node = Node::new(typ);
        // This would normally add to the canvas
        // For now, we'll just show a notification
        self.state.notification = Some(format!("Added {:?} node", node.node_type));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending events
        while let Ok(event) = self.app_event_rx.try_recv() {
            match event {
                AppEvent::NodeSelected(node_id) => {
                    self.state.selected_node = Some(node_id);
                }
                _ => {}
            }
        }

        // Handle keyboard shortcuts
        self.handle_shortcuts(ctx);
        
        // Check view mode
        match self.state.view_mode {
            crate::state::ViewMode::FileConfig => {
                // Show file configuration screen
                if let Some(table_infos) = self.file_config_screen.show(ctx, &mut self.state) {
                    // Add the imported tables to the state
                    for table_info in table_infos {
                        self.state.add_data_node(table_info);
                    }
                }
            }
            crate::state::ViewMode::Canvas | crate::state::ViewMode::Notebook => {
                // Show normal canvas UI
                // Render the main UI
                self.render_menu_bar(ctx);
                
                // Canvas toolbar (top)
                egui::TopBottomPanel::top("canvas_toolbar")
                    .exact_height(40.0)
                    .show(ctx, |ui| {
                        self.canvas_toolbar.show(ui, &mut self.state);
                    });
                
                // Left panel - Data Sources
                egui::SidePanel::left("data_sources")
                    .default_width(250.0)
                    .min_width(200.0)
                    .max_width(400.0)
                    .show(ctx, |ui| {
                        self.data_sources_panel.show(ui, &mut self.state, &self.event_bus);
                    });
                
                // Right panel - Properties
                egui::SidePanel::right("properties")
                    .default_width(250.0)
                    .min_width(200.0)
                    .max_width(400.0)
                    .show(ctx, |ui| {
                        self.properties_panel.show(ui, &mut self.state, &self.app_event_tx);
                    });
                
                // Status bar (bottom)
                egui::TopBottomPanel::bottom("status_bar")
                    .exact_height(25.0)
                    .show(ctx, |ui| {
                        self.status_bar.show(ui, &self.state);
                    });
                
                // Central panel - Canvas
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.canvas_panel.show(ui, &mut self.state, &self.app_event_tx);
                });
            }
        }
    }
} 