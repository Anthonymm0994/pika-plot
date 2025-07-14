## 🟢 Properties Panel (Right Sidebar)

**📛 Name:** Properties Panel  
**📍 Location:** Right edge of the screen

**🎯 Purpose:**  
Displays editable properties for the currently selected node on the canvas.

**✅ Behavior:**

- When **nothing is selected**:  
    Shows message like “No node selected”
    
- When a **node is selected**:  
    Displays appropriate fields depending on type:
    
    - **Table node:**
        
        - File path / database origin
            
        - Rows & columns
            
        - Schema (column names + types)
            
    - **Plot node:**
        
        - Plot type
            
        - X / Y column mapping
            
        - Style settings (color, size, aggregation)
            
        - Link to its parent data source
            
    - **Text / shape:**
        
        - Font, content, alignment, fill, etc.
            

**🚫 Canvas Info (at bottom) has been removed.**  
We won’t pin persistent canvas stats to this panel — those are lightweight and can appear elsewhere (like in the View menu or in the main toolbar if necessary).

---
