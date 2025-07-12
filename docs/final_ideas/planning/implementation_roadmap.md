# 🗺️ Pika-Plot Implementation Roadmap

## Overview

This roadmap provides a concrete, week-by-week plan for implementing Pika-Plot from scratch. All features are built in parallel where possible, with no phased rollouts.

## 📅 8-Week Implementation Schedule

### Week 1: Foundation & Setup
**Goal**: Establish project structure and core types

**Tasks**:
- [ ] Create 5-crate workspace structure
- [ ] Set up CI/CD with GitHub Actions
- [ ] Implement all types in `pika-core`
- [ ] Create error taxonomy
- [ ] Set up logging infrastructure
- [ ] Configure development environment

**Deliverables**:
- Compiling workspace with all crates
- Core types fully defined
- Basic test harness running

### Week 2: Engine Core & DuckDB Integration
**Goal**: Data engine with CSV import working

**Tasks**:
- [ ] Implement DuckDB storage engine
- [ ] Create CSV import with type inference
- [ ] Build event system (channels)
- [ ] Implement basic query execution
- [ ] Add memory monitoring (Windows API)
- [ ] Create engine thread runtime

**Deliverables**:
- Can import CSV files via CLI
- Can execute basic SQL queries
- Memory usage tracked accurately

### Week 3: Caching & UI Foundation
**Goal**: Query caching and basic egui window

**Tasks**:
- [ ] Implement 2-tier cache system
- [ ] Create main application window
- [ ] Build workspace structure (dual mode)
- [ ] Implement theme system
- [ ] Create basic toolbar and menus
- [ ] Set up egui-wgpu integration

**Deliverables**:
- Query results cached properly
- Basic UI window with mode toggle
- Dark/light theme working

### Week 4: Node System Implementation
**Goal**: All node types functional in both modes

**Tasks**:
- [ ] Implement Table node (adapt from pebble)
- [ ] Implement Query node with SQL editor
- [ ] Create Plot node shell
- [ ] Build node rendering for both modes
- [ ] Implement canvas drag/drop
- [ ] Create notebook cell management

**Deliverables**:
- Can create and manipulate nodes
- Both UI modes fully functional
- Nodes persist in workspace

### Week 5: GPU Infrastructure & Basic Plots
**Goal**: GPU rendering pipeline with first plots

**Tasks**:
- [ ] Set up wgpu render pipelines
- [ ] Implement vertex buffer layouts
- [ ] Create basic shaders (direct mode)
- [ ] Port scatter plot from frog-viz
- [ ] Port line plot from frog-viz
- [ ] Integrate plots with egui

**Deliverables**:
- Scatter plots rendering with GPU
- Line plots working
- < 50k points rendering smoothly

### Week 6: Advanced Rendering & More Plots
**Goal**: All rendering modes and core plot types

**Tasks**:
- [ ] Implement instanced rendering
- [ ] Create aggregation compute shader
- [ ] Add histogram plot type
- [ ] Add bar chart
- [ ] Add heatmap
- [ ] Implement plot interactions (zoom/pan)

**Deliverables**:
- All three rendering modes working
- 5 core plot types implemented
- Can handle millions of points

### Week 7: Polish & Export Features
**Goal**: Complete user experience

**Tasks**:
- [ ] Implement all export formats
- [ ] Create workspace save/load
- [ ] Add progress indicators
- [ ] Implement error toasts
- [ ] Create keyboard shortcuts
- [ ] Build import dialog UI

**Deliverables**:
- Full import/export working
- Workspace persistence
- Polished error handling

### Week 8: Testing & Optimization
**Goal**: Production-ready release

**Tasks**:
- [ ] Performance optimization pass
- [ ] Memory leak testing
- [ ] Create benchmarks
- [ ] Build release binaries
- [ ] Write CLI commands
- [ ] Final bug fixes

**Deliverables**:
- Optimized release build
- All tests passing
- CLI fully functional

## 🛠️ Implementation Strategy

### Parallel Development Tracks

**Track 1: Core Infrastructure**
- Developer A works on engine and caching
- Focus on data processing pipeline

**Track 2: UI Development**
- Developer B works on UI components
- Focus on user interaction

**Track 3: GPU/Visualization**
- Developer C works on rendering
- Focus on performance

### Daily Sync Points
- Morning: Review overnight CI results
- Midday: Integration testing
- Evening: Merge approved PRs

### Code Reuse Strategy

**From pebble**:
- CSV import dialog
- Table schema display
- File chooser integration
- Query result table widget

**From frog-viz**:
- Plot type implementations
- Color scale algorithms
- Axis calculation logic
- Legend generation

### Testing Approach

**Continuous Testing**:
- Unit tests with each commit
- Integration tests daily
- Performance benchmarks weekly
- Manual testing for UI/UX

**Test Data**:
- Small: 100 rows (unit tests)
- Medium: 10K rows (integration)
- Large: 1M rows (performance)
- Huge: 50M rows (stress tests)

## 📊 Success Metrics

### Week 1-2: Foundation
- [ ] All crates compile
- [ ] Can import a CSV file
- [ ] Basic queries execute

### Week 3-4: Core Features  
- [ ] UI responds < 16ms
- [ ] Mode switching works
- [ ] Nodes can be created

### Week 5-6: Visualization
- [ ] 60 FPS with 1M points
- [ ] All plot types render
- [ ] GPU memory tracked

### Week 7-8: Polish
- [ ] No memory leaks
- [ ] All exports working
- [ ] < 5 second startup

## 🚨 Risk Mitigation

### Technical Risks

**Risk**: GPU driver compatibility
**Mitigation**: Test on multiple GPUs early

**Risk**: Memory usage with large datasets
**Mitigation**: Implement streaming where possible

**Risk**: DuckDB performance
**Mitigation**: Profile queries, add indexes

### Schedule Risks

**Risk**: Integration delays
**Mitigation**: Daily integration, feature flags

**Risk**: GPU shader complexity
**Mitigation**: Start simple, iterate

**Risk**: Windows-specific issues
**Mitigation**: Test on multiple Windows versions

## 🔧 Development Environment

### Required Tools
- Rust 1.75+ (latest stable)
- Visual Studio 2022 (Windows SDK)
- Git with LFS for test data
- DXC shader compiler
- GPU debugging tools

### Recommended Setup
- 32GB RAM development machine
- Discrete GPU (NVIDIA/AMD)
- Windows 11 for best debugging
- Multiple monitors for UI work
- Fast NVMe for large test files

## 📝 Documentation During Development

### Code Documentation
- Doc comments on all public APIs
- Examples in doc tests
- Architecture decision records

### User Documentation
- Screenshot-driven tutorials
- Video of key workflows
- Keyboard shortcut reference

## 🎯 Definition of Done

### Feature Complete When
- [ ] All acceptance criteria met
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Documentation updated
- [ ] Code reviewed and approved

### Release Ready When
- [ ] All P0 bugs fixed
- [ ] Performance targets met
- [ ] Memory leaks eliminated
- [ ] Installer tested
- [ ] Release notes written

## 🚀 Post-Launch Plan

### Week 9+: Maintenance
- Monitor user feedback
- Fix critical bugs
- Optimize based on real usage patterns
- Address any compatibility issues discovered
- Improve documentation based on user questions

This roadmap provides a clear path from empty repository to shipping product in 8 weeks. 