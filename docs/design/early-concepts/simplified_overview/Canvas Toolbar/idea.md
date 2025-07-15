## 🔵 Canvas Toolbar (Top Action Bar)

**📛 Name:** Canvas Toolbar  
**📍 Location:** Horizontal strip at the top of the workspace, under the main menu bar

**🎯 Purpose:**  
Controls how the user interacts with the canvas (selecting, drawing, annotating, etc.)

**🧱 Elements:**

```
[ Pika-Plot Canvas ] [ Select ] [ Rectangle ] [ Circle ] [ Line ] [ Draw ] [ Text ]
Zoom: 1.0x    Elements: 0
```

**✅ Behavior:**

- **Pika-Plot Canvas label:**  
    Just a static label, not clickable — canvas and notebook modes are handled via the `View` menu.
    
- **Interaction tools (Select, Draw, etc.):**
    
    - Only one active at a time
        
    - `Select`: move, resize, highlight items
        
    - `Rectangle / Circle / Line`: draw basic shapes
        
    - `Draw`: freeform drawing for annotations
        
    - `Text`: add text boxes
        
- **Zoom/Elements label:**
    
    - Purely informational
        
    - Can update live with scroll zoom or node additions
        
    - Future: consider optional hotkeys for zoom reset, fit to content, etc.
        

**🚫 Omitted:**

- No `Plot` tool in the toolbar for now — plot creation happens through right-click on data source nodes
    
- No dropdown to switch canvases here
    

---
