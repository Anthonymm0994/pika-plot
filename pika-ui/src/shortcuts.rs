//! Keyboard shortcuts system for Pika-Plot

use egui::{Context, Key, Modifiers};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Available shortcut actions in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    // File operations
    ImportData,
    SaveWorkspace,
    LoadWorkspace,
    ExportPlot,
    NewWorkspace,
    
    // Edit operations
    Undo,
    Redo,
    Copy,
    Paste,
    Delete,
    SelectAll,
    
    // View operations
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomReset,
    ToggleDataPanel,
    TogglePropertiesPanel,
    ToggleFullscreen,
    
    // Navigation
    FocusSearch,
    NextNode,
    PreviousNode,
    
    // Tools
    OpenCommandPalette,
    ClearCache,
    ShowMemoryDialog,
    ToggleDebugOverlay,
    
    // Quick actions
    CreateScatterPlot,
    CreateLinePlot,
    CreateBarChart,
    CreateHistogram,
}

/// Manages keyboard shortcuts for the application
pub struct ShortcutManager {
    shortcuts: HashMap<(Modifiers, Key), ShortcutAction>,
    enabled: bool,
}

impl ShortcutManager {
    /// Create a new shortcut manager with default shortcuts
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();
        
        // File operations
        shortcuts.insert((Modifiers::CTRL, Key::O), ShortcutAction::ImportData);
        shortcuts.insert((Modifiers::CTRL, Key::S), ShortcutAction::SaveWorkspace);
        shortcuts.insert((Modifiers::CTRL, Key::L), ShortcutAction::LoadWorkspace);
        shortcuts.insert((Modifiers::CTRL | Modifiers::SHIFT, Key::E), ShortcutAction::ExportPlot);
        shortcuts.insert((Modifiers::CTRL, Key::N), ShortcutAction::NewWorkspace);
        
        // Edit operations
        shortcuts.insert((Modifiers::CTRL, Key::Z), ShortcutAction::Undo);
        shortcuts.insert((Modifiers::CTRL | Modifiers::SHIFT, Key::Z), ShortcutAction::Redo);
        shortcuts.insert((Modifiers::CTRL, Key::C), ShortcutAction::Copy);
        shortcuts.insert((Modifiers::CTRL, Key::V), ShortcutAction::Paste);
        shortcuts.insert((Modifiers::NONE, Key::Delete), ShortcutAction::Delete);
        shortcuts.insert((Modifiers::CTRL, Key::A), ShortcutAction::SelectAll);
        
        // View operations
        shortcuts.insert((Modifiers::CTRL, Key::Equals), ShortcutAction::ZoomIn);
        shortcuts.insert((Modifiers::CTRL, Key::Minus), ShortcutAction::ZoomOut);
        shortcuts.insert((Modifiers::CTRL, Key::Num0), ShortcutAction::ZoomReset);
        shortcuts.insert((Modifiers::CTRL, Key::F), ShortcutAction::ZoomFit);
        shortcuts.insert((Modifiers::NONE, Key::F9), ShortcutAction::ToggleDataPanel);
        shortcuts.insert((Modifiers::NONE, Key::F10), ShortcutAction::TogglePropertiesPanel);
        shortcuts.insert((Modifiers::NONE, Key::F11), ShortcutAction::ToggleFullscreen);
        
        // Navigation
        shortcuts.insert((Modifiers::CTRL, Key::K), ShortcutAction::OpenCommandPalette);
        shortcuts.insert((Modifiers::CTRL, Key::F), ShortcutAction::FocusSearch);
        shortcuts.insert((Modifiers::NONE, Key::Tab), ShortcutAction::NextNode);
        shortcuts.insert((Modifiers::SHIFT, Key::Tab), ShortcutAction::PreviousNode);
        
        // Tools
        shortcuts.insert((Modifiers::CTRL | Modifiers::SHIFT, Key::C), ShortcutAction::ClearCache);
        shortcuts.insert((Modifiers::CTRL | Modifiers::SHIFT, Key::M), ShortcutAction::ShowMemoryDialog);
        shortcuts.insert((Modifiers::NONE, Key::F12), ShortcutAction::ToggleDebugOverlay);
        
