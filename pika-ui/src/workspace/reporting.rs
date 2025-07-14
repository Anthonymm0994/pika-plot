use egui::{Ui, Color32, ScrollArea, RichText, TextEdit, Button, ComboBox, Stroke};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use pika_core::error::Result;

/// Report types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportType {
    DataAnalysis,
    Visualization,
    Statistical,
    Executive,
    Technical,
    Custom,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportType::DataAnalysis => write!(f, "Data Analysis"),
            ReportType::Visualization => write!(f, "Visualization"),
            ReportType::Statistical => write!(f, "Statistical"),
            ReportType::Executive => write!(f, "Executive Summary"),
            ReportType::Technical => write!(f, "Technical"),
            ReportType::Custom => write!(f, "Custom"),
        }
    }
}

/// Report section types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionType {
    Title,
    Summary,
    KeyFindings,
    Methodology,
    Results,
    Visualization,
    Statistics,
    Recommendations,
    Conclusions,
    Appendix,
    Custom(String),
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionType::Title => write!(f, "Title"),
            SectionType::Summary => write!(f, "Executive Summary"),
            SectionType::KeyFindings => write!(f, "Key Findings"),
            SectionType::Methodology => write!(f, "Methodology"),
            SectionType::Results => write!(f, "Results"),
            SectionType::Visualization => write!(f, "Visualizations"),
            SectionType::Statistics => write!(f, "Statistical Analysis"),
            SectionType::Recommendations => write!(f, "Recommendations"),
            SectionType::Conclusions => write!(f, "Conclusions"),
            SectionType::Appendix => write!(f, "Appendix"),
            SectionType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Report content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text(String),
    Markdown(String),
    Plot {
        plot_id: String,
        title: String,
        description: String,
        image_data: Vec<u8>,
    },
    Table {
        title: String,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Statistics {
        title: String,
        metrics: HashMap<String, f64>,
        insights: Vec<String>,
    },
    Chart {
        chart_type: ChartType,
        data: ChartData,
        title: String,
        description: String,
    },
    Image {
        title: String,
        description: String,
        image_data: Vec<u8>,
        format: ImageFormat,
    },
}

/// Chart types for embedded charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
    Histogram,
    Box,
    Heatmap,
}

/// Chart data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub series: Vec<DataSeries>,
}

/// Data series for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub color: Option<String>,
}

/// Image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub id: String,
    pub section_type: SectionType,
    pub title: String,
    pub content: Vec<ContentType>,
    pub order: usize,
    pub visible: bool,
}

impl ReportSection {
    pub fn new(section_type: SectionType, title: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            section_type,
            title,
            content: Vec::new(),
            order: 0,
            visible: true,
        }
    }
    
    pub fn add_content(&mut self, content: ContentType) {
        self.content.push(content);
    }
    
    pub fn add_text(&mut self, text: String) {
        self.add_content(ContentType::Text(text));
    }
    
    pub fn add_markdown(&mut self, markdown: String) {
        self.add_content(ContentType::Markdown(markdown));
    }
    
    pub fn add_table(&mut self, title: String, headers: Vec<String>, rows: Vec<Vec<String>>) {
        self.add_content(ContentType::Table { title, headers, rows });
    }
    
    pub fn add_statistics(&mut self, title: String, metrics: HashMap<String, f64>, insights: Vec<String>) {
        self.add_content(ContentType::Statistics { title, metrics, insights });
    }
}

/// Complete report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub report_type: ReportType,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub sections: Vec<ReportSection>,
    pub metadata: HashMap<String, String>,
    pub template_id: Option<String>,
}

impl Report {
    pub fn new(title: String, report_type: ReportType, author: String) -> Self {
        let mut report = Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            report_type: report_type.clone(),
            author,
            created_at: Utc::now(),
            last_modified: Utc::now(),
            sections: Vec::new(),
            metadata: HashMap::new(),
            template_id: None,
        };
        
