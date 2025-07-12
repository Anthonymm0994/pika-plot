use pika_core::types::NodeId;
use egui::{Ui, Color32, ScrollArea};

/// Cell types in notebook mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Table,
    Query,
    Plot,
    Markdown,
}

/// A cell in notebook mode
#[derive(Debug, Clone)]
pub struct NotebookCell {
    pub id: NodeId,
    pub cell_type: CellType,
    pub content: String,
    pub execution_count: Option<usize>,
    pub output: Option<String>,
    pub is_collapsed: bool,
}

/// Notebook mode workspace
pub struct NotebookMode {
    cells: Vec<NotebookCell>,
    active_cell: Option<usize>,
    execution_counter: usize,
}

impl NotebookMode {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            active_cell: None,
            execution_counter: 0,
        }
    }
    
    pub fn cells(&self) -> &[NotebookCell] {
        &self.cells
    }
    
    pub fn clear(&mut self) {
        self.cells.clear();
        self.active_cell = None;
    }
    
    pub fn add_cell(&mut self, cell: NotebookCell) {
        self.cells.push(cell);
    }
    
    pub fn render(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Cell").clicked() {
                        self.add_new_cell();
                    }
                    
                    ui.separator();
                    
                    if ui.button("▶️ Run All").clicked() {
                        self.run_all_cells();
                    }
                    
                    if ui.button("🧹 Clear Outputs").clicked() {
                        self.clear_all_outputs();
                    }
                });
                
                ui.separator();
                
                // Render cells
                let mut to_delete = None;
                let mut to_move_up = None;
                let mut to_move_down = None;
                
                for (idx, cell) in self.cells.iter_mut().enumerate() {
                    let is_active = self.active_cell == Some(idx);
                    
                    ui.push_id(idx, |ui| {
                        // Cell container
                        let response = ui.group(|ui| {
                            // Cell header
                            ui.horizontal(|ui| {
                                // Cell type indicator
                                let type_icon = match cell.cell_type {
                                    CellType::Table => "📊",
                                    CellType::Query => "🔍",
                                    CellType::Plot => "📈",
                                    CellType::Markdown => "📝",
                                };
                                ui.label(type_icon);
                                
                                // Execution count
                                if let Some(count) = cell.execution_count {
                                    ui.label(format!("[{}]", count));
                                } else {
                                    ui.label("[ ]");
                                }
                                
                                ui.separator();
                                
                                // Cell controls
                                if ui.small_button("▶").on_hover_text("Run cell").clicked() {
                                    self.run_cell(idx);
                                }
                                
                                if ui.small_button(if cell.is_collapsed { "▼" } else { "▲" })
                                    .on_hover_text("Collapse/Expand")
                                    .clicked() 
                                {
                                    cell.is_collapsed = !cell.is_collapsed;
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✖").on_hover_text("Delete cell").clicked() {
                                        to_delete = Some(idx);
                                    }
                                    
                                    if idx < self.cells.len() - 1 {
                                        if ui.small_button("↓").on_hover_text("Move down").clicked() {
                                            to_move_down = Some(idx);
                                        }
                                    }
                                    
                                    if idx > 0 {
                                        if ui.small_button("↑").on_hover_text("Move up").clicked() {
                                            to_move_up = Some(idx);
                                        }
                                    }
                                });
                            });
                            
                            if !cell.is_collapsed {
                                ui.separator();
                                
                                // Cell content
                                let content_height = match cell.cell_type {
                                    CellType::Markdown => 100.0,
                                    _ => 150.0,
                                };
                                
                                ui.add(
                                    egui::TextEdit::multiline(&mut cell.content)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(5)
                                        .code_editor()
                                        .font(egui::TextStyle::Monospace)
                                );
                                
                                // Cell output
                                if let Some(output) = &cell.output {
                                    ui.separator();
                                    ui.group(|ui| {
                                        ui.set_min_height(50.0);
                                        ui.label("Output:");
                                        ui.code(output);
                                    });
                                }
                            }
                        });
                        
                        // Handle cell selection
                        if response.response.clicked() {
                            self.active_cell = Some(idx);
                        }
                        
                        // Highlight active cell
                        if is_active {
                            ui.painter().rect_stroke(
                                response.response.rect,
                                4.0,
                                egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                            );
                        }
                    });
                    
                    ui.add_space(10.0);
                }
                
                // Handle cell operations
                if let Some(idx) = to_delete {
                    self.cells.remove(idx);
                    if self.active_cell == Some(idx) {
                        self.active_cell = None;
                    }
                }
                
                if let Some(idx) = to_move_up {
                    if idx > 0 {
                        self.cells.swap(idx, idx - 1);
                        if self.active_cell == Some(idx) {
                            self.active_cell = Some(idx - 1);
                        }
                    }
                }
                
                if let Some(idx) = to_move_down {
                    if idx < self.cells.len() - 1 {
                        self.cells.swap(idx, idx + 1);
                        if self.active_cell == Some(idx) {
                            self.active_cell = Some(idx + 1);
                        }
                    }
                }
                
                // Add cell button at the end
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Cell").clicked() {
                        self.add_new_cell();
                    }
                });
            });
    }
    
    fn add_new_cell(&mut self) {
        let cell = NotebookCell {
            id: NodeId::new(),
            cell_type: CellType::Query,
            content: String::new(),
            execution_count: None,
            output: None,
            is_collapsed: false,
        };
        
        self.cells.push(cell);
        self.active_cell = Some(self.cells.len() - 1);
    }
    
    fn run_cell(&mut self, idx: usize) {
        if let Some(cell) = self.cells.get_mut(idx) {
            self.execution_counter += 1;
            cell.execution_count = Some(self.execution_counter);
            
            // Simulate execution
            match cell.cell_type {
                CellType::Query => {
                    cell.output = Some(format!("Query executed: {}", cell.content));
                }
                CellType::Table => {
                    cell.output = Some("Table loaded".to_string());
                }
                CellType::Plot => {
                    cell.output = Some("Plot rendered".to_string());
                }
                CellType::Markdown => {
                    cell.output = None; // Markdown doesn't have output
                }
            }
        }
    }
    
    fn run_all_cells(&mut self) {
        for idx in 0..self.cells.len() {
            self.run_cell(idx);
        }
    }
    
    fn clear_all_outputs(&mut self) {
        for cell in &mut self.cells {
            cell.output = None;
            cell.execution_count = None;
        }
        self.execution_counter = 0;
    }
} 