//! Canvas panel for node-based visual interface.

use egui::{Color32, Vec2};
use pika_core::{
    events::{EventBus, CanvasEvent, NodeEvent},
    nodes::CanvasNode,
    types::NodeId,
};
use crate::canvas::{CanvasState, CanvasWidget};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Canvas panel for the node-based interface
pub struct CanvasPanel {
    /// Canvas state
    canvas_state: CanvasState,
    
    /// Event bus for communication
    event_bus: Arc<EventBus>,
    
    /// Canvas event receiver
    canvas_receiver: broadcast::Receiver<CanvasEvent>,
}

impl CanvasPanel {
    /// Create a new canvas panel
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let canvas_receiver = event_bus.subscribe_canvas_events();
        
        // Create a new broadcast channel for canvas-specific events  
        let (canvas_sender, _) = broadcast::channel(512);
        
        CanvasPanel {
            canvas_state: CanvasState::new(canvas_sender),
            event_bus,
            canvas_receiver,
        }
    }
    
    /// Show the canvas panel
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Process any pending canvas events
        self.process_events();
        
        // Main canvas area
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                // Add toolbar
                self.show_toolbar(ui);
                
                // Canvas widget
                ui.add(CanvasWidget::new(&mut self.canvas_state));
                
                // Show floating controls if needed
                self.show_floating_controls(ctx);
            });
    }
    
    /// Process pending events
    fn process_events(&mut self) {
        // Process canvas events
        while let Ok(event) = self.canvas_receiver.try_recv() {
            match event {
                CanvasEvent::CenterOnNodes { node_ids } => {
                    // TODO: Implement centering logic
                    for node_id in node_ids {
                        if let Some(node) = self.canvas_state.nodes.get(&node_id) {
                            // Center camera on node
                            self.canvas_state.camera.target_center = egui::Pos2::new(
                                node.position.x,
                                node.position.y,
                            );
                        }
                    }
                }
                _ => {
                    // Other events are handled by the canvas widget itself
                }
            }
        }
    }
    
    /// Show toolbar at the top
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
            
            // Zoom controls
            if ui.button("🔍-").on_hover_text("Zoom Out").clicked() {
                self.canvas_state.camera.target_zoom *= 0.8;
            }
            
            ui.label(format!("{:.0}%", self.canvas_state.camera.zoom * 100.0));
            
            if ui.button("🔍+").on_hover_text("Zoom In").clicked() {
                self.canvas_state.camera.target_zoom *= 1.25;
            }
            
            if ui.button("⊡").on_hover_text("Fit to Screen").clicked() {
                self.fit_to_screen();
            }
            
            ui.separator();
            
            // Grid toggle
            let mut show_grid = self.canvas_state.camera.zoom > 0.5;
            if ui.checkbox(&mut show_grid, "Grid").changed() {
                // Grid visibility is automatic based on zoom
            }
            
            // Snap toggle
            if ui.checkbox(&mut self.canvas_state.snap_config.snap_to_nodes, "Snap")
                .on_hover_text("Enable node snapping")
                .changed() {
                // Snap state updated
            }
            
            ui.separator();
            
            // Node creation buttons
            if ui.button("📊 Table").on_hover_text("Create Table Node").clicked() {
                self.create_node_at_center(NodeType::Table);
            }
            
            if ui.button("🔍 Query").on_hover_text("Create Query Node").clicked() {
                self.create_node_at_center(NodeType::Query);
            }
            
            if ui.button("📈 Plot").on_hover_text("Create Plot Node").clicked() {
                self.create_node_at_center(NodeType::Plot);
            }
            
            if ui.button("🔄 Transform").on_hover_text("Create Transform Node").clicked() {
                self.create_node_at_center(NodeType::Transform);
            }
            
            if ui.button("💾 Export").on_hover_text("Create Export Node").clicked() {
                self.create_node_at_center(NodeType::Export);
            }
        });
    }
    
    /// Show floating controls (minimap, etc.)
    fn show_floating_controls(&mut self, ctx: &egui::Context) {
        // Minimap in bottom-right corner
        egui::Window::new("Minimap")
            .id(egui::Id::new("canvas_minimap"))
            .fixed_size(Vec2::new(200.0, 150.0))
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-10.0, -10.0))
            .title_bar(false)
            .resizable(false)
            .show(ctx, |ui| {
                self.draw_minimap(ui);
            });
        
        // Node count indicator
        let node_count = self.canvas_state.nodes.len();
        let connection_count = self.canvas_state.connections.len();
        
        egui::Window::new("Stats")
            .id(egui::Id::new("canvas_stats"))
            .anchor(egui::Align2::LEFT_BOTTOM, Vec2::new(10.0, -10.0))
            .title_bar(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Nodes: {}", node_count));
                    ui.separator();
                    ui.label(format!("Connections: {}", connection_count));
                });
            });
    }
    
    /// Draw minimap
    fn draw_minimap(&self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        
        let rect = response.rect;
        painter.rect_filled(rect, 2.0, Color32::from_gray(30));
        
        // Calculate bounds of all nodes
        if self.canvas_state.nodes.is_empty() {
            return;
        }
        
        let mut min_pos = egui::Pos2::new(f32::INFINITY, f32::INFINITY);
        let mut max_pos = egui::Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for (_, node) in &self.canvas_state.nodes {
            min_pos.x = min_pos.x.min(node.position.x - node.size.x / 2.0);
            min_pos.y = min_pos.y.min(node.position.y - node.size.y / 2.0);
            max_pos.x = max_pos.x.max(node.position.x + node.size.x / 2.0);
            max_pos.y = max_pos.y.max(node.position.y + node.size.y / 2.0);
        }
        
        // Add padding
        let padding = 50.0;
        min_pos -= Vec2::splat(padding);
        max_pos += Vec2::splat(padding);
        
        let world_size = max_pos - min_pos;
        let scale = (rect.size() / world_size).min_elem() * 0.9;
        
        // Draw nodes as dots
        for (_, node) in &self.canvas_state.nodes {
            let world_pos = egui::Pos2::new(node.position.x, node.position.y);
            let minimap_pos = rect.min + ((world_pos - min_pos) * scale);
            
            let color = match node.node_type {
                pika_core::nodes::NodeType::Table(_) => Color32::from_rgb(60, 120, 180),
                pika_core::nodes::NodeType::Query(_) => Color32::from_rgb(180, 120, 60),
                pika_core::nodes::NodeType::Plot(_) => Color32::from_rgb(120, 180, 60),
                pika_core::nodes::NodeType::Transform(_) => Color32::from_rgb(180, 60, 120),
                pika_core::nodes::NodeType::Export(_) => Color32::from_rgb(60, 180, 120),
            };
            
            painter.circle_filled(minimap_pos, 3.0, color);
        }
        
        // Draw viewport rectangle
        let viewport_min = rect.min + ((self.canvas_state.camera.center - min_pos - Vec2::splat(400.0 / self.canvas_state.camera.zoom)) * scale);
        let viewport_max = rect.min + ((self.canvas_state.camera.center - min_pos + Vec2::splat(400.0 / self.canvas_state.camera.zoom)) * scale);
        let viewport_rect = egui::Rect::from_min_max(viewport_min, viewport_max);
        
        painter.rect_stroke(
            viewport_rect,
            2.0,
            egui::Stroke::new(1.0, Color32::from_rgb(255, 200, 100)),
        );
        
        // Handle clicks to jump to location
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative = (pos - rect.min) / scale;
                self.canvas_state.camera.target_center = min_pos + relative;
            }
        }
    }
    
    /// Fit all nodes in view
    fn fit_to_screen(&mut self) {
        if self.canvas_state.nodes.is_empty() {
            return;
        }
        
        let mut min_pos = egui::Pos2::new(f32::INFINITY, f32::INFINITY);
        let mut max_pos = egui::Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for (_, node) in &self.canvas_state.nodes {
            min_pos.x = min_pos.x.min(node.position.x - node.size.x / 2.0);
            min_pos.y = min_pos.y.min(node.position.y - node.size.y / 2.0);
            max_pos.x = max_pos.x.max(node.position.x + node.size.x / 2.0);
            max_pos.y = max_pos.y.max(node.position.y + node.size.y / 2.0);
        }
        
        let center = (min_pos + max_pos.to_vec2()) / 2.0;
        let size = max_pos - min_pos;
        
        self.canvas_state.camera.target_center = center;
        // Assume 800x600 viewport for now
        self.canvas_state.camera.target_zoom = (800.0 / size.x).min(600.0 / size.y) * 0.8;
    }
    
    /// Create a node at the center of the viewport
    fn create_node_at_center(&mut self, node_type: NodeType) {
        // This is a placeholder - actual implementation would create proper node data
        // For now, just demonstrate the pattern
        
        let center = self.canvas_state.camera.center;
        let node_id = NodeId::new();
        
        // Send event to create node
        let _ = self.event_bus.send_node_event(NodeEvent::NodeExecutionStarted { 
            node_id 
        });
    }
}

/// Helper enum for node type selection (simplified)
#[derive(Debug, Clone, Copy)]
enum NodeType {
    Table,
    Query,
    Plot,
    Transform,
    Export,
} 