        // Initialize with default sections based on report type
        report.initialize_default_sections();
        report
    }
    
    fn initialize_default_sections(&mut self) {
        match self.report_type {
            ReportType::DataAnalysis => {
                self.add_section(SectionType::Title, "Data Analysis Report".to_string());
                self.add_section(SectionType::Summary, "Executive Summary".to_string());
                self.add_section(SectionType::Methodology, "Data Sources and Methods".to_string());
                self.add_section(SectionType::Results, "Analysis Results".to_string());
                self.add_section(SectionType::Visualization, "Data Visualizations".to_string());
                self.add_section(SectionType::Recommendations, "Recommendations".to_string());
                self.add_section(SectionType::Conclusions, "Conclusions".to_string());
            }
            ReportType::Executive => {
                self.add_section(SectionType::Title, "Executive Summary".to_string());
                self.add_section(SectionType::KeyFindings, "Key Findings".to_string());
                self.add_section(SectionType::Recommendations, "Strategic Recommendations".to_string());
                self.add_section(SectionType::Conclusions, "Conclusions".to_string());
            }
            ReportType::Statistical => {
                self.add_section(SectionType::Title, "Statistical Analysis Report".to_string());
                self.add_section(SectionType::Methodology, "Statistical Methods".to_string());
                self.add_section(SectionType::Statistics, "Statistical Results".to_string());
                self.add_section(SectionType::Visualization, "Statistical Visualizations".to_string());
                self.add_section(SectionType::Conclusions, "Statistical Conclusions".to_string());
            }
            _ => {
                self.add_section(SectionType::Title, self.title.clone());
                self.add_section(SectionType::Summary, "Summary".to_string());
                self.add_section(SectionType::Results, "Results".to_string());
            }
        }
    }
    
    pub fn add_section(&mut self, section_type: SectionType, title: String) {
        let mut section = ReportSection::new(section_type, title);
        section.order = self.sections.len();
        self.sections.push(section);
        self.last_modified = Utc::now();
    }
    
    pub fn remove_section(&mut self, section_id: &str) {
        self.sections.retain(|s| s.id != section_id);
        self.reorder_sections();
        self.last_modified = Utc::now();
    }
    
    fn reorder_sections(&mut self) {
        for (i, section) in self.sections.iter_mut().enumerate() {
            section.order = i;
        }
    }
    
    pub fn get_section_mut(&mut self, section_id: &str) -> Option<&mut ReportSection> {
        self.sections.iter_mut().find(|s| s.id == section_id)
    }
    
    pub fn export_to_markdown(&self) -> String {
        let mut markdown = String::new();
        
        // Title
        markdown.push_str(&format!("# {}\n\n", self.title));
        
        // Metadata
        markdown.push_str(&format!("**Author:** {}\n", self.author));
        markdown.push_str(&format!("**Created:** {}\n", self.created_at.format("%Y-%m-%d %H:%M UTC")));
        markdown.push_str(&format!("**Type:** {}\n\n", self.report_type));
        
        markdown.push_str("---\n\n");
        
        // Sections
        for section in &self.sections {
            if !section.visible {
                continue;
            }
            
            markdown.push_str(&format!("## {}\n\n", section.title));
            
            for content in &section.content {
                match content {
                    ContentType::Text(text) => {
                        markdown.push_str(text);
                        markdown.push_str("\n\n");
                    }
                    ContentType::Markdown(md) => {
                        markdown.push_str(md);
                        markdown.push_str("\n\n");
                    }
                    ContentType::Table { title, headers, rows } => {
                        markdown.push_str(&format!("### {}\n\n", title));
                        
                        // Table headers
                        markdown.push('|');
                        for header in headers {
                            markdown.push_str(&format!(" {} |", header));
                        }
                        markdown.push('\n');
                        
                        // Table separator
                        markdown.push('|');
                        for _ in headers {
                            markdown.push_str(" --- |");
                        }
                        markdown.push('\n');
                        
                        // Table rows
                        for row in rows {
                            markdown.push('|');
                            for cell in row {
                                markdown.push_str(&format!(" {} |", cell));
                            }
                            markdown.push('\n');
                        }
                        markdown.push('\n');
                    }
                    ContentType::Statistics { title, metrics, insights } => {
                        markdown.push_str(&format!("### {}\n\n", title));
                        
                        markdown.push_str("**Key Metrics:**\n\n");
                        for (key, value) in metrics {
                            markdown.push_str(&format!("- {}: {:.3}\n", key, value));
                        }
                        markdown.push('\n');
                        
                        if !insights.is_empty() {
                            markdown.push_str("**Insights:**\n\n");
                            for insight in insights {
                                markdown.push_str(&format!("- {}\n", insight));
                            }
                            markdown.push('\n');
                        }
                    }
                    ContentType::Plot { title, description, .. } => {
                        markdown.push_str(&format!("### {}\n\n", title));
                        markdown.push_str(&format!("{}\n\n", description));
                        markdown.push_str("*[Plot visualization would appear here]*\n\n");
                    }
                    ContentType::Chart { title, description, .. } => {
                        markdown.push_str(&format!("### {}\n\n", title));
                        markdown.push_str(&format!("{}\n\n", description));
                        markdown.push_str("*[Chart visualization would appear here]*\n\n");
                    }
                    ContentType::Image { title, description, .. } => {
                        markdown.push_str(&format!("### {}\n\n", title));
                        markdown.push_str(&format!("{}\n\n", description));
                        markdown.push_str("*[Image would appear here]*\n\n");
                    }
                }
            }
        }
        
        markdown
    }
    
    pub fn export_to_html(&self) -> String {
        let markdown = self.export_to_markdown();
        
        // Simple markdown to HTML conversion
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", self.title));
        html.push_str("<style>\n");
        html.push_str(include_str!("report_styles.css"));
        html.push_str("</style>\n</head>\n<body>\n");
        
        // Convert markdown to HTML (simplified)
        let html_content = markdown
            .replace("# ", "<h1>")
            .replace("\n", "</h1>\n")
            .replace("## ", "<h2>")
            .replace("### ", "<h3>")
            .replace("**", "<strong>")
            .replace("*", "<em>");
        
        html.push_str(&html_content);
        html.push_str("</body>\n</html>");
        
        html
    }
}

