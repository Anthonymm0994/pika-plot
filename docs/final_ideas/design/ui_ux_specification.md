# 🎨 Pika-Plot UI/UX Design Specification

## Overview

Pika-Plot's interface is designed to be powerful yet approachable, supporting both guided (notebook) and freeform (canvas) workflows. The design emphasizes clarity, performance feedback, and seamless mode switching.

## 🎯 Design Principles

1. **Progressive Disclosure**: Simple tasks are obvious, advanced features are discoverable
2. **Visual Feedback**: Every action has immediate visual response
3. **Consistent Metaphors**: Similar concepts look and behave similarly
4. **Performance Visibility**: Users see when the system is working
5. **Error Prevention**: Guide users away from mistakes before they happen

## 🎨 Visual Design System

### Color Palette

#### Dark Theme (Default)
```css
/* Background layers */
--bg-primary: #1a1b26      /* Main background */
--bg-secondary: #24283b    /* Panels, cards */
--bg-tertiary: #1f2335     /* Hover states */

/* Text */
--text-primary: #c0caf5    /* Main text */
--text-secondary: #a9b1d6  /* Secondary text */
--text-disabled: #565f89   /* Disabled text */

/* Accent colors */
--accent-primary: #7aa2f7  /* Primary actions */
--accent-success: #9ece6a  /* Success states */
--accent-warning: #e0af68  /* Warnings */
--accent-error: #f7768e    /* Errors */
--accent-info: #7dcfff     /* Information */

/* Node type colors */
--node-table: #bb9af7      /* Purple - data sources */
--node-query: #7aa2f7      /* Blue - transformations */
--node-plot: #9ece6a       /* Green - visualizations */
--node-transform: #e0af68  /* Yellow - data ops */
--node-export: #73daca     /* Teal - outputs */
```

#### Light Theme
```css
/* Inverse of dark theme with adjusted contrast */
--bg-primary: #ffffff
--bg-secondary: #f6f6f8
--bg-tertiary: #e9e9ed
/* ... etc */
```

### Typography

```css
/* Font stack */
--font-sans: "Inter", -apple-system, BlinkMacSystemFont, sans-serif;
--font-mono: "JetBrains Mono", "Consolas", monospace;

/* Font sizes */
--text-xs: 11px;
--text-sm: 13px;
--text-base: 14px;
--text-lg: 16px;
--text-xl: 20px;
--text-2xl: 24px;

/* Font weights */
--font-normal: 400;
--font-medium: 500;
--font-bold: 700;
```

### Spacing System

```css
/* 4px base unit */
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
--space-8: 32px;
--space-10: 40px;
```

### Border Radius

```css
--radius-sm: 4px;
--radius-md: 6px;
--radius-lg: 8px;
--radius-xl: 12px;
```

## 🖼️ Layout Structure

### Application Shell

```
┌─────────────────────────────────────────────────────────┐
│ Title Bar                                    [_][□][X]  │
├─────────────────────────────────────────────────────────┤
│ Menu Bar  │ File  Edit  View  Tools  Help              │
├───────────┴─────────────────────────────────────────────┤
│ Toolbar   │ [Import] [Query] [Plot] │ [Notebook][Canvas]│
├───────────┴─────────────────────────────────────────────┤
│                                                         │
│                    Main Workspace                       │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ Status Bar │ 125,432 rows │ 8.2GB used │ 60 FPS       │
└─────────────────────────────────────────────────────────┘
```

### Toolbar Design

**Primary Actions** (Left side):
- Import CSV - Icon: 📁
- New Query - Icon: 🔍
- Add Plot - Icon: 📊

**Mode Toggle** (Right side):
- Radio button style toggle
- Smooth transition animation (200ms)
- Current mode highlighted with accent color

## 📓 Notebook Mode Design

### Cell Structure

