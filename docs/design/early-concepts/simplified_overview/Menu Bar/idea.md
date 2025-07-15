## 🧭 **Pika-Plot Menu Bar Design Overview 

---

### 🔲 Menu Bar Layout (Updated Mockup)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│ File     Edit     View     Data     Help                                                    │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 📁 File

_Workspace and data import/export._

- **New Workspace...**
    
- **Open Database...**
    
- **Import CSV...**
    
    > _Also visible in the left Data Sources panel at all times._
    
- **Save Project**
    
- **Save Project As...**
    
- ~~Export Canvas as Image~~ _(Removed — canvas is zoomable/pannable)_
    
- **Exit** _(Optional — may be redundant)_
    

---

### ✏️ Edit

_Canvas-level actions._

- **Undo / Redo**
    
- **Cut / Copy / Paste**
    
- **Select All**
    
- ~~Delete Nodes~~
    
    > _Handled via right-click or Delete key instead._
    

---

### 🖼️ View

_Zoom, layout, and navigation._

- **Zoom In / Zoom Out** (scroll wheel preferred)
    
- **Reset Zoom**
    
- **Center View on Selection**
    
- **Toggle Grid / Snap to Grid**
    
- **Canvas Mode / Notebook Mode**
    
    > _Notebook = paginated storytelling mode_
    

---

### 📊 Data

_Insights and state summary of the current workspace._

- **List of Active Data Sources**
    
- **Connected Plots**
    
- **Query Validity (OK/Error)**
    
- **Unconnected Nodes**
    
- **Notes / Annotations**
    
- _(Later: memory usage, table size, schema info, etc.)_
    

---

### ❓ Help

_User guidance and support._

- **About**
    
- **Keyboard Shortcuts**
    
- **Tutorial / Walkthrough**
    
- **Open Logs**
    
- **Documentation**
    

---

This is now clean, consistent, and minimal — with everything serving a clear purpose. Ready to spec out behavior or mock the data panel if you want.