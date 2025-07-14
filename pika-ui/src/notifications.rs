//! Toast notification system for user feedback and error handling

use pika_core::error::{PikaError, ErrorContext, ErrorSeverity, RecoverySuggestion};
use egui::{Context, Color32, Ui, Vec2, Pos2, Align2, FontId, RichText, Response, Sense};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Notification manager for displaying toast messages
pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    max_notifications: usize,
    position: NotificationPosition,
}

/// Individual notification
pub struct Notification {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub action: Option<NotificationAction>,
    pub created_at: Instant,
    pub duration: Option<Duration>,
    pub dismissible: bool,
}

/// Type of notification
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

/// Action button for notifications
pub struct NotificationAction {
    pub label: String,
    pub callback: Option<Box<dyn Fn() + Send + Sync>>,
}

/// Position for notifications
#[derive(Debug, Clone, Copy)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            max_notifications: 5,
            position: NotificationPosition::TopRight,
        }
    }
    
    pub fn show_info(&mut self, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        self.add_notification(NotificationType::Info, "Info".to_string(), message, action, callback);
    }
    
    pub fn show_success(&mut self, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        self.add_notification(NotificationType::Success, "Success".to_string(), message, action, callback);
    }
    
    pub fn show_warning(&mut self, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        self.add_notification(NotificationType::Warning, "Warning".to_string(), message, action, callback);
    }
    
    pub fn show_error(&mut self, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        self.add_notification(NotificationType::Error, "Error".to_string(), message, action, callback);
    }
    
    pub fn show_critical(&mut self, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        self.add_notification(NotificationType::Critical, "Critical".to_string(), message, action, callback);
    }
    
    fn add_notification(&mut self, notification_type: NotificationType, title: String, message: String, action: Option<String>, callback: Option<Box<dyn Fn() + Send + Sync>>) {
        let notification = Notification {
            id: Uuid::new_v4(),
            notification_type,
            title,
            message,
            action: action.map(|label| NotificationAction {
                label,
                callback,
            }),
            created_at: Instant::now(),
            duration: Some(Duration::from_secs(5)),
            dismissible: true,
        };
        
        self.notifications.push_back(notification);
        
        // Remove excess notifications
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
    }
    
    pub fn update(&mut self, ctx: &Context) {
        // Remove expired notifications
        let now = Instant::now();
        self.notifications.retain(|notification| {
            if let Some(duration) = notification.duration {
                now.duration_since(notification.created_at) < duration
            } else {
                true
            }
        });
        
        // Display notifications
        if !self.notifications.is_empty() {
            self.show_notifications(ctx);
        }
    }
    
    fn show_notifications(&mut self, ctx: &Context) {
        let screen_rect = ctx.screen_rect();
        let mut y_offset = 10.0;
        
        let mut to_remove = Vec::new();
        
        for (index, notification) in self.notifications.iter().enumerate() {
            let pos = match self.position {
                NotificationPosition::TopRight => Pos2::new(screen_rect.max.x - 320.0, y_offset),
                NotificationPosition::TopLeft => Pos2::new(10.0, y_offset),
                NotificationPosition::BottomRight => Pos2::new(screen_rect.max.x - 320.0, screen_rect.max.y - y_offset - 100.0),
                NotificationPosition::BottomLeft => Pos2::new(10.0, screen_rect.max.y - y_offset - 100.0),
            };
            
            let response = self.show_notification(ctx, notification, pos);
            
            if response.clicked() && notification.dismissible {
                to_remove.push(index);
            }
            
            y_offset += 110.0;
        }
        
        // Remove dismissed notifications
        for &index in to_remove.iter().rev() {
            self.notifications.remove(index);
        }
    }
    
    fn show_notification(&self, ctx: &Context, notification: &Notification, pos: Pos2) -> Response {
        let (fill, stroke) = match notification.notification_type {
            NotificationType::Info => (Color32::from_rgb(60, 120, 180), Color32::from_rgb(80, 140, 200)),
            NotificationType::Success => (Color32::from_rgb(60, 150, 80), Color32::from_rgb(80, 170, 100)),
            NotificationType::Warning => (Color32::from_rgb(200, 150, 60), Color32::from_rgb(220, 170, 80)),
            NotificationType::Error => (Color32::from_rgb(180, 60, 60), Color32::from_rgb(200, 80, 80)),
            NotificationType::Critical => (Color32::from_rgb(150, 30, 30), Color32::from_rgb(170, 50, 50)),
        };
        
        egui::Area::new(egui::Id::new(notification.id.to_string()))
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::window(&egui::Style::default())
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0, stroke))
                    .shadow(egui::Shadow {
                        offset: egui::Vec2::new(2.0, 4.0),
                        blur: 8.0,
                        spread: 0.0,
                        color: egui::Color32::from_black_alpha(96),
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(300.0);
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&notification.title).strong().color(Color32::WHITE));
                            
                            if notification.dismissible {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("×").clicked() {
                                        // Will be handled by caller
                                    }
                                });
                            }
                        });
                        
                        ui.label(RichText::new(&notification.message).color(Color32::WHITE));
                        
                        if let Some(ref action) = notification.action {
                            ui.separator();
                            if ui.button(&action.label).clicked() {
                                if let Some(ref callback) = action.callback {
                                    callback();
                                }
                            }
                        }
                    })
            })
            .response
    }
    
    /// Convert PikaError to user-friendly notification
    pub fn show_error_notification(&mut self, error: &PikaError) {
        let (notification_type, title, message, action) = match error {
            PikaError::FileReadError(_msg) => (
                NotificationType::Error,
                "File Access Error".to_string(),
                "Unable to read the selected file. Please check the file path and permissions.".to_string(),
                Some("Retry".to_string()),
            ),
            PikaError::FileWriteError(msg) => (
                NotificationType::Error,
                "File Write Error".to_string(),
                format!("Unable to write file: {}", msg),
                Some("Retry".to_string()),
            ),
            PikaError::Database(db_err) => (
                NotificationType::Error,
                "Database Error".to_string(),
                format!("Database operation failed: {}", db_err),
                Some("Retry".to_string()),
            ),
            PikaError::MemoryLimitExceeded(_msg) => (
                NotificationType::Critical,
                "Memory Limit Exceeded".to_string(),
                "The application is running low on memory. Consider closing other applications or reducing data size.".to_string(),
                Some("Clear Cache".to_string()),
            ),
            PikaError::InvalidPlotConfig(msg) => (
                NotificationType::Warning,
                "Invalid Plot Configuration".to_string(),
                format!("Plot configuration error: {}", msg),
                Some("Fix Config".to_string()),
            ),
            PikaError::CsvImport(_msg) => (
                NotificationType::Error,
                "CSV Import Failed".to_string(),
                "Unable to import CSV file. Please check the file format and encoding.".to_string(),
                Some("Try Again".to_string()),
            ),
            PikaError::QueryExecution(_msg) => (
                NotificationType::Error,
                "Query Failed".to_string(),
                "The database query could not be executed. Please check the SQL syntax.".to_string(),
                Some("Edit Query".to_string()),
            ),
            PikaError::RenderError(_msg) => (
                NotificationType::Warning,
                "Render Error".to_string(),
                "Unable to render the plot. Falling back to simplified rendering.".to_string(),
                Some("Retry".to_string()),
            ),
            PikaError::NotImplemented(feature) => (
                NotificationType::Info,
                "Feature Not Available".to_string(),
                format!("The feature '{}' is not yet implemented.", feature),
                None,
            ),
            PikaError::Internal(msg) => (
                NotificationType::Error,
                "Internal Error".to_string(),
                format!("An internal error occurred: {}", msg),
                Some("Report Bug".to_string()),
            ),
            _ => (
                NotificationType::Error,
                "Error".to_string(),
                error.to_string(),
                Some("Retry".to_string()),
            ),
        };
        
        self.add_notification(notification_type, title, message, action, None);
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notification_creation() {
        let mut manager = NotificationManager::new();
        manager.show_info("Test message".to_string(), None, None);
        assert_eq!(manager.notifications.len(), 1);
        
        let notification = &manager.notifications[0];
        assert_eq!(notification.notification_type, NotificationType::Info);
        assert_eq!(notification.title, "Info");
        assert_eq!(notification.message, "Test message");
    }
    
    #[test]
    fn test_error_notification_creation() {
        let mut manager = NotificationManager::new();
        let error = PikaError::FileReadError("test.csv".to_string());
        manager.show_error_notification(&error);
        
        assert_eq!(manager.notifications.len(), 1);
        
        let notification = &manager.notifications[0];
        assert_eq!(notification.notification_type, NotificationType::Error);
        assert_eq!(notification.title, "File Access Error");
        assert!(notification.message.contains("Unable to read the selected file"));
    }
    
    #[test]
    fn test_notification_capacity() {
        let mut manager = NotificationManager::new();
        manager.max_notifications = 3;
        
        // Add 5 notifications
        for i in 0..5 {
            manager.show_info(format!("Test message {}", i), None, None);
        }
        
        // Should only have 3 notifications (oldest removed)
        assert_eq!(manager.notifications.len(), 3);
        
        // Should have messages 2, 3, 4 (0-indexed)
        assert_eq!(manager.notifications[0].message, "Test message 2");
        assert_eq!(manager.notifications[1].message, "Test message 3");
        assert_eq!(manager.notifications[2].message, "Test message 4");
    }
    
    #[test]
    fn test_different_notification_types() {
        let mut manager = NotificationManager::new();
        
        manager.show_info("Info message".to_string(), None, None);
        manager.show_success("Success message".to_string(), None, None);
        manager.show_warning("Warning message".to_string(), None, None);
        manager.show_error("Error message".to_string(), None, None);
        manager.show_critical("Critical message".to_string(), None, None);
        
        assert_eq!(manager.notifications.len(), 5);
        assert_eq!(manager.notifications[0].notification_type, NotificationType::Info);
        assert_eq!(manager.notifications[1].notification_type, NotificationType::Success);
        assert_eq!(manager.notifications[2].notification_type, NotificationType::Warning);
        assert_eq!(manager.notifications[3].notification_type, NotificationType::Error);
        assert_eq!(manager.notifications[4].notification_type, NotificationType::Critical);
    }
    
    #[test]
    fn test_notification_with_action() {
        let mut manager = NotificationManager::new();
        manager.show_info("Test message".to_string(), Some("Action".to_string()), None);
        
        let notification = &manager.notifications[0];
        assert!(notification.action.is_some());
        assert_eq!(notification.action.as_ref().unwrap().label, "Action");
    }
} 