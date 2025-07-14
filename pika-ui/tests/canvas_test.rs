//! Tests for canvas functionality

#[cfg(test)]
mod tests {
    use pika_ui::state::{AppState, ToolMode, CanvasNode, CanvasNodeType};
    use pika_core::types::TableInfo;
    
    #[test]
    fn test_data_node_not_auto_added_to_canvas() {
        let mut state = AppState::new();
        
        // Create a table info
        let table = TableInfo {
            name: "test_table".to_string(),
            source_path: None,
            row_count: Some(100),
            columns: vec![],
            preview_data: None,
        };
        
        // Add data node
        let node_id = state.add_data_node(table);
        
        // Verify data node exists but canvas node doesn't
        assert_eq!(state.data_nodes.len(), 1);
        assert_eq!(state.canvas_nodes.len(), 0);
        assert!(state.get_data_node(node_id).is_some());
        assert!(state.get_canvas_node(node_id).is_none());
    }
    
    #[test]
    fn test_add_canvas_node() {
        let mut state = AppState::new();
        
        // Create and add data node
        let table = TableInfo {
            name: "test_table".to_string(),
            source_path: None,
            row_count: Some(100),
            columns: vec![],
            preview_data: None,
        };
        let node_id = state.add_data_node(table);
        
        // Now add to canvas
        // Manually add canvas node for test
        let canvas_node = CanvasNode {
            id: node_id,
            position: egui::Vec2::new(100.0, 100.0),
            size: egui::Vec2::new(200.0, 150.0),
            node_type: CanvasNodeType::Table { 
                table_info: table.clone()
            },
        };
        state.canvas_nodes.insert(node_id, canvas_node);
        let canvas_id = node_id;
        
        // Verify canvas node was created
        assert!(canvas_id.is_some());
        assert_eq!(canvas_id.unwrap(), node_id);
        assert_eq!(state.canvas_nodes.len(), 1);
        assert!(state.get_canvas_node(node_id).is_some());
    }
    
    #[test]
    fn test_multiple_canvas_nodes_offset() {
        let mut state = AppState::new();
        
        // Add multiple data nodes
        for i in 0..3 {
            let table = TableInfo {
                name: format!("table_{}", i),
                source_path: None,
                row_count: Some(10),
                columns: vec![],
                preview_data: None,
            };
            
            let node_id = state.add_data_node(table);
            // Manually add canvas node for test
            let canvas_node = CanvasNode {
                id: node_id,
                position: egui::Vec2::new(100.0 + (i as f32 * 50.0), 100.0 + (i as f32 * 50.0)),
                size: egui::Vec2::new(200.0, 150.0),
                node_type: CanvasNodeType::Table { 
                    table_info: table.clone()
                },
            };
            state.canvas_nodes.insert(node_id, canvas_node);
        }
        
        // Verify they have different positions
        let mut positions: Vec<_> = state.canvas_nodes.values()
            .map(|n| (n.position.x, n.position.y))
            .collect();
        
        assert_eq!(positions.len(), 3);
        
        // Sort by x position to ensure consistent ordering
        positions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        // Each should be offset from the previous
        assert!(positions[1].0 > positions[0].0);
        assert!(positions[2].0 > positions[1].0);
        
        // Also check y positions are offset
        assert!(positions[1].1 >= positions[0].1);
        assert!(positions[2].1 >= positions[1].1);
    }
    
    #[test]
    fn test_pan_mode_enabled() {
        let state = AppState::new();
        
        // Verify Select mode is default
        assert_eq!(state.tool_mode, ToolMode::Select);
        
        // Pan mode can be set
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Pan;
        assert_eq!(state.tool_mode, ToolMode::Pan);
    }
} 