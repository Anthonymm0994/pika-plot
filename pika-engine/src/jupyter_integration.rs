use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Jupyter protocol imports
use jupyter_protocol::{
    KernelInfo, KernelInfoReply, ExecuteRequest, ExecuteReply, ExecuteResult,
    DisplayData, StreamContent, ErrorContent, CompleteRequest, CompleteReply,
    InspectRequest, InspectReply, HistoryRequest, HistoryReply,
    IsCompleteRequest, IsCompleteReply, CommInfoRequest, CommInfoReply,
    ShutdownRequest, ShutdownReply, InterruptRequest, InterruptReply,
    Status, ExecutionState, Header, Message, MessageType,
};

/// Jupyter notebook integration engine
pub struct JupyterIntegrationEngine {
    kernel_info: KernelInfo,
    execution_engine: ExecutionEngine,
    notebook_manager: NotebookManager,
    output_manager: OutputManager,
    completion_engine: CompletionEngine,
    magic_commands: MagicCommandRegistry,
    widget_manager: WidgetManager,
    plotting_backend: PlottingBackend,
    session_manager: SessionManager,
}

/// Rust code execution engine using evcxr
pub struct ExecutionEngine {
    context: evcxr::EvalContext,
    execution_count: u64,
    variables: HashMap<String, VariableInfo>,
    imports: Vec<String>,
    dependencies: Vec<String>,
}

/// Notebook management system
pub struct NotebookManager {
    notebooks: HashMap<String, Notebook>,
    active_notebook: Option<String>,
    autosave_enabled: bool,
    backup_manager: BackupManager,
}

/// Output management for rich display
pub struct OutputManager {
    outputs: HashMap<String, Vec<CellOutput>>,
    display_handlers: HashMap<String, DisplayHandler>,
    stream_buffers: HashMap<String, StreamBuffer>,
}

/// Code completion engine
pub struct CompletionEngine {
    completions: HashMap<String, Vec<Completion>>,
    documentation: HashMap<String, Documentation>,
    signature_help: HashMap<String, SignatureHelp>,
}

/// Magic command registry
pub struct MagicCommandRegistry {
    line_magics: HashMap<String, LineMagic>,
    cell_magics: HashMap<String, CellMagic>,
}

/// Widget management system
pub struct WidgetManager {
    widgets: HashMap<String, Widget>,
    comm_manager: CommManager,
}

/// Plotting backend for visualization
pub struct PlottingBackend {
    backend_type: PlottingBackendType,
    figure_manager: FigureManager,
    interactive_plots: HashMap<String, InteractivePlot>,
}