/// Report builder for creating reports from analysis results
pub struct ReportBuilder {
    report: Report,
}

impl ReportBuilder {
    pub fn new(title: String, report_type: ReportType, author: String) -> Self {
        Self {
            report: Report::new(title, report_type, author),
        }
    }
    
    pub fn add_data_summary(&mut self, row_count: usize, column_count: usize, memory_usage: usize) -> &mut Self {
        if let Some(summary_section) = self.report.sections.iter_mut().find(|s| matches!(s.section_type, SectionType::Summary)) {
            let summary_text = format!(
                "This report analyzes a dataset containing **{} rows** and **{} columns**, \
                with a total memory footprint of **{:.2} MB**.\n\n\
                The analysis includes statistical summaries, correlation analysis, outlier detection, \
                and data quality assessment.",
                row_count,
                column_count,
                memory_usage as f64 / (1024.0 * 1024.0)
            );
            summary_section.add_markdown(summary_text);
        }
        self
    }
    
    /*
    pub fn add_statistical_summary(&mut self, statistics: Vec<StatisticalSummary>) -> &mut Self {
        // Method commented out - StatisticalSummary type not available
        self
    }
    
    pub fn add_correlation_analysis(&mut self, correlation: Option<CorrelationMatrix>) -> &mut Self {
        // Method commented out - CorrelationMatrix type not available
        self
    }
    
    pub fn add_outlier_analysis(&mut self, outliers: Vec<OutlierAnalysis>) -> &mut Self {
        // Method commented out - OutlierAnalysis type not available
        self
    }
    
    pub fn add_data_quality_assessment(&mut self, quality_report: DataQualityReport) -> &mut Self {
        // Method commented out - DataQualityReport type not available
        self
    }
    */
    
    pub fn add_recommendations(&mut self, recommendations: Vec<String>) -> &mut Self {
        if let Some(rec_section) = self.report.sections.iter_mut().find(|s| matches!(s.section_type, SectionType::Recommendations)) {
            let mut rec_text = String::new();
            rec_text.push_str("Based on the analysis results, the following recommendations are provided:\n\n");
            
            for (i, recommendation) in recommendations.iter().enumerate() {
                rec_text.push_str(&format!("{}. {}\n", i + 1, recommendation));
            }
            
            rec_section.add_markdown(rec_text);
        }
        self
    }
    
    pub fn build(self) -> Report {
        self.report
    }
}

/// Report manager for handling multiple reports
pub struct ReportManager {
    reports: Vec<Report>,
    current_report: Option<String>,
}

