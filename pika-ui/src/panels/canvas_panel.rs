//! Canvas panel module.

use pika_core::types::NodeId;

#[derive(Debug, Clone)]
pub enum AppEvent {
    NodeSelected(NodeId),
    NodeMoved { id: NodeId, position: egui::Vec2 },
    ConnectionCreated { from: NodeId, to: NodeId },
    NodeDeleted(NodeId),
} 