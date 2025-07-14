use egui::{Ui, Color32, ScrollArea, RichText, TextEdit, ComboBox, Vec2};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use pika_core::{Result, PikaError};
use pika_core::NodeId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellType {
    Markdown,
    SQL,
    Plot,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CellStatus {
    Ready,
    Running,
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: String,
    pub cell_type: CellType,
    pub content: String,
    pub output: Option<CellOutput>,
    pub execution_count: usize,
    pub status: CellStatus,
    pub created_at: u64,
    pub last_executed: Option<u64>,
    pub metadata: CellMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub output_type: OutputType,
    pub data: OutputData,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputType {
    Text,
    Table,
    Plot,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputData {
    Text(String),
    Table(Vec<Vec<String>>),
    Plot(Vec<u8>),
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellMetadata {
    pub tags: Vec<String>,
    pub collapsed: bool,
    pub read_only: bool,
    pub deletable: bool,
}

impl Default for NotebookCell {
    fn default() -> Self {
        Self {
            id: format!("cell_{}", uuid::Uuid::new_v4()),
            cell_type: CellType::SQL,
            content: String::new(),
            output: None,
            execution_count: 0,
            status: CellStatus::Ready,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_executed: None,
            metadata: CellMetadata::default(),
        }
    }
}

pub struct NotebookMode {
    cells: Vec<NotebookCell>,
    selected_cell: Option<usize>,
    execution_count: usize,
}

impl NotebookMode {
    pub fn new() -> Self {
        Self {
            cells: vec![NotebookCell::default()],
            selected_cell: Some(0),
            execution_count: 0,
        }
    }
    
    pub fn add_cell(&mut self, cell_type: CellType) {
        let mut cell = NotebookCell::default();
        cell.cell_type = cell_type;
        self.cells.push(cell);
        self.selected_cell = Some(self.cells.len() - 1);
    }
    
    pub fn delete_cell(&mut self, index: usize) {
        if index < self.cells.len() && self.cells[index].metadata.deletable {
            self.cells.remove(index);
            if self.selected_cell == Some(index) {
                self.selected_cell = if self.cells.is_empty() {
                    None
                } else {
                    Some(index.min(self.cells.len() - 1))
                };
            }
        }
    }
    
    pub fn move_cell_up(&mut self, index: usize) {
        if index > 0 && index < self.cells.len() {
            self.cells.swap(index, index - 1);
            if self.selected_cell == Some(index) {
                self.selected_cell = Some(index - 1);
            }
        }
    }
    
    pub fn move_cell_down(&mut self, index: usize) {
        if index < self.cells.len() - 1 {
            self.cells.swap(index, index + 1);
            if self.selected_cell == Some(index) {
                self.selected_cell = Some(index + 1);
            }
        }
    }
    
    pub fn execute_cell(&mut self, index: usize) -> Result<()> {
        if index >= self.cells.len() {
            return Err(PikaError::NotFound("Cell not found".to_string()));
        }
        
        self.execution_count += 1;
        let cell = &mut self.cells[index];
        cell.execution_count = self.execution_count;
        cell.status = CellStatus::Running;
        cell.last_executed = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        
        // Simulate execution
        match &cell.cell_type {
            CellType::SQL => {
                let output = CellOutput {
                    output_type: OutputType::Table,
                    data: OutputData::Table(vec![
                        vec!["Column1".to_string(), "Column2".to_string()],
                        vec!["Value1".to_string(), "Value2".to_string()],
                    ]),
                    execution_time_ms: 123.4,
                };
                cell.output = Some(output);
                cell.status = CellStatus::Success;
            }
            CellType::Plot => {
                let output = CellOutput {
                    output_type: OutputType::Plot,
                    data: OutputData::Plot(vec![]),
                    execution_time_ms: 456.7,
                };
                cell.output = Some(output);
                cell.status = CellStatus::Success;
            }
            CellType::Markdown => {
                cell.status = CellStatus::Success;
            }
            CellType::Note => {
                cell.status = CellStatus::Success;
            }
        }
        
        Ok(())
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("📓 Notebook Mode");
        
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("➕ Add Cell").clicked() {
                self.add_cell(CellType::SQL);
            }
            
            ui.separator();
            
            if ui.button("▶️ Run All").clicked() {
                for i in 0..self.cells.len() {
                    let _ = self.execute_cell(i);
                }
            }
            
            if let Some(selected) = self.selected_cell {
                if ui.button("▶️ Run Cell").clicked() {
                    let _ = self.execute_cell(selected);
                }
            }
        });
        
        ui.separator();
        
        // Cells
        ScrollArea::vertical()
            .id_source("notebook_cells")
            .show(ui, |ui| {
                let mut cells_to_delete = Vec::new();
                let mut cell_to_move_up = None;
                let mut cell_to_move_down = None;
                let mut cells_to_execute = Vec::new();
                
                for (index, cell) in self.cells.iter_mut().enumerate() {
                    let is_selected = self.selected_cell == Some(index);
                    
                    ui.group(|ui| {
                        if is_selected {
                            ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_gray(40);
                        }
                        
                        // Cell header
                        ui.horizontal(|ui| {
                            ui.label(format!("[{}]", cell.execution_count));
                            
                            ComboBox::from_id_source(format!("cell_type_{}", index))
                                .selected_text(format!("{:?}", cell.cell_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut cell.cell_type, CellType::Markdown, "Markdown");
                                    ui.selectable_value(&mut cell.cell_type, CellType::SQL, "SQL");
                                    ui.selectable_value(&mut cell.cell_type, CellType::Plot, "Plot");
                                    ui.selectable_value(&mut cell.cell_type, CellType::Note, "Note");
                                });
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🗑️").clicked() {
                                    cells_to_delete.push(index);
                                }
                                if ui.small_button("⬇️").clicked() {
                                    cell_to_move_down = Some(index);
                                }
                                if ui.small_button("⬆️").clicked() {
                                    cell_to_move_up = Some(index);
                                }
                                if ui.small_button("▶️").clicked() {
                                    cells_to_execute.push(index);
                                }
                            });
                        });
                        
                        // Cell content
                        let response = ui.add(
                            TextEdit::multiline(&mut cell.content)
                                .desired_width(f32::INFINITY)
                                .desired_rows(5)
                                .code_editor()
                        );
                        
                        if response.clicked() {
                            self.selected_cell = Some(index);
                        }
                        
                        // Cell output
                        if let Some(output) = &cell.output {
                            ui.separator();
                            match &output.output_type {
                                OutputType::Text => {
                                    if let OutputData::Text(text) = &output.data {
                                        ui.label(text);
                                    }
                                }
                                OutputType::Table => {
                                    if let OutputData::Table(rows) = &output.data {
                                        egui::Grid::new(format!("output_table_{}", index))
                                            .striped(true)
                                            .show(ui, |ui| {
                                                for row in rows {
                                                    for cell in row {
                                                        ui.label(cell);
                                                    }
                                                    ui.end_row();
                                                }
                                            });
                                    }
                                }
                                OutputType::Plot => {
                                    ui.label("📊 Plot output");
                                }
                                OutputType::Error => {
                                    if let OutputData::Error(error) = &output.data {
                                        ui.colored_label(Color32::RED, error);
                                    }
                                }
                            }
                            
                            ui.label(format!("Execution time: {:.2}ms", output.execution_time_ms));
                        }
                    });
                    
                    ui.add_space(10.0);
                }
                
                // Apply actions
                for index in cells_to_delete.into_iter().rev() {
                    self.delete_cell(index);
                }
                
                if let Some(index) = cell_to_move_up {
                    self.move_cell_up(index);
                }
                
                if let Some(index) = cell_to_move_down {
                    self.move_cell_down(index);
                }

                for index in cells_to_execute {
                    let _ = self.execute_cell(index);
                }
            });
    }
}

// Placeholder types for compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInfo {
    pub rows: usize,
    pub columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub summary: String,
} 