```
┌─────────────────────────────────────────────┐
│ [▼] Cell 1: Import Sales Data          [⋮] │
├─────────────────────────────────────────────┤
│ 📁 sales_2024.csv                           │
│ 125,000 rows × 5 columns                    │
│ Last modified: 2024-01-15                   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ [▶] Cell 2: Monthly Summary            [⋮] │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ [▼] Cell 3: Sales by Product          [⋮] │
├─────────────────────────────────────────────┤
│      [Scatter Plot Visualization]           │
│                                             │
│      ● ● ●   ●                             │
│    ●     ● ●   ●                           │
│  ●   ●       ●   ●                         │
│                                             │
└─────────────────────────────────────────────┘
```

### Cell Types Visual Distinction

- **Table Cell**: Purple left border, database icon
- **Query Cell**: Blue left border, search icon
- **Plot Cell**: Green left border, chart icon
- **Markdown Cell**: Gray left border, document icon

### Cell Interactions

- **Hover**: Highlight with bg-tertiary
- **Selected**: 2px accent border
- **Executing**: Pulsing border animation
- **Error**: Red border with error message below

### Keyboard Navigation

- `↑/↓`: Navigate between cells
- `Enter`: Edit selected cell
- `Shift+Enter`: Execute cell
- `Ctrl+Enter`: Execute and advance
- `A`: Add cell above
- `B`: Add cell below
- `D,D`: Delete cell
- `M`: Convert to markdown
- `Y`: Convert to code

## 🎨 Canvas Mode Design

### Node Appearance

```
┌─────────────────────────┐
│ ● Table: sales_2024     │
├─────────────────────────┤
│ 📁 125,000 rows         │
│ 5 columns               │
│                         │
│ ○ data        filter ○  │
└─────────────────────────┘
```

**Node Components**:
- Header: Icon + Type + Name
- Body: Key information
- Ports: Input (left) and Output (right)
- Resize handle: Bottom-right corner

### Node States

- **Default**: Standard colors
- **Hover**: 10% brightness increase
- **Selected**: 2px accent outline
- **Executing**: Animated gradient border
- **Error**: Red outline with tooltip

### Connection Drawing

```
[Output Port] ●━━━━━┓
                    ┃
                    ┗━━━━━● [Input Port]
```

**Connection Behavior**:
- Bezier curves with auto-routing
- Highlight on hover
- Animate data flow during execution
- Different colors for different data types

### Canvas Controls

**Pan**: 
- Middle mouse drag
- Space + left mouse drag
- Two-finger trackpad gesture

**Zoom**:
- Mouse wheel
- Ctrl + Plus/Minus
- Pinch gesture

**Selection**:
- Click: Select single
- Ctrl+Click: Add to selection
- Drag: Box select
- Ctrl+A: Select all

### Minimap

```
┌──────────┐
│ ┌─┐  ┌─┐ │  Position: Bottom-right
│ └─┘  └─┘ │  Size: 200×150px
│     ▄     │  Current view highlighted
└──────────┘
```

## 📊 Plot Rendering Area

### Plot Container

```
┌─────────────────────────────────────────┐
│ Sales Trend 2024              [⚙️] [📤] │ <- Title bar with actions
├─────────────────────────────────────────┤
│                                         │
│         [GPU-Rendered Plot]             │ <- Main plot area
│                                         │
├─────────────────────────────────────────┤
│ ● Product A  ● Product B  ● Product C   │ <- Legend
└─────────────────────────────────────────┘
```

### Interactive Features

- **Hover**: Tooltip with exact values
- **Pan**: Click and drag
- **Zoom**: Mouse wheel or box select
- **Select**: Click points or box select
- **Context Menu**: Right-click for options

### Performance Indicators

**Render Mode Badge**:
- Direct: "Direct (50k points)"
- Instanced: "GPU Optimized (2.5M points)"
- Aggregated: "Aggregated (50M points)"

**FPS Counter**: Top-right corner when < 60 FPS

## 🎛️ Dialogs and Modals

### Import CSV Dialog

