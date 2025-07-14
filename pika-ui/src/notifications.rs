//! Notification system for displaying user feedback.

use egui::{Context, Color32, Pos2, RichText, Response};
use pika_core::error::PikaError;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: usize,
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
    pub dismissible: bool,
    pub progress: Option<f32>,
}

pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    next_id: usize,
    max_notifications: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            next_id: 0,
            max_notifications: 5,
        }
    }
    
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let notification = Notification {
            id,
            message,
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
            dismissible: true,
            progress: None,
        };
        
        self.notifications.push_back(notification);
        
        // Keep only max notifications
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
        
        id
    }
    
    pub fn add_error(&mut self, error: &PikaError) -> usize {
        self.add_notification(error.to_string(), NotificationType::Error)
    }
    
    pub fn update_progress(&mut self, id: usize, progress: f32) {
        if let Some(notification) = self.notifications.iter_mut().find(|n| n.id == id) {
            notification.progress = Some(progress);
        }
    }
    
    pub fn dismiss(&mut self, id: usize) {
        self.notifications.retain(|n| n.id != id);
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| {
            now.duration_since(n.created_at) < n.duration || !n.dismissible
        });
    }
    
    pub fn show(&mut self, ctx: &Context) {
        let mut to_dismiss = Vec::new();
        
        let window_size = ctx.available_rect().size();
        let mut y_offset = 20.0;
        
        for notification in &self.notifications {
            let response = self.show_notification(ctx, notification, window_size, &mut y_offset);
            if response.map_or(false, |r| r.clicked()) && notification.dismissible {
                to_dismiss.push(notification.id);
            }
        }
        
        for id in to_dismiss {
            self.dismiss(id);
        }
    }
    
    fn show_notification(
        &self,
        ctx: &Context,
        notification: &Notification,
        window_size: egui::Vec2,
        y_offset: &mut f32,
    ) -> Option<Response> {
        let (bg_color, text_color) = match &notification.notification_type {
            NotificationType::Info => (Color32::from_rgb(52, 152, 219), Color32::WHITE),
            NotificationType::Success => (Color32::from_rgb(46, 204, 113), Color32::WHITE),
            NotificationType::Warning => (Color32::from_rgb(241, 196, 15), Color32::BLACK),
            NotificationType::Error => (Color32::from_rgb(231, 76, 60), Color32::WHITE),
        };
        
        let notification_width = 300.0;
        let notification_height = 60.0;
        let x = window_size.x - notification_width - 20.0;
        let y = *y_offset;
        
        let rect = egui::Rect::from_min_size(
            Pos2::new(x, y),
            egui::Vec2::new(notification_width, notification_height),
        );
        
        let response = egui::Window::new(format!("notification_{}", notification.id))
            .fixed_pos(rect.min)
            .fixed_size(rect.size())
            .title_bar(false)
            .frame(egui::Frame::none().fill(bg_color).rounding(5.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.label(RichText::new(&notification.message).color(text_color));
                    
                    if let Some(progress) = notification.progress {
                        ui.add_space(5.0);
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                    }
                });
            });
        
        *y_offset += notification_height + 10.0;
        
        response.and_then(|r| r.inner).and_then(|_| None)
    }
} 