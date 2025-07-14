use egui::{Ui, Color32, ScrollArea, RichText, TextEdit, Button, ComboBox};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use pika_core::types::NodeId;
use pika_core::error::Result;

/// Types of notebook cells
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellType {
    Code,
    Markdown,
    Plot,
    Data,
    Analysis,
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::Code => write!(f, "Code"),
            CellType::Markdown => write!(f, "Markdown"),
            CellType::Plot => write!(f, "Plot"),
            CellType::Data => write!(f, "Data"),
            CellType::Analysis => write!(f, "Analysis"),
        }
    }
}

/// Execution status of a cell
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellStatus {
    NotExecuted,
    Running,
    Success,
    Error(String),
}

/// Output from cell execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellOutput {
    Text(String),
    Html(String),
    Plot {
        plot_id: String,
        image_data: Vec<u8>,
        interactive: bool,
    },
    Data {
        table_name: String,
        row_count: usize,
        column_count: usize,
        preview: Vec<Vec<String>>,
    },
    Analysis {
        statistics: HashMap<String, f64>,
        insights: Vec<String>,
        recommendations: Vec<String>,
    },
    Error(String),
}

/// A notebook cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: String,
    pub cell_type: CellType,
    pub content: String,
    pub output: Option<CellOutput>,
    pub status: CellStatus,
    pub created_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
    pub execution_count: usize,
    pub metadata: HashMap<String, String>,
}

impl NotebookCell {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            cell_type,
            content: String::new(),
            output: None,
            status: CellStatus::NotExecuted,
            created_at: Utc::now(),
            last_executed: None,
            execution_count: 0,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
    
    pub fn execute(&mut self, context: &NotebookContext) -> Result<()> {
        self.status = CellStatus::Running;
        self.execution_count += 1;
        
        match self.cell_type {
            CellType::Code => self.execute_code(context),
            CellType::Markdown => self.execute_markdown(),
            CellType::Plot => self.execute_plot(context),
            CellType::Data => self.execute_data(context),
            CellType::Analysis => self.execute_analysis(context),
        }
    }
    
    fn execute_code(&mut self, context: &NotebookContext) -> Result<()> {
        // Execute SQL or other code
        if self.content.trim().to_uppercase().starts_with("SELECT") {
            // SQL query
            match context.execute_sql(&self.content) {
                Ok(result) => {
                    self.output = Some(CellOutput::Data {
                        table_name: "query_result".to_string(),
                        row_count: result.len(),
                        column_count: if result.is_empty() { 0 } else { result[0].len() },
                        preview: result.into_iter().take(10).collect(),
                    });
                    self.status = CellStatus::Success;
                }
                Err(e) => {
                    self.output = Some(CellOutput::Error(e.to_string()));
                    self.status = CellStatus::Error(e.to_string());
                }
            }
        } else {
            // Other code types (placeholder)
            self.output = Some(CellOutput::Text("Code execution not yet implemented".to_string()));
            self.status = CellStatus::Success;
        }
        
        self.last_executed = Some(Utc::now());
        Ok(())
    }
    
    fn execute_markdown(&mut self) -> Result<()> {
        // Render markdown to HTML
        let html = markdown_to_html(&self.content);
        self.output = Some(CellOutput::Html(html));
        self.status = CellStatus::Success;
        self.last_executed = Some(Utc::now());
        Ok(())
    }
    
    fn execute_plot(&mut self, context: &NotebookContext) -> Result<()> {
        // Generate plot based on content
        match context.generate_plot(&self.content) {
            Ok(plot_data) => {
                self.output = Some(CellOutput::Plot {
                    plot_id: format!("plot_{}", self.id),
                    image_data: plot_data,
                    interactive: true,
                });
                self.status = CellStatus::Success;
            }
            Err(e) => {
                self.output = Some(CellOutput::Error(e.to_string()));
                self.status = CellStatus::Error(e.to_string());
            }
        }
        
        self.last_executed = Some(Utc::now());
        Ok(())
    }
    
    fn execute_data(&mut self, context: &NotebookContext) -> Result<()> {
        // Load or process data
        match context.load_data(&self.content) {
            Ok(data_info) => {
                self.output = Some(CellOutput::Data {
                    table_name: data_info.name,
                    row_count: data_info.row_count,
                    column_count: data_info.column_count,
                    preview: data_info.preview,
                });
                self.status = CellStatus::Success;
            }
            Err(e) => {
                self.output = Some(CellOutput::Error(e.to_string()));
                self.status = CellStatus::Error(e.to_string());
            }
        }
        
        self.last_executed = Some(Utc::now());
        Ok(())
    }
    
