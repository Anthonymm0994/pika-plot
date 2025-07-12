//! Status bar showing system status and progress.

use egui::{Ui, Color32, ProgressBar};
use std::time::{Duration, Instant};

/// Status bar at the bottom of the application.
pub struct StatusBar {
    message: String,
    message_type: MessageType,
    message_time: Instant,
    progress: Option<f32>,
    memory_used: usize,
    memory_total: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum MessageType {
    Info,
    Success,
    Error,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            message_type: MessageType::Info,
            message_time: Instant::now(),
            progress: None,
            memory_used: 0,
            memory_total: 8192, // Default 8GB
        }
    }
    
    pub fn set_message(&mut self, message: String) {
        self.message = message;
        self.message_type = MessageType::Info;
        self.message_time = Instant::now();
    }
    
    pub fn set_error(&mut self, message: String) {
        self.message = message;
        self.message_type = MessageType::Error;
        self.message_time = Instant::now();
    }
    
    pub fn set_success(&mut self, message: String) {
        self.message = message;
        self.message_type = MessageType::Success;
        self.message_time = Instant::now();
    }
    
    pub fn set_progress(&mut self, progress: Option<f32>) {
        self.progress = progress;
    }
    
    pub fn has_progress(&self) -> bool {
        self.progress.is_some()
    }
    
    pub fn update_memory_usage(&mut self, used_mb: usize, total_mb: usize) {
        self.memory_used = used_mb;
        self.memory_total = total_mb;
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Progress bar if active
            if let Some(progress) = self.progress {
                ui.add(
                    ProgressBar::new(progress)
                        .desired_width(150.0)
                        .show_percentage()
                );
                ui.separator();
            }
            
            // Status message with fade out
            let elapsed = self.message_time.elapsed();
            if elapsed < Duration::from_secs(5) && !self.message.is_empty() {
                let alpha = if elapsed > Duration::from_secs(3) {
                    let fade_time = (elapsed.as_secs_f32() - 3.0) / 2.0;
                    1.0 - fade_time.min(1.0)
                } else {
                    1.0
                };
                
                let color = match self.message_type {
                    MessageType::Info => Color32::from_gray(200),
                    MessageType::Success => Color32::from_rgb(100, 200, 100),
                    MessageType::Error => Color32::from_rgb(200, 100, 100),
                };
                
                let color = Color32::from_rgba_premultiplied(
                    (color.r() as f32 * alpha) as u8,
                    (color.g() as f32 * alpha) as u8,
                    (color.b() as f32 * alpha) as u8,
                    (color.a() as f32 * alpha) as u8,
                );
                
                ui.colored_label(color, &self.message);
            }
            
            // Right-aligned items
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Memory usage
                let memory_percentage = (self.memory_used as f32 / self.memory_total as f32 * 100.0) as i32;
                let memory_color = if memory_percentage > 90 {
                    Color32::from_rgb(200, 100, 100)
                } else if memory_percentage > 70 {
                    Color32::from_rgb(200, 200, 100)
                } else {
                    Color32::from_gray(180)
                };
                
                ui.colored_label(
                    memory_color,
                    format!("💾 {} / {} MB ({}%)", 
                        self.memory_used, 
                        self.memory_total,
                        memory_percentage
                    )
                );
                
                ui.separator();
                
                // GPU status
                ui.label("🎮 GPU Ready");
            });
        });
    }
} 