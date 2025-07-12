use egui::{Ui, Context, Window, Color32, ProgressBar};
use pika_core::memory::MemoryInfo;
use pika_engine::memory::MemoryCoordinator;
use std::sync::Arc;
use parking_lot::RwLock;

/// Memory usage dialog
pub struct MemoryDialog {
    show: bool,
    memory_coordinator: Option<Arc<MemoryCoordinator>>,
    history: Vec<MemorySnapshot>,
    max_history: usize,
}

#[derive(Clone)]
struct MemorySnapshot {
    timestamp: std::time::Instant,
    used_mb: usize,
    total_mb: usize,
    query_cache_mb: usize,
    gpu_cache_mb: usize,
}

impl MemoryDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            memory_coordinator: None,
            history: Vec::new(),
            max_history: 60, // Keep 60 snapshots
        }
    }
    
    pub fn set_memory_coordinator(&mut self, coordinator: Arc<MemoryCoordinator>) {
        self.memory_coordinator = Some(coordinator);
    }
    
    pub fn show(&mut self) {
        self.show = true;
    }
    
    pub fn hide(&mut self) {
        self.show = false;
    }
    
    pub fn update(&mut self, ctx: &Context) -> Option<MemoryAction> {
        if !self.show {
            return None;
        }
        
        let mut action = None;
        
        Window::new("Memory Usage")
            .open(&mut self.show)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                if let Some(coordinator) = &self.memory_coordinator {
                    let info = coordinator.get_memory_info();
                    
                    // Update history
                    self.update_history(&info);
                    
                    // Current usage
                    ui.heading("Current Memory Usage");
                    
                    let usage_percent = (info.used_mb as f32 / info.total_mb as f32).clamp(0.0, 1.0);
                    let color = if usage_percent > 0.9 {
                        Color32::from_rgb(255, 100, 100)
                    } else if usage_percent > 0.7 {
                        Color32::from_rgb(255, 200, 100)
                    } else {
                        Color32::from_rgb(100, 200, 100)
                    };
                    
                    ui.add(
                        ProgressBar::new(usage_percent)
                            .text(format!("{} / {} MB ({:.1}%)", 
                                info.used_mb, 
                                info.total_mb,
                                usage_percent * 100.0
                            ))
                            .fill(color)
                    );
                    
                    ui.add_space(10.0);
                    
                    // Breakdown
                    egui::Grid::new("memory_breakdown")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Available:");
                            ui.label(format!("{} MB", info.available_mb));
                            ui.end_row();
                            
                            ui.label("Query Cache:");
                            ui.label(format!("{} MB", info.query_cache_mb));
                            ui.end_row();
                            
                            ui.label("GPU Cache:");
                            ui.label(format!("{} MB", info.gpu_cache_mb));
                            ui.end_row();
                            
                            ui.label("Other:");
                            let other_mb = info.used_mb.saturating_sub(info.query_cache_mb + info.gpu_cache_mb);
                            ui.label(format!("{} MB", other_mb));
                            ui.end_row();
                        });
                    
                    ui.separator();
                    
                    // Usage graph
                    ui.heading("Usage History");
                    self.render_usage_graph(ui);
                    
                    ui.separator();
                    
                    // Actions
                    ui.heading("Actions");
                    
                    ui.horizontal(|ui| {
                        if ui.button("Clear Query Cache").clicked() {
                            action = Some(MemoryAction::ClearQueryCache);
                        }
                        
                        if ui.button("Clear GPU Cache").clicked() {
                            action = Some(MemoryAction::ClearGpuCache);
                        }
                        
                        if ui.button("Clear All Caches").clicked() {
                            action = Some(MemoryAction::ClearAllCaches);
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    if ui.button("Run Garbage Collection").clicked() {
                        action = Some(MemoryAction::RunGC);
                    }
                    
                    // Memory limit adjustment
                    ui.add_space(10.0);
                    ui.separator();
                    ui.heading("Memory Limit");
                    
                    let mut limit_mb = info.total_mb;
                    ui.horizontal(|ui| {
                        ui.label("Limit:");
                        if ui.add(
                            egui::DragValue::new(&mut limit_mb)
                                .speed(100)
                                .clamp_range(1024..=32768)
                                .suffix(" MB")
                        ).changed() {
                            action = Some(MemoryAction::SetLimit(limit_mb));
                        }
                    });
                } else {
                    ui.label("Memory coordinator not available");
                }
            });
        
        action
    }
    
    fn update_history(&mut self, info: &MemoryInfo) {
        let snapshot = MemorySnapshot {
            timestamp: std::time::Instant::now(),
            used_mb: info.used_mb,
            total_mb: info.total_mb,
            query_cache_mb: info.query_cache_mb,
            gpu_cache_mb: info.gpu_cache_mb,
        };
        
        self.history.push(snapshot);
        
        // Keep only recent history
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
    
    fn render_usage_graph(&self, ui: &mut Ui) {
        use egui::plot::{Plot, PlotPoints, Line, Legend};
        
        if self.history.is_empty() {
            ui.label("No history data yet");
            return;
        }
        
        let plot = Plot::new("memory_history")
            .height(150.0)
            .legend(Legend::default())
            .show_axes([true, true])
            .allow_zoom(false)
            .allow_drag(false);
        
        plot.show(ui, |plot_ui| {
            // Total usage line
            let total_points: PlotPoints = self.history.iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.used_mb as f64])
                .collect();
            
            plot_ui.line(
                Line::new(total_points)
                    .color(Color32::from_rgb(100, 150, 255))
                    .name("Total Used")
            );
            
            // Query cache line
            let cache_points: PlotPoints = self.history.iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.query_cache_mb as f64])
                .collect();
            
            plot_ui.line(
                Line::new(cache_points)
                    .color(Color32::from_rgb(100, 255, 150))
                    .name("Query Cache")
            );
            
            // GPU cache line
            let gpu_points: PlotPoints = self.history.iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.gpu_cache_mb as f64])
                .collect();
            
            plot_ui.line(
                Line::new(gpu_points)
                    .color(Color32::from_rgb(255, 150, 100))
                    .name("GPU Cache")
            );
            
            // Memory limit line
            if let Some(first) = self.history.first() {
                let limit_points: PlotPoints = vec![
                    [0.0, first.total_mb as f64],
                    [self.history.len() as f64, first.total_mb as f64],
                ];
                
                plot_ui.line(
                    Line::new(limit_points)
                        .color(Color32::from_rgb(255, 100, 100))
                        .style(egui::plot::LineStyle::Dashed { length: 10.0 })
                        .name("Limit")
                );
            }
        });
    }
}

