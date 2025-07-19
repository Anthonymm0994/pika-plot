use std::sync::Arc;
use std::io::Write;
use egui;
use crate::core::{Database, QueryResult};

pub struct QueryWindow {
    id: egui::Id,
    title: String,
    query: String,
    result: Option<QueryResult>,
    error: Option<String>,
    page: usize,
    page_size: usize,
    export_format: ExportFormat,
    show_export_menu: bool,
    export_mode: ExportMode,
    add_plot_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExportFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExportMode {
    Page,
    All,
}

impl QueryWindow {
    pub fn new(window_id: usize, title: String, initial_query: String) -> Self {
        Self {
            id: egui::Id::new(format!("query_window_{}", window_id)),
            title,
            query: initial_query,
            result: None,
            error: None,
            page: 0,
            page_size: 25,
            export_format: ExportFormat::Csv,
            show_export_menu: false,
            export_mode: ExportMode::Page,
            add_plot_requested: false,
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context, db: Arc<Database>) -> bool {
        let mut open = true;
        
        egui::Window::new(&self.title)
            .id(self.id)
            .default_size([600.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .open(&mut open)
            .show(ctx, |ui| {
                // Set darker background for query window
                ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_gray(22);
                
                // Query editor section with even darker background
                ui.group(|ui| {
                    ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_gray(18);
                    
                    ui.label("SQL Query:");
                    
                    let text_height = 16.0;
                    let line_count = self.query.lines().count().max(1);
                    let _desired_height = text_height * line_count as f32 + 8.0;
                    let _max_height = 150.0; // Cap the maximum height
                    
                    let response = egui::TextEdit::multiline(&mut self.query)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .min_size(egui::vec2(0.0, text_height + 8.0))
                        .desired_rows(line_count.min(8))
                        .show(ui);
                    
                    // Execute on Ctrl+Enter
                    if response.response.has_focus() 
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl) {
                        self.execute_query(db.clone());
                    }
                });
                
                ui.separator();
                
                // Error display
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("✗ Error: {}", error));
                    ui.separator();
                }
                
                // Results section with darker background
                if let Some(results) = &self.result {
                    let available_height = ui.available_height() - 60.0; // Reserve space for controls
                    
                    ui.group(|ui| {
                        ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_gray(18);
                        ui.set_max_height(available_height);
                        
                        // Results header
                        ui.horizontal(|ui| {
                            let total_rows = results.total_rows.unwrap_or(results.rows.len());
                            ui.label(format!(
                                "Results: {} rows (showing {}-{})",
                                total_rows,
                                self.page * self.page_size + 1,
                                ((self.page + 1) * self.page_size).min(total_rows)
                            ));
                        });
                        
                        ui.separator();
                        
                        // Results table with scroll
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                self.render_results_table(ui, results);
                            });
                    });
                }
                
                // Controls section at bottom - always visible
                ui.add_space(4.0);
                ui.separator();
                ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            if ui.button("Previous")
                                .on_hover_text("Go to previous page")
                                .clicked() && self.page > 0 {
                                self.page = self.page.saturating_sub(1);
                                self.execute_query(db.clone());
                            }
                            
                            let total_pages = if let Some(result) = &self.result {
                                if let Some(total_rows) = result.total_rows {
                                    ((total_rows as f32) / (self.page_size as f32)).ceil() as usize
                                } else {
                                    1
                                }
                            } else {
                                1
                            };
                            
                            ui.label(format!("Page {} of {}", self.page + 1, total_pages));
                            
                            if ui.button("Next")
                                .on_hover_text("Go to next page")
                                .clicked() && self.page + 1 < total_pages {
                                self.page += 1;
                                self.execute_query(db.clone());
                            }
                            
                            ui.separator();
                            
                            ui.label("Page size:");
                            egui::ComboBox::from_label("page_size")
                                .selected_text(self.page_size.to_string())
                                .show_ui(ui, |ui| {
                                    for size in &[10, 25, 50, 100, 500] {
                                        ui.selectable_value(&mut self.page_size, *size, size.to_string());
                                    }
                                });
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Add Plot").clicked() {
                                self.add_plot_requested = true;
                            }
                            
                            if ui.button("Export All").clicked() {
                                self.export_all_csv(db.clone());
                            }
                            
