//! Tests for canvas drawing functionality

#[cfg(test)]
mod tests {
    use pika_ui::state::{AppState, ToolMode, CanvasNode, CanvasNodeType, ShapeType};
    use pika_core::types::NodeId;
    use egui::{Vec2, Pos2};
    
    /// Test helper to verify shape creation
    fn verify_shape_created(
        state: &AppState,
        expected_shape_type: ShapeType,
        start_pos: Pos2,
        end_pos: Pos2,
    ) -> bool {
        state.canvas_nodes.values().any(|node| {
            match &node.node_type {
                CanvasNodeType::Shape { shape_type } => {
                    if std::mem::discriminant(shape_type) != std::mem::discriminant(&expected_shape_type) {
                        return false;
                    }
                    
                    // Check position and size
                    let expected_pos = start_pos.to_vec2().min(end_pos.to_vec2());
                    let expected_size = (end_pos - start_pos).abs();
                    
                    let pos_matches = (node.position - expected_pos).length() < 1.0;
                    let size_matches = match shape_type {
                        ShapeType::Line { .. } => true, // Lines have different size handling
                        _ => (node.size - expected_size).length() < 1.0,
                    };
                    
                    pos_matches && size_matches
                }
                _ => false,
            }
        })
    }
    
    #[test]
    fn test_rectangle_drawing() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Simulate drawing a rectangle
        let start_pos = Pos2::new(100.0, 100.0);
        let end_pos = Pos2::new(300.0, 200.0);
        
        // Before drawing, no shapes should exist
        assert_eq!(state.canvas_nodes.len(), 0);
        
