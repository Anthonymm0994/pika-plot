//! Basic UI component tests

#[cfg(test)]
mod tests {
    use pika_ui::state::AppState;
    
    #[test]
    fn test_state_creation() {
        let state = AppState::new();
        
        // Initial state should be empty
        assert_eq!(state.data_nodes.len(), 0);
        assert_eq!(state.canvas_nodes.len(), 0);
        assert_eq!(state.connections.len(), 0);
        
        // Default tool should be Select
        assert_eq!(state.tool_mode, pika_ui::state::ToolMode::Select);
    }
    
    #[test]
    fn test_state_data_node_operations() {
        let mut state = AppState::new();
        
        // Create a table info
        let table = pika_core::types::TableInfo {
            name: "test_table".to_string(),
            source_path: None,
            row_count: Some(10),
            columns: vec![],
            preview_data: None,
        };
        
        // Add data node
        let node_id = state.add_data_node(table);
        
        // Check it was added
        assert_eq!(state.data_nodes.len(), 1);
        assert!(state.get_data_node(node_id).is_some());
        
        // Remove it
        state.remove_data_node(node_id);
        assert_eq!(state.data_nodes.len(), 0);
    }
}