/// Actions that can be triggered from the memory dialog
#[derive(Debug, Clone)]
pub enum MemoryAction {
    ClearQueryCache,
    ClearGpuCache,
    ClearAllCaches,
    RunGC,
    SetLimit(usize),
}

/// Memory warning dialog
pub struct MemoryWarningDialog {
    show: bool,
    threshold: pika_core::events::MemoryThreshold,
    used_mb: usize,
    available_mb: usize,
}

impl MemoryWarningDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            threshold: pika_core::events::MemoryThreshold::Normal,
            used_mb: 0,
            available_mb: 0,
        }
    }
    
    pub fn show_warning(
        &mut self,
        threshold: pika_core::events::MemoryThreshold,
        used_mb: usize,
        available_mb: usize,
    ) {
        self.show = true;
        self.threshold = threshold;
        self.used_mb = used_mb;
        self.available_mb = available_mb;
    }
    
    pub fn update(&mut self, ctx: &Context) -> Option<MemoryAction> {
        if !self.show {
            return None;
        }
        
        let mut action = None;
        
        let (title, icon, color) = match self.threshold {
            pika_core::events::MemoryThreshold::Normal => {
                self.show = false; // Don't show for normal
                return None;
            }
            pika_core::events::MemoryThreshold::Warning => {
                ("Memory Warning", "⚠️", Color32::from_rgb(255, 200, 100))
            }
            pika_core::events::MemoryThreshold::Severe => {
                ("Severe Memory Warning", "⚠️", Color32::from_rgb(255, 150, 50))
            }
            pika_core::events::MemoryThreshold::Critical => {
                ("Critical Memory Warning", "🚨", Color32::from_rgb(255, 100, 100))
            }
        };
        
        Window::new(title)
            .open(&mut self.show)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new(icon).size(48.0).color(color));
                    
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new(
                        format!("Memory usage is {}!", self.threshold.as_str())
                    ).size(18.0).strong());
                    
                    ui.add_space(10.0);
                    
                    ui.label(format!("Used: {} MB", self.used_mb));
                    ui.label(format!("Available: {} MB", self.available_mb));
                    
                    ui.add_space(20.0);
                    
                    ui.label("Consider clearing caches or closing unused data nodes.");
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Clear Caches").clicked() {
                            action = Some(MemoryAction::ClearAllCaches);
                            self.show = false;
                        }
                        
                        if ui.button("Open Memory Dialog").clicked() {
                            // This would trigger opening the full memory dialog
                            self.show = false;
                        }
                        
                        if ui.button("Dismiss").clicked() {
                            self.show = false;
                        }
                    });
                });
            });
        
        action
    }
}

impl pika_core::events::MemoryThreshold {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Warning => "high",
            Self::Severe => "very high",
            Self::Critical => "critically high",
        }
    }
} 