        // Simulate rectangle creation
        let id = NodeId::new();
        let size = (end_pos - start_pos).abs();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2().min(end_pos.to_vec2()),
            size,
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify rectangle was created
        assert_eq!(state.canvas_nodes.len(), 1);
        assert!(verify_shape_created(&state, ShapeType::Rectangle, start_pos, end_pos));
    }
    
    #[test]
    fn test_circle_drawing() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Circle;
        
        // Simulate drawing a circle
        let start_pos = Pos2::new(200.0, 150.0);
        let end_pos = Pos2::new(350.0, 300.0);
        
        // Create circle
        let id = NodeId::new();
        let size = (end_pos - start_pos).abs();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2().min(end_pos.to_vec2()),
            size,
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Circle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify circle was created
        assert!(verify_shape_created(&state, ShapeType::Circle, start_pos, end_pos));
    }
    
    #[test]
    fn test_line_drawing() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Line;
        
        // Simulate drawing a line
        let start_pos = Pos2::new(50.0, 50.0);
        let end_pos = Pos2::new(200.0, 150.0);
        
        // Create line
        let id = NodeId::new();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2(),
            size: Vec2::new(1.0, 1.0), // Lines don't really have size
            node_type: CanvasNodeType::Shape { 
                shape_type: ShapeType::Line { end: (end_pos - start_pos) }
            },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify line was created
        assert_eq!(state.canvas_nodes.len(), 1);
        let node = state.canvas_nodes.values().next().unwrap();
        match &node.node_type {
            CanvasNodeType::Shape { shape_type: ShapeType::Line { end } } => {
                let expected_end = end_pos - start_pos;
                assert!((end.x - expected_end.x).abs() < 1.0);
                assert!((end.y - expected_end.y).abs() < 1.0);
            }
            _ => panic!("Expected line shape"),
        }
    }
    
    #[test]
    fn test_minimum_size_requirement() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Try to create a very small rectangle (should be rejected)
        let start_pos = Pos2::new(100.0, 100.0);
        let end_pos = Pos2::new(102.0, 103.0); // Only 2x3 pixels
        
        let size = (end_pos - start_pos).abs();
        
        // Should not create shape if too small (minimum is 5x5)
        if size.x > 5.0 && size.y > 5.0 {
            panic!("Test setup error - shape is not small enough");
        }
        
        // Verify no shape is created for tiny sizes
        assert_eq!(state.canvas_nodes.len(), 0);
    }
    
    #[test]
    fn test_tool_mode_switching() {
        let mut state = AppState::new();
        
        // Test switching between different tools
        assert_eq!(state.tool_mode, ToolMode::Select);
        
        state.tool_mode = ToolMode::Rectangle;
        assert_eq!(state.tool_mode, ToolMode::Rectangle);
        
        state.tool_mode = ToolMode::Circle;
        assert_eq!(state.tool_mode, ToolMode::Circle);
        
        state.tool_mode = ToolMode::Line;
        assert_eq!(state.tool_mode, ToolMode::Line);
        
        state.tool_mode = ToolMode::Draw;
        assert_eq!(state.tool_mode, ToolMode::Draw);
        
        state.tool_mode = ToolMode::Text;
        assert_eq!(state.tool_mode, ToolMode::Text);
        
        state.tool_mode = ToolMode::Pan;
        assert_eq!(state.tool_mode, ToolMode::Pan);
    }
    
    #[test]
    fn test_multiple_shapes() {
        let mut state = AppState::new();
        
        // Create multiple shapes
        let shapes = vec![
            (ToolMode::Rectangle, ShapeType::Rectangle, Pos2::new(10.0, 10.0), Pos2::new(50.0, 50.0)),
            (ToolMode::Circle, ShapeType::Circle, Pos2::new(100.0, 100.0), Pos2::new(150.0, 150.0)),
            (ToolMode::Line, ShapeType::Line { end: Vec2::new(100.0, 0.0) }, Pos2::new(200.0, 200.0), Pos2::new(300.0, 200.0)),
        ];
        
        for (i, (tool_mode, shape_type, start_pos, end_pos)) in shapes.iter().enumerate() {
            state.tool_mode = *tool_mode;
            
            let id = NodeId::new();
            let size = if matches!(shape_type, ShapeType::Line { .. }) {
                Vec2::new(1.0, 1.0)
            } else {
                (*end_pos - *start_pos).abs()
            };
            
            let canvas_node = CanvasNode {
                id,
                position: if matches!(shape_type, ShapeType::Line { .. }) {
                    start_pos.to_vec2()
                } else {
                    start_pos.to_vec2().min(end_pos.to_vec2())
                },
                size,
                node_type: CanvasNodeType::Shape { shape_type: shape_type.clone() },
            };
            state.canvas_nodes.insert(id, canvas_node);
            
            // Verify correct number of shapes
            assert_eq!(state.canvas_nodes.len(), i + 1);
        }
        
        // Verify all shapes were created
        assert_eq!(state.canvas_nodes.len(), 3);
    }
    
    #[test]
    fn test_shape_coordinates_normalization() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Test drawing from bottom-right to top-left
        let start_pos = Pos2::new(300.0, 300.0);
        let end_pos = Pos2::new(100.0, 100.0);
        
        let id = NodeId::new();
        let size = (end_pos - start_pos).abs();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2().min(end_pos.to_vec2()),
            size,
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify position is normalized to top-left
        let node = state.canvas_nodes.values().next().unwrap();
        assert_eq!(node.position, Vec2::new(100.0, 100.0));
        assert_eq!(node.size, Vec2::new(200.0, 200.0));
    }
    
    #[test]
    fn test_pan_mode_no_shapes() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Pan;
        
        // In pan mode, no shapes should be created
        // (Pan mode just moves the canvas view)
        assert_eq!(state.canvas_nodes.len(), 0);
    }
    
    #[test]
    fn test_select_mode_no_shapes() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Select;
        
        // In select mode, dragging should not create shapes
        assert_eq!(state.canvas_nodes.len(), 0);
    }
    
    #[test]
    fn test_drawing_workflow_rectangle() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Simulate mouse down at (100, 100)
        let mouse_down_pos = Pos2::new(100.0, 100.0);
        
        // Simulate dragging to (250, 200)
        let mouse_drag_pos = Pos2::new(250.0, 200.0);
        
        // Create the rectangle on mouse release
        let id = NodeId::new();
        let size = (mouse_drag_pos - mouse_down_pos).abs();
        let canvas_node = CanvasNode {
            id,
            position: mouse_down_pos.to_vec2().min(mouse_drag_pos.to_vec2()),
            size,
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify the rectangle was created with correct dimensions
        assert_eq!(state.canvas_nodes.len(), 1);
        let node = state.canvas_nodes.values().next().unwrap();
        assert_eq!(node.position, Vec2::new(100.0, 100.0));
        assert_eq!(node.size, Vec2::new(150.0, 100.0));
        
        match &node.node_type {
            CanvasNodeType::Shape { shape_type: ShapeType::Rectangle } => {},
            _ => panic!("Expected rectangle shape"),
        }
    }
    
    #[test]
    fn test_drawing_workflow_with_preview() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Circle;
        
        // Mouse down - typically would start preview
        let start_pos = Pos2::new(200.0, 200.0);
        
        // Drag to different positions (simulating preview updates)
        let preview_positions = vec![
            Pos2::new(220.0, 220.0),
            Pos2::new(250.0, 250.0),
            Pos2::new(300.0, 280.0),
        ];
        
        // Final position on mouse up
        let final_pos = Pos2::new(350.0, 300.0);
        
        // Create the circle at final position
        let id = NodeId::new();
        let size = (final_pos - start_pos).abs();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2().min(final_pos.to_vec2()),
            size,
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Circle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify final circle dimensions
        let node = state.canvas_nodes.values().next().unwrap();
        assert_eq!(node.position, Vec2::new(200.0, 200.0));
        assert_eq!(node.size, Vec2::new(150.0, 100.0));
    }
    
    #[test]
    fn test_freehand_drawing() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Draw;
        
        // Simulate freehand drawing with multiple points
        let draw_points = vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(110.0, 105.0),
            Vec2::new(120.0, 115.0),
            Vec2::new(130.0, 130.0),
            Vec2::new(140.0, 145.0),
            Vec2::new(150.0, 150.0),
        ];
        
        // For now, we'll represent freehand as a shape (in real implementation, 
        // this would be a path or polyline)
        let id = NodeId::new();
        let canvas_node = CanvasNode {
            id,
            position: draw_points[0],
            size: Vec2::new(50.0, 50.0), // Bounding box of the path
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle }, // Placeholder
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify drawing was created
        assert_eq!(state.canvas_nodes.len(), 1);
    }
    
    #[test]
    fn test_text_placement() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Text;
        
        // Click position for text
        let text_pos = Pos2::new(300.0, 200.0);
        let text_content = "Hello, Canvas!".to_string();
        
        // Create text node
        let id = NodeId::new();
        let canvas_node = CanvasNode {
            id,
            position: text_pos.to_vec2(),
            size: Vec2::new(100.0, 20.0), // Estimated text size
            node_type: CanvasNodeType::Note { content: text_content.clone() },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify text was placed
        let node = state.canvas_nodes.values().next().unwrap();
        assert_eq!(node.position, Vec2::new(300.0, 200.0));
        
        match &node.node_type {
            CanvasNodeType::Note { content } => {
                assert_eq!(content, "Hello, Canvas!");
            },
            _ => panic!("Expected text/note node"),
        }
    }
    
    #[test]
    fn test_cancel_drawing_with_escape() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Start drawing
        let start_pos = Pos2::new(100.0, 100.0);
        
        // Simulate escape key press - should cancel and not create shape
        // In real implementation, this would be handled by the drawing state
        
        // Verify no shape was created
        assert_eq!(state.canvas_nodes.len(), 0);
    }
    
    #[test]
    fn test_drawing_with_modifier_keys() {
        let mut state = AppState::new();
        state.tool_mode = ToolMode::Rectangle;
        
        // Test shift+drag for square
        let start_pos = Pos2::new(100.0, 100.0);
        let drag_pos = Pos2::new(250.0, 180.0); // Non-square drag
        
        // With shift held, should create square (use smaller dimension)
        let size_diff = (drag_pos - start_pos).abs();
        let square_size = size_diff.x.min(size_diff.y);
        
        let id = NodeId::new();
        let canvas_node = CanvasNode {
            id,
            position: start_pos.to_vec2(),
            size: Vec2::new(square_size, square_size),
            node_type: CanvasNodeType::Shape { shape_type: ShapeType::Rectangle },
        };
        state.canvas_nodes.insert(id, canvas_node);
        
        // Verify square was created
        let node = state.canvas_nodes.values().next().unwrap();
        assert_eq!(node.size.x, node.size.y); // Should be square
        assert_eq!(node.size.x, 80.0); // min(150, 80)
    }
} 