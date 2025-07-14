# Canvas Functionality Guide

## Overview

The Pika-Plot canvas provides an interactive workspace for data visualization, combining data analysis with drawing and annotation capabilities similar to tools like Paint or Excalidraw.

## Core Features

### 1. Data Source Management

**Adding Data Sources:**
- Double-click any data source in the left panel to add it to canvas
- Creates a Table Node with live data preview
- Shows first 25 rows with alternating row colors for readability
- Displays column headers and row count

**Table Node Features:**
- Professional data table appearance with dark theme
- Title bar showing data source name
- Paginated preview (automatically sized to node)
- Query editing capability (future feature)

### 2. Visualization Creation

**Creating Plots:**
1. Right-click on any Table Node
2. Select "Add Plot" from context menu
3. Choose from available plot types:
   - Histogram
   - Line Chart
   - Scatter Plot
   - Bar Chart
   - Box Plot
   - Violin Plot
   - Heatmap
   - Correlation Matrix
   - Time Series
   - Radar Chart

**Plot Connections:**
- Automatic bezier curve connections between data and plots
- Visual data flow representation
- Multiple plots can connect to single data source
- Updates propagate through connections

### 3. Drawing Tools

**Available Tools:**
- **Select Tool (S)**: Default tool for selecting and moving nodes
- **Rectangle Tool (R)**: Draw rectangles with live preview
- **Circle Tool (C)**: Draw circles with live preview
- **Line Tool (L)**: Draw straight lines
- **Draw Tool (D)**: Freehand drawing
- **Text Tool (T)**: Add text annotations
- **Pan Tool (P)**: Pan the canvas view

**Drawing Experience:**
- Live preview while drawing (see shape as you drag)
- Shapes appear immediately on first click
- Paint-like interaction model
- All drawings are independent of data nodes

### 4. Canvas Navigation

**Controls:**
- Scroll wheel: Zoom in/out
- Middle mouse button: Pan
- Grid display (toggleable)
- Zoom range: 0.1x to 5.0x

### 5. Canvas Toolbar

**Features:**
- Tool selection buttons
- "Show Grid" checkbox
- Grid visibility persists across sessions
- Visual feedback for active tool

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| S | Select tool |
| R | Rectangle tool |
| C | Circle tool |
| L | Line tool |
| D | Draw tool |
| T | Text tool |
| P | Pan tool |
| Delete | Delete selected item |
| Ctrl+Z | Undo (planned) |
| Ctrl+Y | Redo (planned) |

## Implementation Details

### Architecture
```
Canvas Panel
├── Tool Handling
│   ├── Shape preview during drag
│   ├── Immediate visual feedback
│   └── Tool-specific behaviors
├── Node Management
│   ├── Table nodes with data preview
│   ├── Plot nodes with visualizations
│   └── Shape/drawing nodes
├── Connection System
│   ├── Bezier curve rendering
│   ├── Data flow tracking
│   └── Update propagation
└── Rendering Pipeline
    ├── Background & grid
    ├── Connections
    ├── Nodes
    └── Active drawings
```

### State Management
- Canvas state includes zoom, pan, and grid settings
- Node positions and connections persisted
- Drawing state tracked per tool
- Preview shapes rendered during drag operations

## Visual Design

### Color Scheme
- Background: Dark gray (#1a1a2a)
- Grid: Subtle gray lines
- Table nodes: Dark blue-gray with hover effects
- Connections: Light gray bezier curves
- Selection: Blue highlight (#6496fa)

### Node Styling
- Rounded corners (5px radius)
- Drop shadows for depth
- Hover states for interactivity
- Clear visual hierarchy

## Future Enhancements

1. **Query Editing**: Direct SQL editing in table nodes
2. **Data Refresh**: Live data updates
3. **Export Options**: Save canvas as image/PDF
4. **Templates**: Pre-built analysis layouts
5. **Collaboration**: Multi-user canvas editing

## Best Practices

1. **Organization**: Group related plots near their data source
2. **Annotations**: Use drawing tools to highlight insights
3. **Connections**: Keep connection lines clear and untangled
4. **Naming**: Use descriptive names for data sources
5. **Layout**: Maintain logical flow from data to insights 