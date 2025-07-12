use egui::{Context, Ui, Color32, ProgressBar};
use std::time::{Duration, Instant};

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_mb: usize,
    pub used_mb: usize,
    pub available_mb: usize,
    pub cache_mb: usize,
}

/// Memory dialog for displaying system memory information
pub struct MemoryDialog {
    last_update: Instant,
    memory_info: MemoryInfo,
    update_interval: Duration,
}

impl MemoryDialog {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            memory_info: MemoryInfo {
                total_mb: 8192,     // Default values
                used_mb: 4096,
                available_mb: 4096,
                cache_mb: 512,
            },
            update_interval: Duration::from_millis(1000),
        }
    }
    
    pub fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new("Memory Information")
            .open(open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.show_content(ui);
            });
    }
    
    fn show_content(&mut self, ui: &mut Ui) {
        // Update memory info periodically
        if self.last_update.elapsed() > self.update_interval {
            self.update_memory_info();
            self.last_update = Instant::now();
        }
        
        ui.heading("System Memory");
        ui.separator();
        
        // Memory usage overview
        ui.horizontal(|ui| {
            ui.label("Total Memory:");
            ui.label(format!("{} MB", self.memory_info.total_mb));
        });
        
        ui.horizontal(|ui| {
            ui.label("Used Memory:");
            ui.label(format!("{} MB", self.memory_info.used_mb));
        });
        
        ui.horizontal(|ui| {
            ui.label("Available Memory:");
            ui.label(format!("{} MB", self.memory_info.available_mb));
        });
        
        ui.horizontal(|ui| {
            ui.label("Cache Memory:");
            ui.label(format!("{} MB", self.memory_info.cache_mb));
        });
        
        ui.separator();
        
        // Memory usage bar
        let usage_ratio = self.memory_info.used_mb as f32 / self.memory_info.total_mb as f32;
        
        ui.horizontal(|ui| {
            ui.label("Usage:");
            let progress = ProgressBar::new(usage_ratio)
                .text(format!("{:.1}%", usage_ratio * 100.0));
            ui.add_sized([200.0, 20.0], progress);
        });
        
        // Warning if memory usage is high
        if usage_ratio > 0.8 {
            ui.colored_label(Color32::YELLOW, "⚠ High memory usage detected");
        }
        
        if usage_ratio > 0.9 {
            ui.colored_label(Color32::RED, "⚠ Critical memory usage!");
        }
        
        ui.separator();
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("Clear Cache").clicked() {
                // Simulate cache clearing
                self.memory_info.cache_mb = 0;
                self.memory_info.used_mb = self.memory_info.used_mb.saturating_sub(512);
                self.memory_info.available_mb = self.memory_info.total_mb - self.memory_info.used_mb;
            }
            
            if ui.button("Refresh").clicked() {
                self.update_memory_info();
            }
        });
    }
    
    fn update_memory_info(&mut self) {
        // In a real implementation, this would query actual system memory
        // For now, simulate some memory usage changes
        use std::process;
        
        // Get basic system info
        if let Ok(output) = process::Command::new("wmic")
            .args(&["OS", "get", "TotalVisibleMemorySize", "/value"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.starts_with("TotalVisibleMemorySize=") {
                        if let Ok(kb) = line.split('=').nth(1).unwrap_or("0").parse::<usize>() {
                            self.memory_info.total_mb = kb / 1024;
                        }
                    }
                }
            }
        }
        
        // Simulate used memory (in real implementation, would get actual values)
        self.memory_info.used_mb = (self.memory_info.total_mb as f32 * 0.6) as usize;
        self.memory_info.available_mb = self.memory_info.total_mb - self.memory_info.used_mb;
    }
}

impl Default for MemoryDialog {
    fn default() -> Self {
        Self::new()
    }
} 