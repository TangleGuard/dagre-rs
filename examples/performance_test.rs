use dagrers::{DagreLayout, LayoutOptions, RankDir};
use petgraph::Graph;
use std::time::Instant;

fn create_simple_dag(nodes: usize) -> Graph<String, ()> {
    let mut graph = Graph::new();
    
    let node_indices: Vec<_> = (0..nodes)
        .map(|i| graph.add_node(format!("Node{}", i)))
        .collect();
    
    // Create a hierarchical structure
    let layers = (nodes as f64).sqrt() as usize;
    let nodes_per_layer = nodes / layers;
    
    for layer in 0..layers-1 {
        let layer_start = layer * nodes_per_layer;
        let layer_end = ((layer + 1) * nodes_per_layer).min(nodes);
        let next_layer_start = layer_end;
        let next_layer_end = ((layer + 2) * nodes_per_layer).min(nodes);
        
        for i in layer_start..layer_end {
            // Connect to 1-3 nodes in the next layer
            for j in 0..2 {
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

fn create_wide_graph(width: usize, depth: usize) -> Graph<String, ()> {
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
    
    // Connect layers
    for layer_idx in 0..depth-1 {
        for (i, &source) in layers[layer_idx].iter().enumerate() {
            let target = (i + 1) % width;
            graph.add_edge(source, layers[layer_idx + 1][target], ());
        }
    }
    
    graph
}

fn time_layout(name: &str, graph: Graph<String, ()>) {
    println!("Testing: {}", name);
    println!("  Graph: {} nodes, {} edges", graph.node_count(), graph.edge_count());
    
    let layout = DagreLayout::new();
    
    // Warm up
    let _ = layout.compute(&graph);
    
    // Time multiple runs
    let runs = 3;
    let mut times = Vec::new();
    
    for _ in 0..runs {
        let start = Instant::now();
        let result = layout.compute(&graph);
        let duration = start.elapsed();
        times.push(duration);
        
        // Quick validation
        assert_eq!(result.node_positions.len(), graph.node_count());
    }
    
    let avg = times.iter().sum::<std::time::Duration>() / runs as u32;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();
    
    println!("  Average: {:.2}ms", avg.as_secs_f64() * 1000.0);
    println!("  Range: {:.2}ms - {:.2}ms", min.as_secs_f64() * 1000.0, max.as_secs_f64() * 1000.0);
    println!();
}

fn main() {
    println!("Dagrers Performance Test");
    println!("========================");
    println!();
    
    // Test different graph sizes
    time_layout("Small Graph (50 nodes)", create_simple_dag(50));
    time_layout("Medium Graph (200 nodes)", create_simple_dag(200));
    time_layout("Large Graph (500 nodes)", create_simple_dag(500));
    time_layout("Very Large Graph (1000 nodes)", create_simple_dag(1000));
    
    // Test wide graphs
    time_layout("Wide Graph (20×5)", create_wide_graph(20, 5));
    time_layout("Very Wide Graph (50×4)", create_wide_graph(50, 4));
    
    // Test with different configurations
    println!("Testing Left-to-Right layout:");
    let graph = create_simple_dag(300);
    let layout_ltr = DagreLayout::with_options(LayoutOptions {
        rank_dir: RankDir::LeftToRight,
        ..Default::default()
    });
    
    let start = Instant::now();
    let result = layout_ltr.compute(&graph);
    let duration = start.elapsed();
    
    println!("  {} nodes in {:.2}ms", result.node_positions.len(), duration.as_secs_f64() * 1000.0);
    println!();
    
    println!("Performance test completed!");
    println!();
    println!("For detailed benchmarks with statistical analysis, run:");
    println!("  cargo bench");
}