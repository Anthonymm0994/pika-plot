# Pika-Plot Documentation

Welcome to the Pika-Plot documentation! This folder contains all the design documents, implementation guides, and research for building our "Excalidraw for Gigabytes of Data" vision.

## 📚 Documentation Structure

### Core Documents
- **[VISION.md](VISION.md)** - The complete vision and architecture overview
- **[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)** - Current implementation status and gaps
- **[CRITICAL_PATH.md](CRITICAL_PATH.md)** - MVP requirements and blockers

### Design Documents
- **[final_ideas/](final_ideas/)** - Refined design decisions and specifications
- **[cross_agent_research/](cross_agent_research/)** - Research from multiple AI agents
- **[rough_ideas/](rough_ideas/)** - Initial brainstorming and concepts

### Technical Guides
- **[cross_agent_research/implementation_patterns.md](cross_agent_research/implementation_patterns.md)** - Ready-to-use code patterns
- **[cross_agent_research/technical_questions_and_insights_*.md](cross_agent_research/)** - Deep technical research

## 🎯 Quick Start for Developers

1. **Understand the Vision**: Start with [VISION.md](VISION.md)
2. **Check Current Status**: Review [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)
3. **Focus on MVP**: Follow [CRITICAL_PATH.md](CRITICAL_PATH.md)
4. **Use Code Patterns**: Reference [implementation_patterns.md](cross_agent_research/implementation_patterns.md)

## 🏗️ Architecture Overview

```
pika-plot/
├── pika-core/      # Core types and shared definitions
├── pika-engine/    # Data processing and GPU coordination
├── pika-ui/        # Canvas-based user interface
├── pika-app/       # Main application entry point
└── pika-cli/       # Command-line interface
```

## 🚀 Key Innovations

1. **Infinite Canvas** - Spatial workspace for data exploration
2. **Visual Threads** - Color-coded connections between operations
3. **GPU Everything** - Not just rendering, but computation
4. **Unified Memory** - Intelligent RAM/VRAM coordination
5. **Offline-First** - No cloud dependencies

## 📊 Performance Targets

- 60 FPS with 1-5M points
- Sub-second queries on GB datasets
- Real-time interaction with cached data
- Graceful degradation on lower-end hardware

## 🔍 Where to Find What

### UI/UX Design
- Canvas interaction: `pika-ui/src/panels/canvas.rs`
- Node system: `pika-ui/src/state.rs`
- Theme: `pika-ui/src/theme.rs`

### Data Processing
- SQL execution: `pika-engine/src/query.rs`
- Import pipeline: `pika-engine/src/import.rs`
- Memory management: `pika-engine/src/memory_coordinator.rs`

### GPU Computing
- Device management: `pika-engine/src/gpu/`
- Shaders: `pika-engine/src/gpu/shaders/`
- Aggregation: `pika-engine/src/aggregation.rs`

## 🤝 Contributing

Before contributing:
1. Read the vision documents
2. Check implementation status
3. Follow the patterns in existing code
4. Test with large datasets (1M+ rows)
5. Ensure Windows 10/11 compatibility

## 📝 Documentation TODO

- [ ] API reference documentation
- [ ] User guide with screenshots
- [ ] Performance tuning guide
- [ ] Plugin development guide
- [ ] Data format specifications 