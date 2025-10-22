use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dagrers::{DagreLayout, LayoutOptions, RankDir};
use petgraph::Graph;

fn create_large_dag(nodes: usize, edges_per_node: usize) -> Graph<String, ()> {
    let mut graph = Graph::new();
    
    // Create nodes
    let node_indices: Vec<_> = (0..nodes)
        .map(|i| graph.add_node(format!("Node{}", i)))
        .collect();
    
    // Create edges - connect each node to several nodes in the next "layer"
    let layers = (nodes as f64).sqrt() as usize;
    let nodes_per_layer = nodes / layers;
    
    for layer in 0..layers-1 {
        let layer_start = layer * nodes_per_layer;
        let layer_end = ((layer + 1) * nodes_per_layer).min(nodes);
        let next_layer_start = layer_end;
        let next_layer_end = ((layer + 2) * nodes_per_layer).min(nodes);
        
        for i in layer_start..layer_end {
            for j in 0..edges_per_node {
                if next_layer_start < next_layer_end {
                    let target_idx = next_layer_start + (j % (next_layer_end - next_layer_start));
                    if target_idx < nodes {
                        graph.add_edge(node_indices[i], node_indices[target_idx], ());
                    }
                }
            }
        }
    }
    
    graph
}

fn create_dense_dag(size: usize) -> Graph<String, ()> {
    let mut graph = Graph::new();
    
    // Create nodes
    let nodes: Vec<_> = (0..size)
        .map(|i| graph.add_node(format!("N{}", i)))
        .collect();
    
    // Create dense connections - each node connects to several later nodes
    for i in 0..size {
        for j in (i+1)..size.min(i + 5) {
            graph.add_edge(nodes[i], nodes[j], ());
        }
    }
    
    graph
}

fn create_wide_dag(width: usize, depth: usize) -> Graph<String, ()> {
    let mut graph = Graph::new();
    let mut layers = Vec::new();
    
    // Create layers
    for layer_idx in 0..depth {
        let mut layer = Vec::new();
        for node_idx in 0..width {
            let node = graph.add_node(format!("L{}N{}", layer_idx, node_idx));
            layer.push(node);
        }
        layers.push(layer);
    }
    
    // Connect layers with crossing patterns
    for layer_idx in 0..depth-1 {
        for (i, &source) in layers[layer_idx].iter().enumerate() {
            // Connect to multiple targets to create crossings
            let target1 = (i + width / 3) % width;
            let target2 = (i + 2 * width / 3) % width;
            
            graph.add_edge(source, layers[layer_idx + 1][target1], ());
            if target2 != target1 {
                graph.add_edge(source, layers[layer_idx + 1][target2], ());
            }
        }
    }
    
    graph
}

fn bench_small_graphs(c: &mut Criterion) {
    let graph_50 = create_large_dag(50, 2);
    let graph_dense_50 = create_dense_dag(50);
    let layout = DagreLayout::new();

    c.bench_function("layout_50_nodes", |b| {
        b.iter(|| layout.compute(black_box(&graph_50)))
    });

    c.bench_function("layout_dense_50_nodes", |b| {
        b.iter(|| layout.compute(black_box(&graph_dense_50)))
    });
}

fn bench_medium_graphs(c: &mut Criterion) {
    let graph_200 = create_large_dag(200, 3);
    let wide_graph = create_wide_dag(10, 10);
    let layout = DagreLayout::new();

    c.bench_function("layout_200_nodes", |b| {
        b.iter(|| layout.compute(black_box(&graph_200)))
    });

    c.bench_function("layout_wide_10x10", |b| {
        b.iter(|| layout.compute(black_box(&wide_graph)))
    });
}

fn bench_large_graphs(c: &mut Criterion) {
    let graph_500 = create_large_dag(500, 3);
    let graph_1000 = create_large_dag(1000, 2);
    let very_wide = create_wide_dag(20, 8);
    let layout = DagreLayout::new();

    c.bench_function("layout_500_nodes", |b| {
        b.iter(|| layout.compute(black_box(&graph_500)))
    });

    c.bench_function("layout_1000_nodes", |b| {
        b.iter(|| layout.compute(black_box(&graph_1000)))
    });

    c.bench_function("layout_very_wide_20x8", |b| {
        b.iter(|| layout.compute(black_box(&very_wide)))
    });
}

fn bench_different_configurations(c: &mut Criterion) {
    let graph = create_large_dag(300, 2);

    let layout_ttb = DagreLayout::new();
    let layout_ltr = DagreLayout::with_options(LayoutOptions {
        rank_dir: RankDir::LeftToRight,
        ..Default::default()
    });
    let layout_many_iter = DagreLayout::with_options(LayoutOptions {
        max_iterations: 50,
        ..Default::default()
    });

    c.bench_function("layout_top_to_bottom", |b| {
        b.iter(|| layout_ttb.compute(black_box(&graph)))
    });

    c.bench_function("layout_left_to_right", |b| {
        b.iter(|| layout_ltr.compute(black_box(&graph)))
    });

    c.bench_function("layout_many_iterations", |b| {
        b.iter(|| layout_many_iter.compute(black_box(&graph)))
    });
}

fn bench_crossing_reduction(c: &mut Criterion) {
    let crossing_heavy = create_wide_dag(15, 6);
    
    let layout_few_iter = DagreLayout::with_options(LayoutOptions {
        max_iterations: 5,
        ..Default::default()
    });
    let layout_many_iter = DagreLayout::with_options(LayoutOptions {
        max_iterations: 50,
        ..Default::default()
    });

    c.bench_function("crossing_reduction_5_iter", |b| {
        b.iter(|| layout_few_iter.compute(black_box(&crossing_heavy)))
    });

    c.bench_function("crossing_reduction_50_iter", |b| {
        b.iter(|| layout_many_iter.compute(black_box(&crossing_heavy)))
    });
}

criterion_group!(
    benches,
    bench_small_graphs,
    bench_medium_graphs,
    bench_large_graphs,
    bench_different_configurations,
    bench_crossing_reduction
);
criterion_main!(benches);