    fn execute_analysis(&mut self, context: &NotebookContext) -> Result<()> {
        // Perform statistical analysis
        match context.analyze_data(&self.content) {
            Ok(analysis) => {
                self.output = Some(CellOutput::Analysis {
                    statistics: analysis.statistics,
                    insights: analysis.insights,
                    recommendations: analysis.recommendations,
                });
                self.status = CellStatus::Success;
            }
            Err(e) => {
                self.output = Some(CellOutput::Error(e.to_string()));
                self.status = CellStatus::Error(e.to_string());
            }
        }
        
        self.last_executed = Some(Utc::now());
        Ok(())
    }
}

/// Notebook execution context
#[derive(Default)]
pub struct NotebookContext {
    // Database connection, variables, etc.
    variables: HashMap<String, String>,
    data_sources: Vec<String>,
}

impl NotebookContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            data_sources: Vec::new(),
        }
    }
    
    pub fn execute_sql(&self, sql: &str) -> Result<Vec<Vec<String>>> {
        // Placeholder for SQL execution
        Ok(vec![
            vec!["Column1".to_string(), "Column2".to_string()],
            vec!["Value1".to_string(), "Value2".to_string()],
        ])
    }
    
    pub fn generate_plot(&self, spec: &str) -> Result<Vec<u8>> {
        // Placeholder for plot generation
        Ok(vec![])
    }
    
    pub fn load_data(&self, source: &str) -> Result<DataInfo> {
        // Placeholder for data loading
        Ok(DataInfo {
            name: "sample_data".to_string(),
            row_count: 100,
            column_count: 5,
            preview: vec![
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
            ],
        })
    }
    
    pub fn analyze_data(&self, spec: &str) -> Result<AnalysisResult> {
        // Placeholder for data analysis
        let mut statistics = HashMap::new();
        statistics.insert("mean".to_string(), 42.0);
        statistics.insert("std".to_string(), 10.5);
        
        Ok(AnalysisResult {
            statistics,
            insights: vec![
                "Data shows normal distribution".to_string(),
                "No significant outliers detected".to_string(),
            ],
            recommendations: vec![
                "Consider log transformation".to_string(),
                "Check for seasonality".to_string(),
            ],
        })
    }
}

/// Data information structure
pub struct DataInfo {
    pub name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub preview: Vec<Vec<String>>,
}

/// Analysis result structure
pub struct AnalysisResult {
    pub statistics: HashMap<String, f64>,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Main notebook interface
#[derive(Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    pub cells: Vec<NotebookCell>,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    
    #[serde(skip)]
    context: NotebookContext,
    
    #[serde(skip)]
    selected_cell: Option<usize>,
    
    #[serde(skip)]
    editing_cell: Option<usize>,
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new("Untitled Notebook".to_string())
    }
}

impl Notebook {
    pub fn new(title: String) -> Self {
        let mut notebook = Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            cells: Vec::new(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            metadata: HashMap::new(),
            context: NotebookContext::new(),
            selected_cell: None,
            editing_cell: None,
        };
        
        // Add initial cells
        notebook.add_cell(CellType::Markdown, "# Welcome to Pika-Plot Notebook\n\nThis is a data analysis and visualization notebook.".to_string());
        notebook.add_cell(CellType::Code, "-- SQL query example\nSELECT * FROM your_data LIMIT 10;".to_string());
        
        notebook
    }
    
    pub fn add_cell(&mut self, cell_type: CellType, content: String) {
        let cell = NotebookCell::new(cell_type).with_content(content);
        self.cells.push(cell);
        self.last_modified = Utc::now();
    }
    
    pub fn insert_cell(&mut self, index: usize, cell_type: CellType) {
        let cell = NotebookCell::new(cell_type);
        self.cells.insert(index, cell);
        self.last_modified = Utc::now();
    }
    
    pub fn delete_cell(&mut self, index: usize) {
        if index < self.cells.len() {
            self.cells.remove(index);
            self.last_modified = Utc::now();
            
            // Adjust selected cell
            if let Some(selected) = self.selected_cell {
                if selected >= index && selected > 0 {
                    self.selected_cell = Some(selected - 1);
                } else if selected == index {
                    self.selected_cell = None;
                }
            }
        }
    }
    
