# UI Component Guide

## Overview
This guide provides a visual and functional overview of all UI components in Pika-Plot. Each component is designed following egui patterns and the dark theme aesthetic.

## Component Hierarchy

```
App (Root)
├── Menu Bar
├── Canvas Toolbar
├── Main Layout (3-column)
│   ├── Left Panel (Data Sources)
│   ├── Center (Canvas)
│   └── Right Panel (Properties)
└── Status Bar
```

## 1. Menu Bar

### Visual Design
- **Background**: Dark gray (#1e1e1e)
- **Text**: Light gray (#e0e0e0)
- **Height**: 24px
- **Font**: System default, 14px

### Menu Structure
```
File | Edit | View | Canvas | Data | Tools | Window | Help
```

### Key Shortcuts
- **File**
  - New Workspace: `Ctrl+N`
  - Open: `Ctrl+O`
  - Save: `Ctrl+S`
  - Import CSV: `Ctrl+I`
  
- **Edit**
  - Undo: `Ctrl+Z`
  - Redo: `Ctrl+Y`
  - Delete: `Delete`
  
- **View**
  - Zoom In: `Ctrl++`
  - Zoom Out: `Ctrl+-`
  - Fit to Screen: `Ctrl+0`

## 2. Canvas Toolbar

### Layout
```
[Pika-Plot Canvas] | [🔲] [▭] [⭕] [╱] [✏️] [📝] | Zoom: 100% | Elements: 5
```

### Tool Descriptions
| Tool | Icon | Purpose | Shortcut | Cursor |
|------|------|---------|----------|---------|
| Select | 🔲 | Select and move nodes | V | Default |
| Rectangle | ▭ | Draw rectangles | R | Crosshair |
| Circle | ⭕ | Draw circles | C | Crosshair |
| Line | ╱ | Draw lines | L | Crosshair |
| Draw | ✏️ | Freehand drawing | D | Pencil |
| Text | 📝 | Add text | T | Text |

### Visual States
- **Normal**: Dark button (#2a2a2a)
- **Hover**: Lighter (#3a3a3a)
- **Active**: Blue highlight (#007ACC)
- **Disabled**: Grayed out (#1a1a1a)

## 3. Data Sources Panel (Left)

### Header
```
Data Sources
[➕ Import CSV...] [➕ Open Database...]
```

### Search Bar
```
🔍 Search tables...
```

### Table List Item
```
┌─────────────────────────┐
│ 📊 sales_data           │
│ 1,234 rows × 8 columns  │
│ Updated: 2 mins ago     │
│                    [+]  │
└─────────────────────────┘
```

### Interaction
- **Hover**: Highlight background (#2a2a2a)
- **Click [+]**: Add table node to canvas
- **Double-click**: Open table preview
- **Right-click**: Context menu (View, Export, Delete)

## 4. Canvas Area (Center)

### Grid System
- **Grid Size**: 20px default
- **Grid Color**: #2a2a2a (subtle)
- **Background**: #1a1a1a

### Node Types

#### Table Node
```
┌─────────────────────┐
│ 📊 Table: orders    │ ○ (output port)
├─────────────────────┤
│ ID  │ Customer │ $  │
│ 001 │ Alice    │ 99 │
│ 002 │ Bob      │ 150│
│ ... │ ...      │... │
└─────────────────────┘
```

#### Plot Node
```
○ ┌─────────────────────┐
   │ 📈 Scatter Plot     │
   ├─────────────────────┤
   │   [Plot Preview]    │
   │     •  •   •        │
   │   •    • •   •      │
   │ •    •     •        │
   └─────────────────────┘
```

#### Shape Node
```
┌─────────────┐
│             │  (No ports)
│   Shape     │
│             │
└─────────────┘
```

### Connection Rendering
- **Style**: Bezier curves
- **Colors**:
  - Data connection: Blue (#007ACC)
  - Plot connection: Green (#4CAF50)
  - Error/Invalid: Red (#F44336)
- **Width**: 2px
- **Hover**: Glow effect

## 5. Properties Panel (Right)

### Dynamic Content
Changes based on selection:

#### When Table Selected
```
Properties: Table
─────────────────
Name: sales_data
Rows: 1,234
Columns: 8

Column Details:
┌──────────┬────────┐
│ Name     │ Type   │
├──────────┼────────┤
│ id       │ Integer│
│ customer │ String │
│ amount   │ Float  │
└──────────┴────────┘
```

#### When Plot Selected
```
Properties: Plot
─────────────────
Type: [Scatter ▼]
Title: [Sales Analysis    ]
X-Axis: [Date ▼]
Y-Axis: [Revenue ▼]

□ Show Legend
☑ Show Grid
□ Dark Theme

[Apply] [Reset]
```

## 6. Status Bar

### Layout
```
Mode: Drawing Rectangle | Pos: (245, 382) | Selection: 2 nodes, 1 connection | Zoom: 125%
```

### Sections
- **Mode**: Current tool/action
- **Position**: Mouse coordinates
- **Selection**: Selected items count
- **Zoom**: Current zoom level

## 7. Modal Dialogs

### CSV Import Dialog
```
┌─ Import CSV Files ──────────────────┐
│                                     │
│ Selected Files:                     │
│ ☑ sales_data.csv                   │
│ ☑ customers.csv                    │
│ ☐ products.csv                     │
│                                     │
│ [Add Files...] [Remove] [Clear]     │
│                                     │
│ Preview: sales_data.csv             │
│ ┌─────────────────────────────┐    │
│ │ id │ date  │ amount │ ...   │    │
│ │ 1  │ 2024  │ 99.50  │ ...   │    │
│ │ 2  │ 2024  │ 150.00 │ ...   │    │
│ └─────────────────────────────┘    │
│                                     │
│ [Cancel] [Back] [Next: Configure]   │
└─────────────────────────────────────┘
```

### File Configuration Screen
```
┌─ Configure: sales_data.csv ─────────────────────────────────┐
│                                                             │
│ Configuration                 │ Preview                     │
│ ─────────────────            │ ─────────────────          │
│ Header Row: [1 ▼]            │ ┌─────────────────────┐    │
│ Sample Size: [100] [Resample]│ │ ID │ Customer │ $   │    │
│                              │ │ 1  │ Alice    │ 99  │    │
│ Delimiter:                   │ │ 2  │ Bob      │ 150 │    │
│ ○ Comma (,) ● Tab ○ Other   │ └─────────────────────┘    │
│                              │                             │
│ Null Values:                 │ Column Configuration:       │
│ ☑ Empty strings ☑ "NULL"     │ ┌───────────────────────┐  │
│ ☑ "N/A" ☐ Custom: [    ]     │ │☑│Column │Type    │PK│ │  │
│                              │ │☑│id     │Integer │☑ │ │  │
│                              │ │☑│customer│String │☐ │ │  │
│                              │ │☑│amount │Float   │☐ │ │  │
│                              │ └───────────────────────┘  │
│                              │                             │
│ [Cancel] [Back] [Create Database with 1 Table]              │
└─────────────────────────────────────────────────────────────┘
```

## 8. Context Menus

### Canvas Right-Click
```
┌─────────────────┐
│ Create Plot  >  │───┐
│ Add Note        │   │ Line
│ ─────────────   │   │ Bar
│ Paste           │   │ Scatter
│ Select All      │   │ ...
│ ─────────────   │
│ Canvas Settings │
└─────────────────┘
```

### Node Right-Click
```
┌─────────────────┐
│ Configure       │
│ Duplicate       │
│ ─────────────   │
│ Bring to Front  │
│ Send to Back    │
│ ─────────────   │
│ Delete          │
└─────────────────┘
```

## 9. Visual Feedback

### Hover Effects
- **Nodes**: Subtle glow outline
- **Connections**: Highlight and show tooltip
- **Buttons**: Lighten background
- **Ports**: Enlarge and highlight

### Selection Indicators
- **Single**: Blue outline (2px)
- **Multiple**: Dashed blue outline
- **Locked**: Red lock icon overlay

### Drag Feedback
- **Valid Drop**: Green highlight
- **Invalid Drop**: Red highlight
- **Snapping**: Blue guide lines

## 10. Responsive Behavior

### Panel Resizing
- **Minimum Widths**:
  - Left panel: 200px
  - Right panel: 250px
  - Canvas: 400px
- **Drag Handles**: 4px wide, hover shows resize cursor

### Zoom Behavior
- **Range**: 10% to 500%
- **Steps**: 10% increments
- **Center**: Zoom centers on mouse position

### Scrolling
- **Canvas**: Infinite scroll with pan
- **Panels**: Vertical scroll only
- **Tables**: Both horizontal and vertical

## Theme Constants

```rust
pub const BACKGROUND: Color32 = Color32::from_gray(26);      // #1a1a1a
pub const SURFACE: Color32 = Color32::from_gray(30);         // #1e1e1e
pub const PANEL_BG: Color32 = Color32::from_gray(35);        // #232323
pub const BUTTON_BG: Color32 = Color32::from_gray(42);       // #2a2a2a
pub const BUTTON_HOVER: Color32 = Color32::from_gray(58);    // #3a3a3a
pub const TEXT_PRIMARY: Color32 = Color32::from_gray(224);   // #e0e0e0
pub const TEXT_SECONDARY: Color32 = Color32::from_gray(160); // #a0a0a0
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0, 122, 204);    // #007ACC
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(76, 175, 80);   // #4CAF50
pub const ERROR_RED: Color32 = Color32::from_rgb(244, 67, 54);      // #F44336
```

## Accessibility

- **Keyboard Navigation**: Tab through all interactive elements
- **Focus Indicators**: Visible outline on focused elements
- **Tooltips**: Descriptive tooltips for all tools and buttons
- **Contrast**: WCAG AA compliant color contrast ratios
- **Screen Reader**: Semantic labels for all UI elements 