# 📚 Pika-Plot Documentation

This directory contains the complete documentation set for implementing Pika-Plot, a high-performance GPU-accelerated data visualization desktop application.

## 🗂️ Documentation Structure

### [📋 Requirements](requirements/)
- **[requirements.md](requirements/requirements.md)** - Non-negotiable project requirements and constraints

### [📐 Planning](planning/)
- **[project_management_guide.md](planning/project_management_guide.md)** - Guide for documentation and implementation approach
- **[implementation_roadmap.md](planning/implementation_roadmap.md)** - Week-by-week development plan
- **[questions.md](planning/questions.md)** - Open design questions with reasonable defaults

### [🔧 Technical](technical/)
- **[architecture_plan.md](technical/architecture_plan.md)** - Complete system architecture specification
- **[system_design.md](technical/system_design.md)** - Detailed architectural design with diagrams
- **[module_breakdown.md](technical/module_breakdown.md)** - Comprehensive breakdown of all crates and modules

#### Technical Specifications
- **[types/core_types.rs](technical/types/core_types.rs)** - All shared Rust type definitions
- **[gpu/aggregation.wgsl](technical/gpu/aggregation.wgsl)** - GPU compute shaders
- **[gpu/vertex_layout.md](technical/gpu/vertex_layout.md)** - Vertex buffer specifications
- **[formats/snapshot.ron](technical/formats/snapshot.ron)** - Workspace snapshot file format
- **[errors/error_types.rs](technical/errors/error_types.rs)** - Error taxonomy with user messages

### [🎨 Design](design/)
- **[ui_ux_specification.md](design/ui_ux_specification.md)** - Complete UI/UX design specifications

### [📖 High-Level](.)
- **[overview.md](overview.md)** - Executive summary and project vision

## 🚀 Quick Start

1. **Requirements First**: Read [requirements/requirements.md](requirements/requirements.md) to understand the non-negotiable constraints
2. **Planning Context**: Review [planning/project_management_guide.md](planning/project_management_guide.md) for approach
3. **Architecture**: Study [technical/architecture_plan.md](technical/architecture_plan.md) for system design
4. **Implementation**: Follow [planning/implementation_roadmap.md](planning/implementation_roadmap.md) week by week

## 📦 Supporting Resources

### Existing Codebases
Located in the repository root:
- **pebble/** - SQLite viewer with egui (reuse UI patterns)
- **frog-viz/** - Plot implementations (reuse visualization logic)

### Test Data
- **fixtures/** - Sample CSV files for testing
  - `small.csv` - 100 rows, clean data
  - `medium.csv` - 50 rows with nulls and edge cases

## 🎯 Key Principles

Per the [requirements](requirements/requirements.md):
1. **Complete system delivery** - No phased rollouts
2. **Discrete GPU required** - No integrated graphics support
3. **Offline-only** - No server/cloud features
4. **Dual UI modes** - Notebook and canvas from day one
5. **Windows native** - Rust + egui for Windows 10/11
6. **DuckDB exclusive** - No SQLite, no backend abstraction

## 📝 Documentation Guidelines

- All planning documents reside in `docs/final_ideas/`
- Open questions go in [planning/questions.md](planning/questions.md)
- Follow structure in [planning/project_management_guide.md](planning/project_management_guide.md)
- Maintain vision-driven, ambitious approach

## ✅ Implementation Ready

This documentation provides everything needed to build Pika-Plot:
- Complete type definitions
- GPU shader specifications
- UI/UX mockups
- Error handling strategy
- Module organization
- Clear development roadmap

The project is fully specified and ready for implementation. 