    pub fn execute_cell(&mut self, index: usize) -> Result<()> {
        if let Some(cell) = self.cells.get_mut(index) {
            cell.execute(&self.context)?;
            self.last_modified = Utc::now();
        }
        Ok(())
    }
    
    pub fn execute_all(&mut self) -> Result<()> {
        for i in 0..self.cells.len() {
            self.execute_cell(i)?;
        }
        Ok(())
    }
    
    pub fn show(&mut self, ui: &mut Ui) {
        // Notebook header
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📁 Save").clicked() {
                    // TODO: Save notebook
                }
                if ui.button("▶ Run All").clicked() {
                    let _ = self.execute_all();
                }
                if ui.button("➕ Add Cell").clicked() {
                    self.add_cell(CellType::Code, String::new());
                }
            });
        });
        
        ui.separator();
        
        // Notebook cells
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut cells_to_delete = Vec::new();
                
                for (i, cell) in self.cells.iter_mut().enumerate() {
                    let cell_response = self.show_cell(ui, i, cell);
                    
                    if cell_response.delete_requested {
                        cells_to_delete.push(i);
                    }
                }
                
                // Delete cells (in reverse order to maintain indices)
                for &index in cells_to_delete.iter().rev() {
                    self.delete_cell(index);
                }
            });
    }
    
    fn show_cell(&mut self, ui: &mut Ui, index: usize, cell: &mut NotebookCell) -> CellResponse {
        let mut response = CellResponse::default();
        let is_selected = self.selected_cell == Some(index);
        let is_editing = self.editing_cell == Some(index);
        
        // Cell frame
        let frame_color = if is_selected {
            Color32::BLUE
        } else {
            Color32::GRAY
        };
        
        egui::Frame::none()
            .stroke(egui::Stroke::new(if is_selected { 2.0 } else { 1.0 }, frame_color))
            .inner_margin(8.0)
            .show(ui, |ui| {
                // Cell header
                ui.horizontal(|ui| {
                    // Cell type selector
                    let mut current_type = cell.cell_type.clone();
                    ComboBox::from_id_source(format!("cell_type_{}", index))
                        .selected_text(format!("{}", current_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut current_type, CellType::Code, "Code");
                            ui.selectable_value(&mut current_type, CellType::Markdown, "Markdown");
                            ui.selectable_value(&mut current_type, CellType::Plot, "Plot");
                            ui.selectable_value(&mut current_type, CellType::Data, "Data");
                            ui.selectable_value(&mut current_type, CellType::Analysis, "Analysis");
                        });
                    
                    if current_type != cell.cell_type {
                        cell.cell_type = current_type;
                        cell.output = None; // Clear output when type changes
                    }
                    
                    // Execution count
                    if cell.execution_count > 0 {
                        ui.label(format!("[{}]", cell.execution_count));
                    } else {
                        ui.label("[ ]");
                    }
                    
                    // Status indicator
                    let status_text = match &cell.status {
                        CellStatus::NotExecuted => "⏸",
                        CellStatus::Running => "⏳",
                        CellStatus::Success => "✅",
                        CellStatus::Error(_) => "❌",
                    };
                    ui.label(status_text);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Cell actions
                        if ui.small_button("🗑").clicked() {
                            response.delete_requested = true;
                        }
                        if ui.small_button("⬇").clicked() {
                            self.insert_cell(index + 1, CellType::Code);
                        }
                        if ui.small_button("⬆").clicked() {
                            self.insert_cell(index, CellType::Code);
                        }
                        if ui.small_button("▶").clicked() {
                            let _ = self.execute_cell(index);
                        }
                    });
                });
                
                ui.separator();
                
                // Cell content
                let content_response = ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 100.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        if is_editing {
                            // Edit mode
                            let text_edit = TextEdit::multiline(&mut cell.content)
                                .desired_width(ui.available_width())
                                .desired_rows(5);
                            
                            let edit_response = ui.add(text_edit);
                            
                            if edit_response.lost_focus() {
                                self.editing_cell = None;
                            }
                            
                            // Keyboard shortcuts
                            if ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.shift) {
                                let _ = self.execute_cell(index);
                                self.editing_cell = None;
                            }
                        } else {
                            // Display mode
                            match cell.cell_type {
                                CellType::Markdown => {
                                    if let Some(CellOutput::Html(html)) = &cell.output {
                                        // Render markdown (simplified)
                                        ui.label(RichText::new(&cell.content).weak());
                                    } else {
                                        ui.label(RichText::new(&cell.content).weak());
                                    }
                                }
                                _ => {
                                    ui.label(RichText::new(&cell.content).code());
                                }
                            }
                            
                            // Click to edit
                            if ui.response().clicked() {
                                self.editing_cell = Some(index);
                                self.selected_cell = Some(index);
                            }
                        }
                    },
                );
                
                // Cell output
                if let Some(output) = &cell.output {
                    ui.separator();
                    self.show_cell_output(ui, output);
                }
                
                // Error display
                if let CellStatus::Error(error) = &cell.status {
                    ui.separator();
                    ui.colored_label(Color32::RED, format!("Error: {}", error));
                }
            });
        
        ui.add_space(10.0);
        response
    }
    
    fn show_cell_output(&self, ui: &mut Ui, output: &CellOutput) {
        match output {
            CellOutput::Text(text) => {
                ui.label(text);
            }
            CellOutput::Html(html) => {
                // Simplified HTML rendering
                ui.label(RichText::new(html).weak());
            }
            CellOutput::Plot { plot_id, interactive, .. } => {
                ui.label(format!("Plot: {} (Interactive: {})", plot_id, interactive));
                // TODO: Embed actual plot
            }
            CellOutput::Data { table_name, row_count, column_count, preview } => {
                ui.label(format!("Table: {} ({} rows, {} columns)", table_name, row_count, column_count));
                
                // Show data preview
                egui::Grid::new("data_preview")
                    .striped(true)
                    .show(ui, |ui| {
                        for row in preview.iter().take(5) {
                            for cell in row {
                                ui.label(cell);
                            }
                            ui.end_row();
                        }
                    });
            }
            CellOutput::Analysis { statistics, insights, recommendations } => {
                ui.label("📊 Analysis Results");
                
                ui.collapsing("Statistics", |ui| {
                    for (key, value) in statistics {
                        ui.label(format!("{}: {:.3}", key, value));
                    }
                });
                
                ui.collapsing("Insights", |ui| {
                    for insight in insights {
                        ui.label(format!("• {}", insight));
                    }
                });
                
                ui.collapsing("Recommendations", |ui| {
                    for recommendation in recommendations {
                        ui.label(format!("💡 {}", recommendation));
                    }
                });
            }
            CellOutput::Error(error) => {
                ui.colored_label(Color32::RED, error);
            }
        }
    }
}

