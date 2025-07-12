use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pika_ui::canvas::{CanvasState, Camera2D};
use pika_core::types::NodeId;
use egui::{pos2, vec2, Rect};

fn benchmark_camera_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("camera_transforms");
    
    let camera = Camera2D::new();
    let screen_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(1920.0, 1080.0));
    
    // Benchmark world to screen transformation
    group.bench_function("world_to_screen", |b| {
        let positions: Vec<_> = (0..1000)
            .map(|i| pos2(i as f32 * 10.0, i as f32 * 10.0))
            .collect();
        
        b.iter(|| {
            for pos in &positions {
                let screen_pos = camera.world_to_screen(black_box(*pos), screen_rect);
                black_box(screen_pos);
            }
        });
    });
    
    // Benchmark screen to world transformation
    group.bench_function("screen_to_world", |b| {
        let positions: Vec<_> = (0..1000)
            .map(|i| pos2(i as f32, i as f32))
            .collect();
        
        b.iter(|| {
            for pos in &positions {
                let world_pos = camera.screen_to_world(black_box(*pos), screen_rect);
                black_box(world_pos);
            }
        });
    });
    
    group.finish();
}

fn benchmark_node_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_operations");
    
    // Create canvas with varying number of nodes
    for node_count in [10, 100, 1000].iter() {
        let mut canvas = CanvasState::new();
        
        // Add nodes
        for i in 0..*node_count {
            let id = NodeId::new();
            let pos = pos2((i % 10) as f32 * 100.0, (i / 10) as f32 * 100.0);
            canvas.add_node(id, pos);
        }
        
        // Benchmark node selection
        group.bench_with_input(
            BenchmarkId::new("select_nodes_in_rect", node_count),
            &canvas,
            |b, canvas| {
                let rect = Rect::from_min_size(pos2(200.0, 200.0), vec2(400.0, 400.0));
                b.iter(|| {
                    let selected = canvas.get_nodes_in_rect(black_box(rect));
                    black_box(selected);
                });
            },
        );
        
        // Benchmark snap to grid
        group.bench_with_input(
            BenchmarkId::new("snap_to_grid", node_count),
            &canvas,
            |b, canvas| {
                let positions: Vec<_> = (0..100)
                    .map(|i| pos2(i as f32 * 3.7, i as f32 * 5.3))
                    .collect();
                
                b.iter(|| {
                    for pos in &positions {
                        let snapped = canvas.snap_to_grid(black_box(*pos));
                        black_box(snapped);
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_connection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_operations");
    
    for connection_count in [10, 100, 1000].iter() {
        let mut canvas = CanvasState::new();
        
        // Create nodes and connections
        let mut node_ids = Vec::new();
        for i in 0..*connection_count + 1 {
            let id = NodeId::new();
            let pos = pos2((i % 10) as f32 * 100.0, (i / 10) as f32 * 100.0);
            canvas.add_node(id, pos);
            node_ids.push(id);
        }
        
        // Create connections
        for i in 0..*connection_count {
            canvas.add_connection(node_ids[i], node_ids[i + 1]);
        }
        
        // Benchmark finding connected nodes
        group.bench_with_input(
            BenchmarkId::new("find_connected_nodes", connection_count),
            &canvas,
            |b, canvas| {
                b.iter(|| {
                    for id in &node_ids[..10.min(node_ids.len())] {
                        let connected = canvas.get_connected_nodes(black_box(*id));
                        black_box(connected);
                    }
                });
            },
        );
        
        // Benchmark connection validation
        group.bench_with_input(
            BenchmarkId::new("validate_connection", connection_count),
            &canvas,
            |b, canvas| {
                b.iter(|| {
                    for i in 0..node_ids.len().min(100) {
                        let from = node_ids[i];
                        let to = node_ids[(i + 5) % node_ids.len()];
                        let valid = canvas.can_connect(black_box(from), black_box(to));
                        black_box(valid);
                    }
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_camera_transforms,
    benchmark_node_operations,
    benchmark_connection_operations
);
criterion_main!(benches); 