                            if ui.button("Export Page").clicked() {
                                self.export_page_csv();
                            }
                        });
                    });
                });
        
        // Auto-execute initial query when window opens
        if self.page == 0 && self.result.is_none() && !self.query.is_empty() {
            self.execute_query(db);
        }
        
        open
    }
    
    pub fn check_plot_request(&mut self) -> bool {
        let requested = self.add_plot_requested;
        self.add_plot_requested = false;
        requested
    }
    
    pub fn get_current_result(&self) -> Option<&QueryResult> {
        self.result.as_ref()
    }
    
    fn render_results_table(&self, ui: &mut egui::Ui, result: &QueryResult) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        let available_width = ui.available_width();
        let num_columns = result.columns.len();
        let column_width = if num_columns > 0 {
            (available_width / num_columns as f32).max(100.0)
        } else {
            100.0
        };
        
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .columns(egui_extras::Column::initial(column_width).resizable(true), result.columns.len())
                .header(20.0, |mut header| {
                    for column in &result.columns {
                        header.col(|ui| {
                            ui.strong(column);
                        });
                    }
                })
                .body(|mut body| {
                    for row in &result.rows {
                        body.row(text_height + 8.0, |mut row_ui| {
                            for value in row {
                                row_ui.col(|ui| {
                                    ui.label(value);
                                });
                            }
                        });
                    }
                });
        });
    }
    
    fn execute_query(&mut self, db: Arc<Database>) {
        self.error = None;
        // self.is_executing = true; // This line was removed from imports, so it's removed here.
        
        match crate::core::QueryExecutor::execute_with_pagination(&db, &self.query, self.page, self.page_size) {
            Ok(result) => {
                self.result = Some(result);
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.result = None;
            }
        }
        
        // self.is_executing = false; // This line was removed from imports, so it's removed here.
    }
    
    fn export_page_csv(&self) {
        if let Some(result) = &self.result {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("csv", &["csv"])
                .set_file_name(&format!("{}_page_{}.csv", self.title, self.page + 1))
                .save_file()
            {
                if let Ok(mut writer) = crate::core::CsvWriter::from_path(&path) {
                    // Write headers
                    let _ = writer.write_headers(&result.columns);
                    
                    // Write rows from current page
                    for row in &result.rows {
                        let _ = writer.write_record(row);
                    }
                    
                    let _ = writer.flush();
                }
            }
        }
    }
    
    fn export_page_json(&self) {
        if let Some(result) = &self.result {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("json", &["json"])
                .set_file_name(&format!("{}_page_{}.json", self.title, self.page + 1))
                .save_file()
            {
                let json_data: Vec<serde_json::Map<String, serde_json::Value>> = result.rows
                    .iter()
                    .map(|row| {
                        let mut map = serde_json::Map::new();
                        for (i, value) in row.iter().enumerate() {
                            if let Some(column) = result.columns.get(i) {
                                map.insert(column.clone(), serde_json::Value::String(value.clone()));
                            }
                        }
                        map
                    })
                    .collect();
                
                if let Ok(json_string) = serde_json::to_string_pretty(&json_data) {
                    let _ = std::fs::File::create(path).and_then(|mut file| file.write_all(json_string.as_bytes()));
                }
            }
        }
    }
    
    fn export_all_csv(&self, db: Arc<Database>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("csv", &["csv"])
            .set_file_name(&format!("{}_all.csv", self.title))
            .save_file()
        {
            // Execute query without pagination to get all results
            match crate::core::QueryExecutor::execute(&db, &self.query) {
                Ok(all_results) => {
                    if let Ok(mut writer) = crate::core::CsvWriter::from_path(&path) {
                        // Write headers
                        let _ = writer.write_headers(&all_results.columns);
                        
                        // Write all rows
                        for row in &all_results.rows {
                            let _ = writer.write_record(row);
                        }
                        
                        let _ = writer.flush();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to export all results: {}", e);
                }
            }
        }
    }
    
    fn export_all_json(&self, db: Arc<Database>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("json", &["json"])
            .set_file_name(&format!("{}_all.json", self.title))
            .save_file()
        {
            // Execute query without pagination to get all results
            match crate::core::QueryExecutor::execute(&db, &self.query) {
                Ok(all_results) => {
                    let json_data: Vec<serde_json::Map<String, serde_json::Value>> = all_results.rows
                        .iter()
                        .map(|row| {
                            let mut map = serde_json::Map::new();
                            for (i, value) in row.iter().enumerate() {
                                if let Some(column) = all_results.columns.get(i) {
                                    map.insert(column.clone(), serde_json::Value::String(value.clone()));
                                }
                            }
                            map
                        })
                        .collect();
                    
                    if let Ok(json_string) = serde_json::to_string_pretty(&json_data) {
                        let _ = std::fs::File::create(path).and_then(|mut file| file.write_all(json_string.as_bytes()));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to export all results: {}", e);
                }
            }
        }
    }
} 