/// Session management
pub struct SessionManager {
    sessions: HashMap<String, JupyterSession>,
    active_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub cells: Vec<Cell>,
    pub metadata: NotebookMetadata,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub kernel_spec: KernelSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    pub cell_type: CellType,
    pub source: String,
    pub outputs: Vec<CellOutput>,
    pub metadata: CellMetadata,
    pub execution_count: Option<u64>,
    pub execution_state: CellExecutionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    Code,
    Markdown,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellExecutionState {
    Idle,
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub output_type: OutputType,
    pub data: OutputData,
    pub metadata: HashMap<String, Value>,
    pub execution_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    ExecuteResult,
    DisplayData,
    Stream,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputData {
    Text(String),
    Html(String),
    Markdown(String),
    Latex(String),
    Json(Value),
    Image(ImageData),
    Plot(PlotData),
    Table(TableData),
    Widget(WidgetData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Gif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotData {
    pub plot_type: String,
    pub data: Value,
    pub config: Value,
    pub interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub index: Option<Vec<String>>,
    pub styling: Option<TableStyling>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStyling {
    pub striped: bool,
    pub bordered: bool,
    pub hover: bool,
    pub condensed: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub widget_type: String,
    pub state: Value,
    pub layout: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub kernelspec: KernelSpec,
    pub language_info: LanguageInfo,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub custom: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub name: String,
    pub display_name: String,
    pub language: String,
    pub argv: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub version: String,
    pub mimetype: String,
    pub file_extension: String,
    pub pygments_lexer: String,
    pub codemirror_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub tags: Vec<String>,
    pub collapsed: bool,
    pub scrolled: bool,
    pub trusted: bool,
    pub custom: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub type_name: String,
    pub value: String,
    pub size: Option<usize>,
    pub shape: Option<Vec<usize>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub name: String,
    pub signature: String,
    pub docstring: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelp {
    pub signatures: Vec<Signature>,
    pub active_signature: usize,
    pub active_parameter: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub label: String,
    pub documentation: Option<String>,
}

pub type LineMagic = Box<dyn Fn(&str, &ExecutionEngine) -> Result<CellOutput> + Send + Sync>;
pub type CellMagic = Box<dyn Fn(&str, &str, &ExecutionEngine) -> Result<CellOutput> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub widget_type: String,
    pub state: Value,
    pub layout: Value,
    pub comm_id: String,
}

pub struct CommManager {
    comms: HashMap<String, Comm>,
}

#[derive(Debug, Clone)]
pub struct Comm {
    pub id: String,
    pub target_name: String,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub enum PlottingBackendType {
    Plotters,
    Charming,
    Plotlars,
    Custom(String),
}

pub struct FigureManager {
    figures: HashMap<String, Figure>,
    active_figure: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Figure {
    pub id: String,
    pub title: String,
    pub size: (u32, u32),
    pub dpi: f64,
    pub format: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct InteractivePlot {
    pub id: String,
    pub plot_type: String,
    pub data: Value,
    pub widgets: Vec<String>,
}

pub type DisplayHandler = Box<dyn Fn(&Value) -> Result<OutputData> + Send + Sync>;

pub struct StreamBuffer {
    pub content: String,
    pub stream_type: StreamType,
}

#[derive(Debug, Clone)]
pub enum StreamType {
    Stdout,
    Stderr,
}

pub struct BackupManager {
    backup_dir: PathBuf,
    backup_interval: std::time::Duration,
    max_backups: usize,
}

#[derive(Debug, Clone)]
pub struct JupyterSession {
    pub id: String,
    pub kernel_id: String,
    pub notebook_path: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

impl JupyterIntegrationEngine {
    pub fn new() -> Result<Self> {
        let kernel_info = KernelInfo {
            protocol_version: "5.3".to_string(),
            implementation: "pika-plot".to_string(),
            implementation_version: "0.1.0".to_string(),
            language_info: jupyter_protocol::LanguageInfo {
                name: "rust".to_string(),
                version: "1.75.0".to_string(),
                mimetype: "text/x-rust".to_string(),
                file_extension: ".rs".to_string(),
                pygments_lexer: "rust".to_string(),
                codemirror_mode: "rust".to_string(),
                nbconvert_exporter: "rust".to_string(),
            },
            banner: "Pika-Plot Rust Kernel for Data Analysis and Visualization".to_string(),
            help_links: vec![],
            status: Status::Ok,
        };

        let execution_engine = ExecutionEngine {
            context: evcxr::EvalContext::new()?,
            execution_count: 0,
            variables: HashMap::new(),
            imports: vec![
                "use polars::prelude::*;".to_string(),
                "use plotters::prelude::*;".to_string(),
                "use serde::{Serialize, Deserialize};".to_string(),
                "use std::collections::HashMap;".to_string(),
            ],
            dependencies: vec![
                "polars".to_string(),
                "plotters".to_string(),
                "serde".to_string(),
                "serde_json".to_string(),
                "chrono".to_string(),
                "uuid".to_string(),
            ],
        };

        let notebook_manager = NotebookManager {
            notebooks: HashMap::new(),
            active_notebook: None,
            autosave_enabled: true,
            backup_manager: BackupManager {
                backup_dir: PathBuf::from("./backups"),
                backup_interval: std::time::Duration::from_secs(300), // 5 minutes
                max_backups: 10,
            },
        };

        let mut output_manager = OutputManager {
            outputs: HashMap::new(),
            display_handlers: HashMap::new(),
            stream_buffers: HashMap::new(),
        };

        // Register display handlers
        output_manager.register_display_handlers();

        let completion_engine = CompletionEngine {
            completions: HashMap::new(),
            documentation: HashMap::new(),
            signature_help: HashMap::new(),
        };

        let mut magic_commands = MagicCommandRegistry {
            line_magics: HashMap::new(),
            cell_magics: HashMap::new(),
        };

        // Register magic commands
        magic_commands.register_standard_magics();

        let widget_manager = WidgetManager {
            widgets: HashMap::new(),
            comm_manager: CommManager {
                comms: HashMap::new(),
            },
        };

        let plotting_backend = PlottingBackend {
            backend_type: PlottingBackendType::Plotters,
            figure_manager: FigureManager {
                figures: HashMap::new(),
                active_figure: None,
            },
            interactive_plots: HashMap::new(),
        };

        let session_manager = SessionManager {
            sessions: HashMap::new(),
            active_session: None,
        };

        Ok(Self {
            kernel_info,
            execution_engine,
            notebook_manager,
            output_manager,
            completion_engine,
            magic_commands,
            widget_manager,
            plotting_backend,
            session_manager,
        })
    }

    /// Execute code in a cell
    pub async fn execute_cell(&mut self, cell_id: &str, code: &str) -> Result<ExecuteReply> {
        self.execution_engine.execution_count += 1;
        let execution_count = self.execution_engine.execution_count;

        // Check for magic commands
        if let Some(magic_result) = self.handle_magic_commands(code)? {
            return Ok(ExecuteReply {
                status: Status::Ok,
                execution_count,
                payload: vec![],
                user_expressions: HashMap::new(),
            });
        }

        // Execute the code
        let result = self.execution_engine.execute(code).await?;

        // Store outputs
        self.output_manager.outputs.insert(cell_id.to_string(), result.outputs.clone());

        // Update cell in notebook
        if let Some(notebook_id) = &self.notebook_manager.active_notebook {
            if let Some(notebook) = self.notebook_manager.notebooks.get_mut(notebook_id) {
                if let Some(cell) = notebook.cells.iter_mut().find(|c| c.id == cell_id) {
                    cell.outputs = result.outputs;
                    cell.execution_count = Some(execution_count);
                    cell.execution_state = if result.success {
                        CellExecutionState::Completed
                    } else {
                        CellExecutionState::Error
                    };
                }
            }
        }

        Ok(ExecuteReply {
            status: if result.success { Status::Ok } else { Status::Error },
            execution_count,
            payload: vec![],
            user_expressions: HashMap::new(),
        })
    }

    /// Handle magic commands
    fn handle_magic_commands(&mut self, code: &str) -> Result<Option<CellOutput>> {
        let lines: Vec<&str> = code.lines().collect();
        
        if lines.is_empty() {
            return Ok(None);
        }

        let first_line = lines[0].trim();

        // Line magic
        if first_line.starts_with('%') && !first_line.starts_with("%%") {
            let parts: Vec<&str> = first_line[1..].splitn(2, ' ').collect();
            let magic_name = parts[0];
            let args = parts.get(1).unwrap_or(&"");

            if let Some(magic) = self.magic_commands.line_magics.get(magic_name) {
                return Ok(Some(magic(args, &self.execution_engine)?));
            }
        }

        // Cell magic
        if first_line.starts_with("%%") {
            let parts: Vec<&str> = first_line[2..].splitn(2, ' ').collect();
            let magic_name = parts[0];
            let args = parts.get(1).unwrap_or(&"");
            let cell_body = lines[1..].join("\n");

            if let Some(magic) = self.magic_commands.cell_magics.get(magic_name) {
                return Ok(Some(magic(args, &cell_body, &self.execution_engine)?));
            }
        }

        Ok(None)
    }

    /// Create a new notebook
    pub fn create_notebook(&mut self, name: String, path: PathBuf) -> Result<String> {
        let notebook_id = Uuid::new_v4().to_string();
        
        let notebook = Notebook {
            id: notebook_id.clone(),
            name,
            path,
            cells: vec![],
            metadata: NotebookMetadata {
                kernelspec: KernelSpec {
                    name: "rust".to_string(),
                    display_name: "Rust".to_string(),
                    language: "rust".to_string(),
                    argv: vec!["pika-plot".to_string(), "kernel".to_string()],
                    env: HashMap::new(),
                },
                language_info: LanguageInfo {
                    name: "rust".to_string(),
                    version: "1.75.0".to_string(),
                    mimetype: "text/x-rust".to_string(),
                    file_extension: ".rs".to_string(),
                    pygments_lexer: "rust".to_string(),
                    codemirror_mode: "rust".to_string(),
                },
                title: None,
                authors: vec![],
                tags: vec![],
                custom: HashMap::new(),
            },
            created_at: Utc::now(),
            modified_at: Utc::now(),
            kernel_spec: KernelSpec {
                name: "rust".to_string(),
                display_name: "Rust".to_string(),
                language: "rust".to_string(),
                argv: vec!["pika-plot".to_string(), "kernel".to_string()],
                env: HashMap::new(),
            },
        };

        self.notebook_manager.notebooks.insert(notebook_id.clone(), notebook);
        self.notebook_manager.active_notebook = Some(notebook_id.clone());

        Ok(notebook_id)
    }

    /// Add a cell to the notebook
    pub fn add_cell(&mut self, notebook_id: &str, cell_type: CellType, source: String) -> Result<String> {
        let cell_id = Uuid::new_v4().to_string();
        
        let cell = Cell {
            id: cell_id.clone(),
            cell_type,
            source,
            outputs: vec![],
            metadata: CellMetadata {
                tags: vec![],
                collapsed: false,
                scrolled: false,
                trusted: true,
                custom: HashMap::new(),
            },
            execution_count: None,
            execution_state: CellExecutionState::Idle,
        };

        if let Some(notebook) = self.notebook_manager.notebooks.get_mut(notebook_id) {
            notebook.cells.push(cell);
            notebook.modified_at = Utc::now();
        }

        Ok(cell_id)
    }

    /// Get code completions
    pub fn get_completions(&self, code: &str, cursor_pos: usize) -> Result<Vec<Completion>> {
        // Extract the current word being typed
        let before_cursor = &code[..cursor_pos];
        let words: Vec<&str> = before_cursor.split_whitespace().collect();
        let current_word = words.last().unwrap_or(&"");

        let mut completions = Vec::new();

        // Rust keywords
        let keywords = vec![
            "let", "mut", "fn", "struct", "enum", "impl", "trait", "use", "pub", "mod",
            "if", "else", "match", "for", "while", "loop", "break", "continue", "return",
            "true", "false", "Some", "None", "Ok", "Err", "Vec", "HashMap", "String",
        ];

        for keyword in keywords {
            if keyword.starts_with(current_word) {
                completions.push(Completion {
                    text: keyword.to_string(),
                    start: cursor_pos - current_word.len(),
                    end: cursor_pos,
                    type_info: Some("keyword".to_string()),
                    documentation: None,
                });
            }
        }

        // Variable completions
        for (var_name, var_info) in &self.execution_engine.variables {
            if var_name.starts_with(current_word) {
                completions.push(Completion {
                    text: var_name.clone(),
                    start: cursor_pos - current_word.len(),
                    end: cursor_pos,
                    type_info: Some(var_info.type_name.clone()),
                    documentation: var_info.description.clone(),
                });
            }
        }

        // Function completions (simplified)
        let functions = vec![
            ("println!", "macro"),
            ("format!", "macro"),
            ("vec!", "macro"),
            ("DataFrame::new", "function"),
            ("LazyFrame::new", "function"),
            ("col", "function"),
            ("lit", "function"),
        ];

        for (func_name, func_type) in functions {
            if func_name.starts_with(current_word) {
                completions.push(Completion {
                    text: func_name.to_string(),
                    start: cursor_pos - current_word.len(),
                    end: cursor_pos,
                    type_info: Some(func_type.to_string()),
                    documentation: None,
                });
            }
        }

        Ok(completions)
    }

    /// Save notebook
    pub fn save_notebook(&self, notebook_id: &str) -> Result<()> {
        if let Some(notebook) = self.notebook_manager.notebooks.get(notebook_id) {
            let json = serde_json::to_string_pretty(notebook)?;
            std::fs::write(&notebook.path, json)?;
        }
        Ok(())
    }

    /// Load notebook
    pub fn load_notebook(&mut self, path: PathBuf) -> Result<String> {
        let content = std::fs::read_to_string(&path)?;
        let notebook: Notebook = serde_json::from_str(&content)?;
        let notebook_id = notebook.id.clone();
        
        self.notebook_manager.notebooks.insert(notebook_id.clone(), notebook);
        self.notebook_manager.active_notebook = Some(notebook_id.clone());
        
        Ok(notebook_id)
    }

    /// Export notebook to HTML
    pub fn export_to_html(&self, notebook_id: &str) -> Result<String> {
        let notebook = self.notebook_manager.notebooks.get(notebook_id)
            .ok_or_else(|| anyhow::anyhow!("Notebook not found"))?;

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", notebook.name));
        html.push_str("<style>\n");
        // html.push_str(include_str!("../assets/notebook.css"));
        html.push_str(r#"
                .notebook-container { font-family: Arial, sans-serif; }
                .cell { margin: 10px 0; padding: 10px; border: 1px solid #ddd; }
                .output { background: #f5f5f5; padding: 10px; }
        "#);
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str(&format!("<h1>{}</h1>\n", notebook.name));

        for cell in &notebook.cells {
            match cell.cell_type {
                CellType::Code => {
                    html.push_str("<div class=\"code-cell\">\n");
                    html.push_str("<div class=\"input\">\n");
                    html.push_str(&format!("<pre><code>{}</code></pre>\n", cell.source));
                    html.push_str("</div>\n");
                    
                    if !cell.outputs.is_empty() {
                        html.push_str("<div class=\"output\">\n");
                        for output in &cell.outputs {
                            html.push_str(&self.output_to_html(output)?);
                        }
                        html.push_str("</div>\n");
                    }
                    html.push_str("</div>\n");
                },
                CellType::Markdown => {
                    html.push_str("<div class=\"markdown-cell\">\n");
                    // Convert markdown to HTML (simplified)
                    html.push_str(&format!("<div class=\"markdown\">{}</div>\n", cell.source));
                    html.push_str("</div>\n");
                },
                CellType::Raw => {
                    html.push_str("<div class=\"raw-cell\">\n");
                    html.push_str(&format!("<pre>{}</pre>\n", cell.source));
                    html.push_str("</div>\n");
                },
            }
        }

        html.push_str("</body>\n</html>");
        Ok(html)
    }

    /// Convert output to HTML
    fn output_to_html(&self, output: &CellOutput) -> Result<String> {
        match &output.data {
            OutputData::Text(text) => Ok(format!("<pre>{}</pre>", text)),
            OutputData::Html(html) => Ok(html.clone()),
            OutputData::Markdown(md) => Ok(format!("<div class=\"markdown\">{}</div>", md)),
            OutputData::Image(img) => {
                let base64 = base64::encode(&img.data);
                let mime_type = match img.format {
                    ImageFormat::Png => "image/png",
                    ImageFormat::Jpeg => "image/jpeg",
                    ImageFormat::Svg => "image/svg+xml",
                    ImageFormat::Gif => "image/gif",
                };
                Ok(format!("<img src=\"data:{};base64,{}\" />", mime_type, base64))
            },
            OutputData::Table(table) => {
                let mut html = String::new();
                html.push_str("<table class=\"data-table\">\n");
                
                // Headers
                html.push_str("<thead><tr>\n");
                for header in &table.headers {
                    html.push_str(&format!("<th>{}</th>\n", header));
                }
                html.push_str("</tr></thead>\n");
                
                // Rows
                html.push_str("<tbody>\n");
                for row in &table.rows {
                    html.push_str("<tr>\n");
                    for cell in row {
                        html.push_str(&format!("<td>{}</td>\n", cell));
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody>\n</table>\n");
                
                Ok(html)
            },
            _ => Ok(format!("<pre>{:?}</pre>", output.data)),
        }
    }

    /// Get kernel info
    pub fn get_kernel_info(&self) -> &KernelInfo {
        &self.kernel_info
    }

    /// Get active notebook
    pub fn get_active_notebook(&self) -> Option<&Notebook> {
        self.notebook_manager.active_notebook.as_ref()
            .and_then(|id| self.notebook_manager.notebooks.get(id))
    }

    /// List all notebooks
    pub fn list_notebooks(&self) -> Vec<&Notebook> {
        self.notebook_manager.notebooks.values().collect()
    }
}

impl ExecutionEngine {
    /// Execute Rust code
    pub async fn execute(&mut self, code: &str) -> Result<ExecutionResult> {
        // Add imports if not already present
        for import in &self.imports {
            if !self.context.defined_names().contains(&import.clone()) {
                self.context.eval(import)?;
            }
        }

        // Execute the code
        let result = self.context.eval(code);

        let mut outputs = Vec::new();
        let success = match result {
            Ok(output) => {
                if !output.is_empty() {
                    outputs.push(CellOutput {
                        output_type: OutputType::ExecuteResult,
                        data: OutputData::Text(output),
                        metadata: HashMap::new(),
                        execution_count: Some(self.execution_count),
                    });
                }
                true
            },
            Err(error) => {
                outputs.push(CellOutput {
                    output_type: OutputType::Error,
                    data: OutputData::Text(format!("Error: {}", error)),
                    metadata: HashMap::new(),
                    execution_count: Some(self.execution_count),
                });
                false
            },
        };

        // Update variables
        self.update_variables();

        Ok(ExecutionResult {
            success,
            outputs,
            execution_count: self.execution_count,
        })
    }

    /// Update variable information
    fn update_variables(&mut self) {
        // This is a simplified version - in practice, you'd need to
        // inspect the evaluation context for variables
        let defined_names = self.context.defined_names();
        
        for name in defined_names {
            if !self.variables.contains_key(&name) {
                self.variables.insert(name.clone(), VariableInfo {
                    name: name.clone(),
                    type_name: "unknown".to_string(),
                    value: "...".to_string(),
                    size: None,
                    shape: None,
                    description: None,
                });
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub outputs: Vec<CellOutput>,
    pub execution_count: u64,
}

impl OutputManager {
    /// Register display handlers
    fn register_display_handlers(&mut self) {
        // Register handler for DataFrames
        self.display_handlers.insert("DataFrame".to_string(), Box::new(|value| {
            // Convert DataFrame to table format
            Ok(OutputData::Table(TableData {
                headers: vec!["Column".to_string(), "Type".to_string()],
                rows: vec![],
                index: None,
                styling: Some(TableStyling {
                    striped: true,
                    bordered: true,
                    hover: true,
                    condensed: false,
                    theme: "default".to_string(),
                }),
            }))
        }));

        // Register handler for plots
        self.display_handlers.insert("Plot".to_string(), Box::new(|value| {
            Ok(OutputData::Plot(PlotData {
                plot_type: "line".to_string(),
                data: value.clone(),
                config: serde_json::json!({}),
                interactive: true,
            }))
        }));
    }
}

impl MagicCommandRegistry {
    /// Register standard magic commands
    fn register_standard_magics(&mut self) {
        // %time magic
        self.line_magics.insert("time".to_string(), Box::new(|args, engine| {
            let start = std::time::Instant::now();
            // Execute the code (simplified)
            let duration = start.elapsed();
            
            Ok(CellOutput {
                output_type: OutputType::Stream,
                data: OutputData::Text(format!("Execution time: {:?}", duration)),
                metadata: HashMap::new(),
                execution_count: Some(engine.execution_count),
            })
        }));

        // %load magic
        self.line_magics.insert("load".to_string(), Box::new(|args, _engine| {
            let content = std::fs::read_to_string(args.trim())?;
            Ok(CellOutput {
                output_type: OutputType::ExecuteResult,
                data: OutputData::Text(content),
                metadata: HashMap::new(),
                execution_count: None,
            })
        }));

        // %%html magic
        self.cell_magics.insert("html".to_string(), Box::new(|_args, body, _engine| {
            Ok(CellOutput {
                output_type: OutputType::DisplayData,
                data: OutputData::Html(body.to_string()),
                metadata: HashMap::new(),
                execution_count: None,
            })
        }));

        // %%markdown magic
        self.cell_magics.insert("markdown".to_string(), Box::new(|_args, body, _engine| {
            Ok(CellOutput {
                output_type: OutputType::DisplayData,
                data: OutputData::Markdown(body.to_string()),
                metadata: HashMap::new(),
                execution_count: None,
            })
        }));
    }
}

impl Default for JupyterIntegrationEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jupyter_engine_creation() {
        let engine = JupyterIntegrationEngine::new();
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_notebook_creation() {
        let mut engine = JupyterIntegrationEngine::new().unwrap();
        let notebook_id = engine.create_notebook(
            "Test Notebook".to_string(),
            PathBuf::from("test.ipynb")
        ).unwrap();
        
        assert!(!notebook_id.is_empty());
        assert!(engine.notebook_manager.notebooks.contains_key(&notebook_id));
    }

    #[tokio::test]
    async fn test_cell_execution() {
        let mut engine = JupyterIntegrationEngine::new().unwrap();
        let notebook_id = engine.create_notebook(
            "Test Notebook".to_string(),
            PathBuf::from("test.ipynb")
        ).unwrap();
        
        let cell_id = engine.add_cell(&notebook_id, CellType::Code, "let x = 42;".to_string()).unwrap();
        let result = engine.execute_cell(&cell_id, "let x = 42;").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_code_completion() {
        let engine = JupyterIntegrationEngine::new().unwrap();
        let completions = engine.get_completions("let x = Vec", 13).unwrap();
        
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.text == "Vec"));
    }

    #[test]
    fn test_html_export() {
        let mut engine = JupyterIntegrationEngine::new().unwrap();
        let notebook_id = engine.create_notebook(
            "Test Notebook".to_string(),
            PathBuf::from("test.ipynb")
        ).unwrap();
        
        engine.add_cell(&notebook_id, CellType::Code, "let x = 42;".to_string()).unwrap();
        
        let html = engine.export_to_html(&notebook_id);
        assert!(html.is_ok());
        
        let html_content = html.unwrap();
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("Test Notebook"));
    }
} 