```
┌──────────────────────────────────────────┐
│ Import CSV Data                     [X] │
├──────────────────────────────────────────┤
│ File: sales_2024.csv                     │
│ Size: 15.2 MB                           │
│                                         │
│ ▼ Import Options                        │
│ ┌───────────────────────────────────┐   │
│ │ ☑ Has Header Row                  │   │
│ │ Delimiter: [,▼]                   │   │
│ │ Encoding: [UTF-8▼]                │   │
│ │ Skip Rows: [0    ]                │   │
│ └───────────────────────────────────┘   │
│                                         │
│ ▼ Preview (first 10 rows)              │
│ ┌───────────────────────────────────┐   │
│ │ date  | product | quantity | ...  │   │
│ │ ───────┼─────────┼──────────┼──── │   │
│ │ 2024.. | Widget A| 125      | ... │   │
│ └───────────────────────────────────┘   │
│                                         │
│ [Cancel]                    [Import]    │
└──────────────────────────────────────────┘
```

### Export Dialog

```
┌──────────────────────────────────────────┐
│ Export Plot                         [X] │
├──────────────────────────────────────────┤
│ Format: [PNG        ▼]                  │
│                                         │
│ ▼ Image Options                        │
│ ┌───────────────────────────────────┐   │
│ │ Width:  [1920    ] px             │   │
│ │ Height: [1080    ] px             │   │
│ │ DPI:    [144     ]                │   │
│ │ ☑ Transparent Background          │   │
│ └───────────────────────────────────┘   │
│                                         │
│ Save to: C:\exports\plot.png    [...]  │
│                                         │
│ [Cancel]                    [Export]    │
└──────────────────────────────────────────┘
```

## 🔔 Notifications and Feedback

### Toast Notifications

Position: Top-right corner
Types:
- Success: Green with ✓ icon
- Warning: Yellow with ⚠ icon
- Error: Red with ✗ icon
- Info: Blue with ℹ icon

### Progress Indicators

**Import Progress**:
```
Importing sales_2024.csv...
[████████████░░░░░░░] 75% - 94,325 rows
```

**Query Execution**:
```
Executing query...
[Spinner animation] Elapsed: 2.3s
```

## 🎯 Responsive Behavior

### Window Resizing

- Minimum window size: 800×600
- Panels collapse to icons at < 1024px width
- Canvas minimap hides at < 1200px width
- Maintain aspect ratios for plots

### High DPI Support

- Detect system DPI settings
- Scale UI elements appropriately
- Use vector icons where possible
- Ensure text remains crisp

## ⌨️ Keyboard Shortcuts

### Global Shortcuts

- `Ctrl+N`: New workspace
- `Ctrl+O`: Open CSV
- `Ctrl+S`: Save workspace
- `Ctrl+Shift+S`: Save as
- `Ctrl+Z/Y`: Undo/Redo
- `F1`: Show help
- `F2`: Toggle mode
- `F11`: Fullscreen

### Mode-Specific Shortcuts

Listed in respective sections above

## 🌐 Accessibility

### Screen Reader Support

- All interactive elements have aria-labels
- Logical tab order
- Announce state changes
- Describe chart data in alt text

### Keyboard Navigation

- All features accessible via keyboard
- Visual focus indicators
- Skip links for main content
- Consistent navigation patterns

### Color Contrast

- WCAG AAA compliance for text
- Don't rely solely on color
- Patterns for color-blind users
- High contrast mode support

## 🎭 Animation and Transitions

### Timing Functions

```css
--ease-out: cubic-bezier(0.0, 0.0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0.0, 0.2, 1);
```

### Standard Durations

- Instant: 0ms (state changes)
- Fast: 100ms (hover effects)
- Normal: 200ms (panel slides)
- Slow: 300ms (mode transitions)

### Animation Guidelines

- Use for state changes, not decoration
- Respect prefers-reduced-motion
- Keep animations subtle
- Ensure animations don't block interaction

This design specification provides a comprehensive foundation for implementing Pika-Plot's user interface while maintaining consistency and usability across all features. 