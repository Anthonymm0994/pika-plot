# ID Conflict Fix Summary

## Problem
When importing multiple CSV files in the file configuration dialog, egui was showing ID conflict warnings for various widgets. These warnings appeared as red text overlays on the UI:
- Table ID F835
- ScrollArea ID FC78  
- widget ID 0E74

## Root Cause
Multiple instances of the same widgets were being created without unique IDs. When rendering multiple files, each file's widgets (checkboxes, text edits, radio buttons, etc.) had the same implicit IDs, causing conflicts.

## Solution
Added unique IDs to all widgets that could have multiple instances:

### 1. File Configuration Widgets
- **Table name text edit**: `ui.push_id(format!("table_name_{}", self.current_file_index), |ui| { ... })`
- **Header row drag value**: `ui.push_id(format!("header_row_{}", self.current_file_index), |ui| { ... })`
- **Sample size drag value**: `ui.push_id(format!("sample_size_{}", self.current_file_index), |ui| { ... })`
- **Resample button**: `ui.push_id(format!("resample_{}", self.current_file_index), |ui| { ... })`

### 2. Delimiter Radio Buttons
- **Comma**: `ui.push_id(format!("delimiter_comma_{}", self.current_file_index), |ui| { ... })`
- **Tab**: `ui.push_id(format!("delimiter_tab_{}", self.current_file_index), |ui| { ... })`
- **Semicolon**: `ui.push_id(format!("delimiter_semi_{}", self.current_file_index), |ui| { ... })`
- **Pipe**: `ui.push_id(format!("delimiter_pipe_{}", self.current_file_index), |ui| { ... })`

### 3. Null Value Checkboxes
- **Empty string**: `ui.push_id(format!("empty_string_{}", self.current_file_index), |ui| { ... })`
- **NULL text**: `ui.push_id(format!("null_text_{}", self.current_file_index), |ui| { ... })`
- **Lowercase null**: `ui.push_id(format!("lowercase_null_{}", self.current_file_index), |ui| { ... })`
- **N/A**: `ui.push_id(format!("na_{}", self.current_file_index), |ui| { ... })`

### 4. Column Selection Buttons
- **Select All**: `ui.push_id(format!("select_all_{}", self.current_file_index), |ui| { ... })`
- **Deselect All**: `ui.push_id(format!("deselect_all_{}", self.current_file_index), |ui| { ... })`

### 5. Column Table Checkboxes
For each checkbox in the column configuration table, added unique IDs combining file index and row index:
- **Include checkbox**: `ui.push_id(format!("include_{}_{}", file_idx, row_index), |ui| { ... })`
- **Primary Key checkbox**: `ui.push_id(format!("pk_{}_{}", file_idx, row_index), |ui| { ... })`
- **Not Null checkbox**: `ui.push_id(format!("not_null_{}_{}", file_idx, row_index), |ui| { ... })`
- **Unique checkbox**: `ui.push_id(format!("unique_{}_{}", file_idx, row_index), |ui| { ... })`
- **Index checkbox**: `ui.push_id(format!("index_{}_{}", file_idx, row_index), |ui| { ... })`

### 6. Other Fixes
- **CSV file selector ComboBox**: Changed from `ComboBox::from_label("")` to `ComboBox::from_id_source("csv_file_selector")`
- **Column type ComboBox**: Already had unique IDs: `format!("type_{}_{}", file_idx, row_index)`
- **ScrollArea widgets**: Added unique IDs to ScrollArea widgets in data sources and properties panels

## Implementation Details
Used egui's `push_id` method with closures to create scoped unique IDs. This pattern ensures:
1. Each widget has a unique ID based on its context (file index, row index)
2. IDs are automatically scoped and don't leak to other parts of the UI
3. The solution is maintainable and follows egui best practices

## Result
All ID conflict warnings are resolved. Multiple CSV files can now be imported and configured without any egui ID warnings appearing on the UI. 