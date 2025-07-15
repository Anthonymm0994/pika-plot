## 🟣 Workspace Canvas (Main Panel)

**📛 Name:** Canvas or Workspace  
**📍 Location:** Center of the UI

**🎯 Purpose:**  
The main interactive area for dropping data sources, building visualizations, and annotating with drawings, shapes, or notes.

**✅ Behavior:**

- Starts empty — user adds a **data source** first (table/view)
    
- Clicking a data source in the left panel adds a **Table Node** centered in the canvas
    
- A Table Node displays a **paginated data preview** (like your screenshot, e.g. 25 rows at a time)
    
- You can **right-click a table node** to create:
    
    - **Plot nodes** (histogram, scatter, etc.)
        
    - Each plot connects to the table via a **bezier curve**
        
    - Curves are styled/colored to visually group all plots from the same data source
        
- **Multiple plots per data source** are allowed
    
    - When a user updates the SQL query inside the table node, **all connected plots update automatically**
        
- **Text, shapes, lines, drawings:**
    
    - These are free-floating, non-connected elements
        
    - Useful for annotating trends, circling outliers, drawing arrows across plots, etc.
        
    - These **do not attach** to table/plot nodes and are intentionally separate
      
    * These should work and feel similar to how it feels to use Paint or Excalidraw
        

**🎨 Styling Ideas:**

- Color theme matching between:
    
    - Bezier curve
        
    - Data source node border
        
    - Plot node accent
        
- Could help users visually trace where each plot came from
    