impl ReportManager {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            current_report: None,
        }
    }
    
    pub fn create_report(&mut self, title: String, report_type: ReportType, author: String) -> String {
        let report = Report::new(title, report_type, author);
        let report_id = report.id.clone();
        self.reports.push(report);
        self.current_report = Some(report_id.clone());
        report_id
    }
    
    pub fn get_report(&self, report_id: &str) -> Option<&Report> {
        self.reports.iter().find(|r| r.id == report_id)
    }
    
    pub fn get_report_mut(&mut self, report_id: &str) -> Option<&mut Report> {
        self.reports.iter_mut().find(|r| r.id == report_id)
    }
    
    pub fn delete_report(&mut self, report_id: &str) {
        self.reports.retain(|r| r.id != report_id);
        if self.current_report.as_ref() == Some(&report_id.to_string()) {
            self.current_report = None;
        }
    }
    
    pub fn list_reports(&self) -> &[Report] {
        &self.reports
    }
    
    pub fn show_report_list(&mut self, ui: &mut Ui) {
        ui.heading("Reports");
        
        ui.horizontal(|ui| {
            if ui.button("📄 New Report").clicked() {
                // TODO: Show new report dialog
            }
            if ui.button("📁 Import").clicked() {
                // TODO: Import report
            }
        });
        
        ui.separator();
        
        ScrollArea::vertical().show(ui, |ui| {
            for report in &self.reports {
                ui.horizontal(|ui| {
                    let is_current = self.current_report.as_ref() == Some(&report.id);
                    
                    if ui.selectable_label(is_current, &report.title).clicked() {
                        self.current_report = Some(report.id.clone());
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑").clicked() {
                            // TODO: Confirm delete
                        }
                        if ui.small_button("📤").clicked() {
                            // TODO: Export report
                        }
                        
                        ui.label(format!("{}", report.report_type));
                        ui.label(report.created_at.format("%Y-%m-%d").to_string());
                    });
                });
                
                ui.separator();
            }
        });
    }
    
    pub fn show_current_report(&mut self, ui: &mut Ui) {
        if let Some(report_id) = self.current_report.clone() {
            // Find the report index
            let report_index = self.reports.iter().position(|r| r.id == report_id);
            if let Some(idx) = report_index {
                self.show_report_editor_for_id(ui, idx);
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No report selected. Create a new report or select an existing one.");
            });
        }
    }
    
    fn show_report_editor(&mut self, ui: &mut Ui, report: &mut Report) {
        // Report header
        ui.horizontal(|ui| {
            ui.heading(&report.title);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾 Save").clicked() {
                    // TODO: Save report
                }
                if ui.button("📤 Export").clicked() {
                    // TODO: Export report
                }
                if ui.button("🔍 Preview").clicked() {
                    // TODO: Show preview
                }
                if ui.button("➕ Add Section").clicked() {
                    report.add_section(SectionType::Custom("New Section".to_string()), "New Section".to_string());
                }
            });
        });
        
        ui.separator();
        
        // Report metadata
        ui.collapsing("Report Information", |ui| {
            ui.label(format!("Type: {}", report.report_type));
            ui.label(format!("Author: {}", report.author));
            ui.label(format!("Created: {}", report.created_at.format("%Y-%m-%d %H:%M UTC")));
            ui.label(format!("Last Modified: {}", report.last_modified.format("%Y-%m-%d %H:%M UTC")));
        });
        
        ui.separator();
        
        // Report sections
        ScrollArea::vertical().show(ui, |ui| {
            let mut sections_to_delete = Vec::new();
            
            for (i, section) in report.sections.iter_mut().enumerate() {
                let section_response = self.show_section_editor(ui, section);
                
                if section_response.delete_requested {
                    sections_to_delete.push(i);
                }
            }
            
            // Delete sections (in reverse order)
            for &index in sections_to_delete.iter().rev() {
                if index < report.sections.len() {
                    report.sections.remove(index);
                    report.reorder_sections();
                }
            }
        });
    }
    
    fn show_report_editor_for_id(&mut self, ui: &mut Ui, report_idx: usize) {
        if report_idx < self.reports.len() {
            // Get report data we need
            let report_title = self.reports[report_idx].title.clone();
            let report_type = self.reports[report_idx].report_type.clone();
            
            ui.heading(&report_title);
            ui.label(format!("Type: {}", report_type));
            ui.separator();
            
            // Collect section indices to delete
            let mut sections_to_delete = Vec::new();
            
            // Show sections - can't call methods on self while borrowing reports
            let num_sections = self.reports[report_idx].sections.len();
            for section_idx in 0..num_sections {
                let section = &mut self.reports[report_idx].sections[section_idx];
                
                // Show section inline
                let mut delete_requested = false;
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&section.title);
                        if ui.small_button("🗑️").clicked() {
                            delete_requested = true;
                        }
                    });
                    
                    // Show content inline instead of calling a method
                    for content in &section.content {
                        match content {
                            ContentType::Text(text) => {
                                ui.label(text);
                            }
                            ContentType::Markdown(markdown) => {
                                ui.label(format!("📝 Markdown: {}", markdown));
                            }
                            ContentType::Table { title, headers, rows } => {
                                ui.label(format!("📊 Table: {} ({} rows)", title, rows.len()));
                            }
                            _ => {
                                ui.label("Content");
                            }
                        }
                    }
                });
                
                if delete_requested {
                    sections_to_delete.push(section_idx);
                }
            }
            
            // Delete sections after iteration
            for idx in sections_to_delete.into_iter().rev() {
                self.reports[report_idx].sections.remove(idx);
            }
        }
    }
    
    fn show_section_editor(&mut self, ui: &mut Ui, section: &mut ReportSection) -> SectionResponse {
        let mut response = SectionResponse::default();
        
        egui::Frame::none()
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .inner_margin(8.0)
            .show(ui, |ui| {
                // Section header
                ui.horizontal(|ui| {
                    ui.checkbox(&mut section.visible, "");
                    
                    ui.add(TextEdit::singleline(&mut section.title).desired_width(200.0));
                    
                    ui.label(format!("{}", section.section_type));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🗑").clicked() {
                            response.delete_requested = true;
                        }
                        if ui.small_button("➕").clicked() {
                            section.add_text("New content...".to_string());
                        }
                    });
                });
                
                ui.separator();
                
                // Section content
                for (i, content) in section.content.iter_mut().enumerate() {
                    self.show_content_editor(ui, content, i);
                    ui.add_space(5.0);
                }
            });
        
        ui.add_space(10.0);
        response
    }
    
    fn show_content_editor(&mut self, ui: &mut Ui, content: &mut ContentType, _index: usize) {
        match content {
            ContentType::Text(text) => {
                ui.add(TextEdit::multiline(text).desired_rows(3));
            }
            ContentType::Markdown(markdown) => {
                ui.label("Markdown Content:");
                ui.add(TextEdit::multiline(markdown).desired_rows(5));
            }
            ContentType::Table { title, headers, rows } => {
                ui.label(format!("Table: {}", title));
                ui.label(format!("{} columns, {} rows", headers.len(), rows.len()));
                // TODO: Show table editor
            }
            ContentType::Statistics { title, metrics, insights } => {
                ui.label(format!("Statistics: {}", title));
                ui.label(format!("{} metrics, {} insights", metrics.len(), insights.len()));
                // TODO: Show statistics editor
            }
            ContentType::Plot { title, description, .. } => {
                ui.label(format!("Plot: {}", title));
                ui.add(TextEdit::multiline(description).desired_rows(2));
            }
            ContentType::Chart { title, description, .. } => {
                ui.label(format!("Chart: {}", title));
                ui.add(TextEdit::multiline(description).desired_rows(2));
            }
            ContentType::Image { title, description, .. } => {
                ui.label(format!("Image: {}", title));
                ui.add(TextEdit::multiline(description).desired_rows(2));
            }
        }
    }
}

#[derive(Default)]
struct SectionResponse {
    delete_requested: bool,
}

// Re-export types from analysis module
// use pika_engine::analysis::{StatisticalSummary, CorrelationMatrix, OutlierAnalysis, OutlierMethod, DataQualityReport};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_creation() {
        let report = Report::new(
            "Test Report".to_string(),
            ReportType::DataAnalysis,
            "Test Author".to_string()
        );
        
        assert_eq!(report.title, "Test Report");
        assert_eq!(report.author, "Test Author");
        assert!(!report.sections.is_empty());
    }
    
    #[test]
    fn test_report_builder() {
        let mut builder = ReportBuilder::new(
            "Analysis Report".to_string(),
            ReportType::DataAnalysis,
            "Analyst".to_string()
        );
        
        builder.add_data_summary(1000, 5, 1024000);
        let report = builder.build();
        
        assert_eq!(report.title, "Analysis Report");
        assert!(!report.sections.is_empty());
    }
} 