#[derive(Default)]
struct CellResponse {
    delete_requested: bool,
}

/// Simple markdown to HTML converter (placeholder)
fn markdown_to_html(markdown: &str) -> String {
    // Very basic markdown parsing
    let mut html = markdown.to_string();
    
    // Headers
    html = html.replace("# ", "<h1>").replace("\n", "</h1>\n");
    html = html.replace("## ", "<h2>").replace("\n", "</h2>\n");
    html = html.replace("### ", "<h3>").replace("\n", "</h3>\n");
    
    // Bold and italic
    html = html.replace("**", "<strong>").replace("**", "</strong>");
    html = html.replace("*", "<em>").replace("*", "</em>");
    
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_notebook_creation() {
        let notebook = Notebook::new("Test Notebook".to_string());
        assert_eq!(notebook.title, "Test Notebook");
        assert!(!notebook.cells.is_empty());
    }
    
    #[test]
    fn test_cell_creation() {
        let cell = NotebookCell::new(CellType::Code);
        assert_eq!(cell.cell_type, CellType::Code);
        assert_eq!(cell.status, CellStatus::NotExecuted);
        assert_eq!(cell.execution_count, 0);
    }
    
    #[test]
    fn test_add_cell() {
        let mut notebook = Notebook::new("Test".to_string());
        let initial_count = notebook.cells.len();
        
        notebook.add_cell(CellType::Markdown, "# Test".to_string());
        assert_eq!(notebook.cells.len(), initial_count + 1);
        
        let last_cell = notebook.cells.last().unwrap();
        assert_eq!(last_cell.cell_type, CellType::Markdown);
        assert_eq!(last_cell.content, "# Test");
    }
} 