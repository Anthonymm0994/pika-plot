//! Integration tests for the complete Pika-Plot workflow

#[cfg(test)]
mod tests {
    use pika_ui::state::{AppState, CanvasNode, CanvasNodeType};
    use pika_core::types::{TableInfo, ColumnInfo, NodeId};
    use pika_core::plots::{PlotConfig, PlotType};
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    /// Create a test CSV file
    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Alice,23.5,A").unwrap();
        writeln!(file, "2,Bob,45.2,B").unwrap();
        writeln!(file, "3,Charlie,31.8,A").unwrap();
        writeln!(file, "4,David,52.1,B").unwrap();
        writeln!(file, "5,Eve,28.9,A").unwrap();
        file.flush().unwrap();
        file
    }
    
    /// Create test table info
    fn create_test_table_info() -> TableInfo {
        TableInfo {
            name: "test_data".to_string(),
            source_path: Some("test_data.csv".into()),
            row_count: Some(5),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                ColumnInfo {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                },
                ColumnInfo {
                    name: "value".to_string(),
                    data_type: "REAL".to_string(),
                    nullable: false,
                },
                ColumnInfo {
                    name: "category".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                },
            ],
            preview_data: None,
        }
    }
    
    #[test]
    fn test_workflow_data_to_canvas() {
        let mut state = AppState::new();
        
        // Step 1: Add data node
        let table_info = create_test_table_info();
        let data_node_id = state.add_data_node(table_info.clone());
        
        // Verify data node exists but not on canvas
        assert_eq!(state.data_nodes.len(), 1);
        assert_eq!(state.canvas_nodes.len(), 0);
        
        // Step 2: Add to canvas
        state.add_canvas_node_for_data(data_node_id);
        
        // Verify canvas node was created
        assert_eq!(state.canvas_nodes.len(), 1);
        let canvas_node = state.canvas_nodes.values().next().unwrap();
        match &canvas_node.node_type {
            CanvasNodeType::Table { table_info: info } => {
                assert_eq!(info.name, "test_data");
            }
            _ => panic!("Expected table node"),
        }
    }
    
    #[test]
    fn test_workflow_create_plot_from_table() {
        let mut state = AppState::new();
        
        // Add data and canvas node
        let table_info = create_test_table_info();
        let data_node_id = state.add_data_node(table_info.clone());
        state.add_canvas_node_for_data(data_node_id);
        
        // Create a plot config (store separately in state)
        let plot_config = PlotConfig {
            plot_type: PlotType::Scatter,
            title: Some("Test Plot".to_string()),
            x_column: "id".to_string(),
            y_label: Some("Value".to_string()),
            ..Default::default()
        };
        
        // Add plot node
        let plot_id = NodeId::new();
        let plot_node = CanvasNode {
            id: plot_id,
            position: egui::Vec2::new(300.0, 100.0),
            size: egui::Vec2::new(400.0, 300.0),
            node_type: CanvasNodeType::Plot {
                plot_type: "Scatter".to_string(),
            },
        };
        state.canvas_nodes.insert(plot_id, plot_node);
        
        // Store the plot config separately
        state.plot_configs.insert(plot_id, plot_config);
        
        // Verify we have both table and plot nodes
        assert_eq!(state.canvas_nodes.len(), 2);
        
        // Count node types
        let mut table_count = 0;
        let mut plot_count = 0;
        for node in state.canvas_nodes.values() {
            match &node.node_type {
                CanvasNodeType::Table { .. } => table_count += 1,
                CanvasNodeType::Plot { .. } => plot_count += 1,
                _ => {}
            }
        }
        assert_eq!(table_count, 1);
        assert_eq!(plot_count, 1);
    }
    
    #[test]
    fn test_multiple_tables_and_plots() {
        let mut state = AppState::new();
        
        // Add multiple tables
        let tables = vec![
            ("sales_data", 100),
            ("customer_data", 50),
            ("product_data", 25),
        ];
        
        let mut table_ids = Vec::new();
        for (name, rows) in tables {
            let table_info = TableInfo {
                name: name.to_string(),
                row_count: Some(rows),
                columns: vec![
                    ColumnInfo {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                    },
                ],
                source_path: Some(format!("{}.csv", name).into()),
                preview_data: None,
            };
            
            let id = state.add_data_node(table_info);
            table_ids.push(id);
        }
        
        // Add all to canvas
        for id in &table_ids {
            state.add_canvas_node_for_data(*id);
        }
        
        // Create plots from each table
        for (i, &table_id) in table_ids.iter().enumerate() {
            let plot_type = match i {
                0 => PlotType::Bar,
                1 => PlotType::Line,
                _ => PlotType::Histogram,
            };
            
            let plot_config = PlotConfig {
                plot_type: plot_type.clone(),
                title: Some(format!("Plot {}", i + 1)),
                x_column: "id".to_string(),
                ..Default::default()
            };
            
            let plot_id = NodeId::new();
            let plot_node = CanvasNode {
                id: plot_id,
                position: egui::Vec2::new(500.0 + i as f32 * 50.0, 100.0 + i as f32 * 50.0),
                size: egui::Vec2::new(400.0, 300.0),
                node_type: CanvasNodeType::Plot {
                    plot_type: format!("{:?}", plot_type),
                },
            };
            state.canvas_nodes.insert(plot_id, plot_node);
            state.plot_configs.insert(plot_id, plot_config);
        }
        
        // Verify counts
        assert_eq!(state.data_nodes.len(), 3);
        assert_eq!(state.canvas_nodes.len(), 6); // 3 tables + 3 plots
    }
    
    #[test]
    fn test_canvas_connections() {
        let mut state = AppState::new();
        
        // Create two table nodes
        let table1 = create_test_table_info();
        let table1_id = state.add_data_node(table1);
        state.add_canvas_node_for_data(table1_id);
        
        let mut table2 = create_test_table_info();
        table2.name = "test_data_2".to_string();
        let table2_id = state.add_data_node(table2);
        state.add_canvas_node_for_data(table2_id);
        
        // Create connection between them
        let connection = pika_ui::state::NodeConnection {
            id: uuid::Uuid::new_v4().to_string(),
            from: table1_id,
            to: table2_id,
            connection_type: pika_ui::state::ConnectionType::DataFlow,
        };
        state.connections.push(connection);
        
        // Verify connection exists
        assert_eq!(state.connections.len(), 1);
        assert_eq!(state.connections[0].from, table1_id);
        assert_eq!(state.connections[0].to, table2_id);
    }
} 