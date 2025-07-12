//! Progress indicator widget for long-running operations.

use egui::{Color32, ProgressBar, Response, RichText, Ui, Widget};
use pika_core::types::NodeId;
use std::time::{Duration, Instant};

/// Progress indicator for operations.
pub struct ProgressIndicator {
    /// Current progress (0.0 to 1.0)
    progress: f32,
    /// Optional text to display
    text: Option<String>,
    /// Start time for elapsed time display
    start_time: Option<Instant>,
    /// Whether to show cancel button
    show_cancel: bool,
    /// Node ID associated with this progress
    node_id: Option<NodeId>,
}

impl ProgressIndicator {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            text: None,
            start_time: None,
            show_cancel: false,
            node_id: None,
        }
    }
    
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    
    pub fn with_elapsed_time(mut self, start_time: Instant) -> Self {
        self.start_time = Some(start_time);
        self
    }
    
    pub fn with_cancel_button(mut self) -> Self {
        self.show_cancel = true;
        self
    }
    
    pub fn with_node_id(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }
}

impl Widget for ProgressIndicator {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Main progress bar
            let bar = ProgressBar::new(self.progress)
                .show_percentage()
                .animate(self.progress < 1.0);
            
            let bar_response = if let Some(text) = &self.text {
                bar.text(text).ui(ui)
            } else {
                bar.ui(ui)
            };
            
            // Additional info row
            ui.horizontal(|ui| {
                // Elapsed time
                if let Some(start_time) = self.start_time {
                    let elapsed = start_time.elapsed();
                    let elapsed_text = format_duration(elapsed);
                    ui.label(RichText::new(elapsed_text).small().color(Color32::GRAY));
                    
                    // ETA calculation
                    if self.progress > 0.01 && self.progress < 0.99 {
                        let eta = estimate_remaining_time(elapsed, self.progress);
                        let eta_text = format!("ETA: {}", format_duration(eta));
                        ui.label(RichText::new(eta_text).small().color(Color32::GRAY));
                    }
                }
                
                // Cancel button
                if self.show_cancel {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("❌ Cancel").clicked() {
                            // TODO: Send cancel event
                            if let Some(node_id) = self.node_id {
                                tracing::debug!("Cancel requested for node {:?}", node_id);
                            }
                        }
                    });
                }
            });
            
            bar_response
        })
        .inner
    }
}

/// Format a duration in a human-readable way.
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Estimate remaining time based on progress.
fn estimate_remaining_time(elapsed: Duration, progress: f32) -> Duration {
    if progress <= 0.0 {
        return Duration::from_secs(0);
    }
    
    let total_estimated = elapsed.as_secs_f32() / progress;
    let remaining = total_estimated - elapsed.as_secs_f32();
    Duration::from_secs_f32(remaining.max(0.0))
}

/// A collection of progress indicators for multiple operations.
pub struct ProgressPanel {
    indicators: Vec<(NodeId, ProgressInfo)>,
}

struct ProgressInfo {
    progress: f32,
    text: String,
    start_time: Instant,
    is_cancellable: bool,
}

impl ProgressPanel {
    pub fn new() -> Self {
        Self {
            indicators: Vec::new(),
        }
    }
    
    /// Update or add a progress indicator.
    pub fn update_progress(
        &mut self,
        node_id: NodeId,
        progress: f32,
        text: String,
        is_cancellable: bool,
    ) {
        if let Some(idx) = self.indicators.iter().position(|(id, _)| *id == node_id) {
            self.indicators[idx].1.progress = progress;
            self.indicators[idx].1.text = text;
        } else {
            self.indicators.push((
                node_id,
                ProgressInfo {
                    progress,
                    text,
                    start_time: Instant::now(),
                    is_cancellable,
                },
            ));
        }
        
        // Remove completed non-cancellable operations
        self.indicators.retain(|(_, info)| {
            info.progress < 1.0 || info.is_cancellable
        });
    }
    
    /// Show the progress panel UI.
    pub fn show(&mut self, ui: &mut Ui) {
        if self.indicators.is_empty() {
            return;
        }
        
        ui.group(|ui| {
            ui.label(RichText::new("Active Operations").strong());
            ui.separator();
            
            for (node_id, info) in &self.indicators {
                let mut indicator = ProgressIndicator::new(info.progress)
                    .with_text(&info.text)
                    .with_elapsed_time(info.start_time)
                    .with_node_id(*node_id);
                
                if info.is_cancellable {
                    indicator = indicator.with_cancel_button();
                }
                
                ui.add(indicator);
                ui.add_space(4.0);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(format_duration(Duration::from_secs(7325)), "2h 2m");
    }
    
    #[test]
    fn test_estimate_remaining_time() {
        let elapsed = Duration::from_secs(30);
        let progress = 0.5;
        let remaining = estimate_remaining_time(elapsed, progress);
        assert_eq!(remaining.as_secs(), 30);
    }
} 