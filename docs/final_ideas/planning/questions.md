# ❓ Open Questions for Pika-Plot Implementation

This document lists questions and decision points that may need clarification during implementation. These are not blockers - reasonable defaults can be assumed - but explicit answers would improve the implementation.

## 📊 Plot Implementation Details

### 1. Plot Type Priority

**Question**: Which plot types should be implemented first?
**Current Assumption**: Start with Scatter, Line, Bar, Histogram, Heatmap
**Impact**: Affects initial feature completeness

### 2. Plot Customization Depth

**Question**: How much customization should be exposed?
**Current Assumption**: Basic options (colors, labels, scales) with room for more customization
**Impact**: UI complexity and development time

### 3. Color Palette Sources

**Question**: Should we use standard palettes (Viridis, etc.) or custom ones?
**Current Assumption**: Use established scientific color palettes from matplotlib/plotly
**Impact**: Visual consistency and accessibility

## 🗄️ Data Handling

### 4. Maximum File Size

**Question**: What's the hard limit for CSV file size?
**Current Assumption**: 2GB file size limit (to fit comfortably in RAM)
**Impact**: Import validation and user messaging

### 5. Supported Data Types

**Question**: Should we support complex types (JSON columns, arrays)?
**Current Assumption**: Basic types only (numeric, string, date, boolean)
**Impact**: DuckDB schema handling complexity
**Note**: Much of this logic is already implemented in Pebble and can be reused.

### 6. Missing Data Handling

**Question**: How to visualize NULL/missing values?
**Current Assumption**: Exclude from plots by default, option to show gaps
**Impact**: Plot rendering logic

## 🖥️ GPU Requirements

### 7. Minimum GPU Specifications

**Updated Answer**: While we are assuming the presence of a discrete GPU, we do want the application to work on slightly older machines still running Windows 10. Minimum spec should target:

* **DirectX 11 support**
* **2GB VRAM minimum**
* Compatibility with NVIDIA GTX 900 series or AMD equivalent
  We’ll test against some mid-range GPUs from 2015–2017 to ensure functionality. If possible, supporting even older GPUs (e.g., early DirectX 11 cards from \~2012) and Windows 7 would be a nice bonus if it doesn’t complicate development.

### 8. GPU Fallback Behavior

**Question**: What happens if GPU runs out of memory?
**Current Assumption**: Show error and suggest reducing data/plot complexity
**Impact**: Error handling and user experience

## 🎨 UI/UX Decisions

### 9. Icon Set

**Question**: Use emoji, Font Awesome, or custom icons?
**Current Assumption**: Use egui's built-in icons + selective emoji
**Impact**: Visual consistency and rendering
**Note**: Be cautious with emojis — many are not recognized on some systems and may render as squares. Icons should favor reliability and clarity over novelty.

### 10. Default Window Size

**Question**: What's the ideal default window size?
**Current Assumption**: 1280×720, remember last size
**Impact**: First-run experience

### 11. Multi-Monitor Support

**Question**: How to handle plots on different monitors?
**Current Assumption**: Window follows standard OS behavior
**Impact**: GPU context management

## 🔧 Technical Decisions

### 12. Log File Location

**Question**: Where to store logs and how much to keep?
**Current Assumption**: `%APPDATA%/PikaPlot/logs/`, keep last 7 days
**Impact**: Debugging and support

### 13. Telemetry

**Question**: Any anonymous usage statistics?
**Current Assumption**: No telemetry (fully offline)
**Impact**: Future product improvement insights

### 14. Auto-Update

**Question**: Should the app check for updates?
**Current Assumption**: No auto-update (manual only)
**Impact**: User experience and maintenance

## 📦 Distribution

### 15. Installation Method

**Updated Answer**: A **single portable `.exe` file** is the preferred and ideal delivery format. No installer should be required. The app should be self-contained and runnable from any directory.
**Note**: That said, an installer is acceptable if absolutely necessary. The goal is frictionless offline execution — prioritize simplicity.

### 16. Code Signing

**Question**: Will the executable be signed?
**Current Assumption**: Unsigned
**Impact**: Windows SmartScreen warnings

## 🧪 Testing Data

### 17. Sample Datasets

**Question**: Should we bundle example datasets?
**Current Assumption**: No bundled data, user provides all
**Impact**: First-run experience and demos

### 18. Benchmark Datasets

**Question**: What datasets to use for performance testing?
**Current Assumption**: Generate synthetic data for benchmarks
**Impact**: Performance validation

## 🔄 Workflow Questions

### 19. Undo/Redo Scope

**Question**: What operations should be undoable?
**Current Assumption**: Only canvas node operations, not data operations
**Impact**: Memory usage and complexity

### 20. Auto-Save

**Question**: Should workspaces auto-save?
**Current Assumption**: No auto-save, explicit save only
**Impact**: Data loss prevention

## 📋 Export Options

### 21. Image Export DPI

**Question**: Maximum DPI for image exports?
**Current Assumption**: Max 300 DPI for print quality
**Impact**: Memory usage during export

### 22. SVG Complexity

**Question**: How many points before switching to raster in SVG?
**Current Assumption**: Max 10,000 points in SVG
**Impact**: File size and rendering performance

## 🎯 Future-Proofing

### 23. Plugin Architecture

**Question**: Should the application include plugin hooks?
**Current Assumption**: No plugin system
**Impact**: Extensibility

### 24. File Format Versioning

**Question**: How to handle future .pikaplot format changes?
**Current Assumption**: Version field, backward compatible reading
**Impact**: Long-term file compatibility

## 📝 Notes

* These questions are **not blockers** - reasonable defaults are provided
* Decisions can be revisited based on user feedback
* Focus on shipping with all core features working well
* Document decisions as they're made for future reference

## 🔍 Clarifications from Requirements

### Interactive Brushing Implementation

**Requirement**: UI components must support "interactive brushing"
**Clarification**: This refers to the ability to select data points in one plot and have related data highlight in other connected plots/tables
**Implementation Note**: Will require shared selection state between connected nodes

### Default UI Mode

**Requirement**: "Notebook mode is the default"
**Confirmed**: Users start in notebook mode, can switch to canvas mode via toggle

### DuckDB for Metadata

**Requirement**: DuckDB should be used for "metadata management"
**Clarification**: Store table schemas, column statistics, and data lineage information in DuckDB system tables

### No SQLite Usage

**Requirement**: "Do not use SQLite in the new system"
**Confirmed**: Only reuse UI/UX patterns from pebble, not its SQLite backend