        // Quick plot creation
        shortcuts.insert((Modifiers::CTRL, Key::Num1), ShortcutAction::CreateScatterPlot);
        shortcuts.insert((Modifiers::CTRL, Key::Num2), ShortcutAction::CreateLinePlot);
        shortcuts.insert((Modifiers::CTRL, Key::Num3), ShortcutAction::CreateBarChart);
        shortcuts.insert((Modifiers::CTRL, Key::Num4), ShortcutAction::CreateHistogram);
        
        Self { shortcuts, enabled: true }
    }
    
    /// Handle keyboard input and return triggered action
    pub fn handle_input(&self, ctx: &Context) -> Option<ShortcutAction> {
        if !self.enabled {
            return None;
        }
        
        ctx.input(|i| {
            for ((modifiers, key), action) in &self.shortcuts {
                if i.modifiers == *modifiers && i.key_pressed(*key) {
                    return Some(*action);
                }
            }
            None
        })
    }
    
    /// Get the shortcut text for display (e.g., "Ctrl+O")
    pub fn get_shortcut_text(&self, action: ShortcutAction) -> Option<String> {
        for ((modifiers, key), shortcut_action) in &self.shortcuts {
            if *shortcut_action == action {
                return Some(format_shortcut(*modifiers, *key));
            }
        }
        None
    }
    
    /// Get all shortcuts grouped by category
    pub fn get_shortcuts_by_category(&self) -> Vec<(String, Vec<(ShortcutAction, String)>)> {
        let mut categories = vec![
            ("File".to_string(), Vec::new()),
            ("Edit".to_string(), Vec::new()),
            ("View".to_string(), Vec::new()),
            ("Navigation".to_string(), Vec::new()),
            ("Tools".to_string(), Vec::new()),
            ("Quick Actions".to_string(), Vec::new()),
        ];
        
        for ((modifiers, key), action) in &self.shortcuts {
            let shortcut_text = format_shortcut(*modifiers, *key);
            let category_index = match action {
                ShortcutAction::ImportData | ShortcutAction::SaveWorkspace | 
                ShortcutAction::LoadWorkspace | ShortcutAction::ExportPlot | 
                ShortcutAction::NewWorkspace => 0,
                
                ShortcutAction::Undo | ShortcutAction::Redo | ShortcutAction::Copy | 
                ShortcutAction::Paste | ShortcutAction::Delete | ShortcutAction::SelectAll => 1,
                
                ShortcutAction::ZoomIn | ShortcutAction::ZoomOut | ShortcutAction::ZoomFit | 
                ShortcutAction::ZoomReset | ShortcutAction::ToggleDataPanel | 
                ShortcutAction::TogglePropertiesPanel | ShortcutAction::ToggleFullscreen => 2,
                
                ShortcutAction::FocusSearch | ShortcutAction::NextNode | 
                ShortcutAction::PreviousNode | ShortcutAction::OpenCommandPalette => 3,
                
                ShortcutAction::ClearCache | ShortcutAction::ShowMemoryDialog | 
                ShortcutAction::ToggleDebugOverlay => 4,
                
                ShortcutAction::CreateScatterPlot | ShortcutAction::CreateLinePlot | 
                ShortcutAction::CreateBarChart | ShortcutAction::CreateHistogram => 5,
            };
            
            categories[category_index].1.push((*action, shortcut_text));
        }
        
        categories
    }
    
    /// Enable or disable shortcuts
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if shortcuts are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Add a custom shortcut
    pub fn add_shortcut(&mut self, modifiers: Modifiers, key: Key, action: ShortcutAction) {
        self.shortcuts.insert((modifiers, key), action);
    }
    
    /// Remove a shortcut
    pub fn remove_shortcut(&mut self, modifiers: Modifiers, key: Key) {
        self.shortcuts.remove(&(modifiers, key));
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format shortcut for display
fn format_shortcut(modifiers: Modifiers, key: Key) -> String {
    let mut parts = Vec::new();
    
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.mac_cmd {
        parts.push("Cmd");
    }
    
    // Format key name
    let key_name = match key {
        Key::Equals => "=",
        Key::Minus => "-",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Delete => "Del",
        Key::Tab => "Tab",
        _ => return format!("{:?}", key).replace("Key::", ""),
    };
    
    parts.push(key_name);
    parts.join("+")
}

/// Get the action name for display
pub fn get_action_name(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::ImportData => "Import Data",
        ShortcutAction::SaveWorkspace => "Save Workspace",
        ShortcutAction::LoadWorkspace => "Load Workspace",
        ShortcutAction::ExportPlot => "Export Plot",
        ShortcutAction::NewWorkspace => "New Workspace",
        ShortcutAction::Undo => "Undo",
        ShortcutAction::Redo => "Redo",
        ShortcutAction::Copy => "Copy",
        ShortcutAction::Paste => "Paste",
        ShortcutAction::Delete => "Delete",
        ShortcutAction::SelectAll => "Select All",
        ShortcutAction::ZoomIn => "Zoom In",
        ShortcutAction::ZoomOut => "Zoom Out",
        ShortcutAction::ZoomFit => "Zoom to Fit",
        ShortcutAction::ZoomReset => "Reset Zoom",
        ShortcutAction::ToggleDataPanel => "Toggle Data Panel",
        ShortcutAction::TogglePropertiesPanel => "Toggle Properties Panel",
        ShortcutAction::ToggleFullscreen => "Toggle Fullscreen",
        ShortcutAction::FocusSearch => "Focus Search",
        ShortcutAction::NextNode => "Next Node",
        ShortcutAction::PreviousNode => "Previous Node",
        ShortcutAction::OpenCommandPalette => "Open Command Palette",
        ShortcutAction::ClearCache => "Clear Cache",
        ShortcutAction::ShowMemoryDialog => "Show Memory Dialog",
        ShortcutAction::ToggleDebugOverlay => "Toggle Debug Overlay",
        ShortcutAction::CreateScatterPlot => "Create Scatter Plot",
        ShortcutAction::CreateLinePlot => "Create Line Plot",
        ShortcutAction::CreateBarChart => "Create Bar Chart",
        ShortcutAction::CreateHistogram => "Create Histogram",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shortcut_manager_creation() {
        let manager = ShortcutManager::new();
        assert!(manager.is_enabled());
        assert!(!manager.shortcuts.is_empty());
    }
    
    #[test]
    fn test_shortcut_text_formatting() {
        let manager = ShortcutManager::new();
        
        // Test that shortcuts are formatted correctly
        let open_text = manager.get_shortcut_text(ShortcutAction::ImportData);
        assert_eq!(open_text, Some("Ctrl+O".to_string())); // Full shortcut format
        
        let save_text = manager.get_shortcut_text(ShortcutAction::SaveWorkspace);
        assert_eq!(save_text, Some("Ctrl+S".to_string()));
        
        let scatter_text = manager.get_shortcut_text(ShortcutAction::CreateScatterPlot);
        assert_eq!(scatter_text, Some("Ctrl+1".to_string())); // This has a shortcut: Ctrl+1
    }
    
    #[test]
    fn test_categories() {
        let manager = ShortcutManager::new();
        let categories = manager.get_shortcuts_by_category();
        assert_eq!(categories.len(), 6);
        assert_eq!(categories[0].0, "File");
        assert!(!categories[0].1.is_empty());
    }
    
    #[test]
    fn test_custom_shortcuts() {
        let mut manager = ShortcutManager::new();
        manager.add_shortcut(Modifiers::CTRL, Key::Q, ShortcutAction::ImportData);
        
        // Should have the new shortcut
        assert!(manager.shortcuts.contains_key(&(Modifiers::CTRL, Key::Q)));
    }
} 