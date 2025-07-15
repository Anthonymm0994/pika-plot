## 📂 Left Panel – Data Sources

```
📂 Data Sources
[＋ Import CSV...]
[＋ Open Database...]

▾ Tables
[🔍 Search tables...]
MOCK_DATA_0        ＋
MOCK_DATA_1        ＋
users              ＋
comprehensive_test_data  ＋

▾ Views
analytics_view_1   ＋
revenue_summary    ＋

───────────────
ℹ️ Selected: MOCK_DATA_1  
Source: ~/Documents/MOCK_DATA_1.csv  
Rows: 10,000  
Columns: 6

▾ Column Details
• id (INTEGER) nullable  
• name (TEXT) nullable  
• email (TEXT) nullable  
• age (INTEGER) nullable  
• gender (TEXT) nullable  
• signup_date (DATE) nullable
```

---

### ✅ Behavior

- **Header Buttons (Always Visible):**
    
    - `[＋ Import CSV...]` → opens file picker for one or more CSVs
        
    - `[＋ Open Database...]` → opens SQLite/DuckDB file
        
- **Tables Section**
    
    - Collapsible under `▾ Tables`
        
    - Search bar (`🔍 Search tables...`) filters in real-time
        
    - Each row has:
        
        - **Table name (clickable)**: shows metadata in info panel below
            
        - **Green `＋` button**: adds table node to canvas (centered)
            
    - ✅ No double-click
        
    - ✅ No right-click
        
    - ❌ Drag-and-drop deferred for now (if it isn't hard to implement, it would be very nice!)
        
- **Views Section**
    
    - Same behavior as Tables
        
- **Info Panel (Bottom Section)**
    
    - Shows full metadata for the currently selected table/view
        
    - Includes:
        
        - **File path (if CSV)**
            
        - **Row and column count**
            
        - **Full schema with types and nullability**
            