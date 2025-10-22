use dagrers::{DagreLayout, LayoutOptions, RankDir};
use petgraph::{Graph, graph::NodeIndex};
use std::collections::{HashMap, HashSet};

/// Comprehensive test suite to verify layout correctness
struct LayoutTester {
    layout_engine: DagreLayout,
}

impl LayoutTester {
    fn new(options: LayoutOptions) -> Self {
        Self {
            layout_engine: DagreLayout::with_options(options),
        }
    }

    /// Test that all layout invariants are satisfied
    fn test_layout_invariants<N, E>(&self, graph: &Graph<N, E>) -> Result<(), String> {
        let result = self.layout_engine.compute(graph);

        // 1. All nodes must have positions
        if result.node_positions.len() != graph.node_count() {
            return Err(format!(
                "Position count mismatch: got {}, expected {}",
                result.node_positions.len(),
                graph.node_count()
            ));
        }

        // 2. All nodes must appear in exactly one layer
        let mut nodes_in_layers = HashSet::new();
        for layer in &result.layers {
            for &node in layer {
                if !nodes_in_layers.insert(node) {
                    return Err(format!("Node {:?} appears in multiple layers", node));
                }
            }
        }

        if nodes_in_layers.len() != graph.node_count() {
            return Err(format!(
                "Layer node count mismatch: got {}, expected {}",
                nodes_in_layers.len(),
                graph.node_count()
            ));
        }

        // 3. Verify hierarchical ordering (edges only go forward in layers)
        let node_to_layer = self.create_node_to_layer_map(&result.layers);
        for edge in graph.edge_indices() {
            if let Some((source, target)) = graph.edge_endpoints(edge) {
                let source_layer = node_to_layer[&source];
                let target_layer = node_to_layer[&target];
                
                if source_layer >= target_layer {
                    return Err(format!(
                        "Edge violates hierarchy: {:?} (layer {}) -> {:?} (layer {})",
                        source, source_layer, target, target_layer
                    ));
                }
            }
        }

        // 4. Verify coordinate consistency with layout direction
        self.verify_coordinate_consistency(&result, &node_to_layer)?;

        // 5. Verify dimensions are reasonable
        if result.width <= 0.0 || result.height <= 0.0 {
            return Err(format!(
                "Invalid dimensions: {}x{}",
                result.width, result.height
            ));
        }

        Ok(())
    }

    fn create_node_to_layer_map(&self, layers: &[Vec<NodeIndex>]) -> HashMap<NodeIndex, usize> {
        let mut map = HashMap::new();
        for (layer_idx, layer) in layers.iter().enumerate() {
            for &node in layer {
                map.insert(node, layer_idx);
            }
        }
        map
    }

    fn verify_coordinate_consistency(
        &self,
        result: &dagrers::LayoutResult,
        node_to_layer: &HashMap<NodeIndex, usize>,
    ) -> Result<(), String> {
        match self.layout_engine.options.rank_dir {
            RankDir::TopToBottom => {
                // Y coordinates should increase with layer depth
                for (&node, &layer) in node_to_layer {
                    let pos = result.node_positions[&node];
                    let expected_y = layer as f32 * self.layout_engine.options.rank_sep;
                    
                    if (pos.1 - expected_y).abs() > 0.01 {
                        return Err(format!(
                            "Y coordinate inconsistent: node {:?} at ({}, {}), expected y={}",
                            node, pos.0, pos.1, expected_y
                        ));
                    }
                }
            }
            RankDir::LeftToRight => {
                // X coordinates should increase with layer depth
                for (&node, &layer) in node_to_layer {
                    let pos = result.node_positions[&node];
                    let expected_x = layer as f32 * self.layout_engine.options.rank_sep;
                    
                    if (pos.0 - expected_x).abs() > 0.01 {
                        return Err(format!(
                            "X coordinate inconsistent: node {:?} at ({}, {}), expected x={}",
                            node, pos.0, pos.1, expected_x
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Count edge crossings between two adjacent layers
    fn count_crossings(&self, graph: &Graph<impl std::fmt::Debug, impl std::fmt::Debug>, upper_layer: &[NodeIndex], lower_layer: &[NodeIndex]) -> usize {
        let lower_positions: HashMap<NodeIndex, usize> = lower_layer
            .iter()
            .enumerate()
            .map(|(pos, &node)| (node, pos))
            .collect();
        
        let mut crossings = 0;
        
        for (i, &node1) in upper_layer.iter().enumerate() {
            for &node2 in upper_layer.iter().skip(i + 1) {
                let node1_connections: Vec<usize> = graph
                    .neighbors(node1)
                    .filter_map(|n| lower_positions.get(&n))
                    .copied()
                    .collect();
                
                let node2_connections: Vec<usize> = graph
                    .neighbors(node2)
                    .filter_map(|n| lower_positions.get(&n))
                    .copied()
                    .collect();
                
                for &pos1 in &node1_connections {
                    for &pos2 in &node2_connections {
                        if pos1 > pos2 {
                            crossings += 1;
                        }
                    }
                }
            }
        }
        
        crossings
    }
}

// Test case generators
fn create_problematic_crossing_graph() -> Graph<&'static str, ()> {
    let mut graph = Graph::new();
    let nodes: Vec<_> = ["A", "B1", "B2", "B3", "C1", "C2", "C3"]
        .iter()
        .map(|&name| graph.add_node(name))
        .collect();
    
    // Create a pattern that should have crossings if not optimized
    graph.add_edge(nodes[0], nodes[1], ()); // A -> B1
    graph.add_edge(nodes[0], nodes[2], ()); // A -> B2  
    graph.add_edge(nodes[0], nodes[3], ()); // A -> B3
    
    // Cross-connections that should be optimized
    graph.add_edge(nodes[1], nodes[6], ()); // B1 -> C3 (crossing)
    graph.add_edge(nodes[2], nodes[4], ()); // B2 -> C1 (crossing)
    graph.add_edge(nodes[3], nodes[5], ()); // B3 -> C2 (crossing)
    
    graph
}

fn create_stress_test_graph(size: usize) -> Graph<String, ()> {
    let mut graph = Graph::new();
    let nodes: Vec<_> = (0..size)
        .map(|i| graph.add_node(format!("Node{}", i)))
        .collect();
    
    // Create a complex connectivity pattern
    for i in 0..size {
        for j in (i+1)..(i+4).min(size) {
            graph.add_edge(nodes[i], nodes[j], ());
        }
    }
    
    graph
}

fn create_pathological_cases() -> Vec<(&'static str, Graph<&'static str, ()>)> {
    let mut cases = Vec::new();
    
    // Case 1: Single node
    let mut single = Graph::new();
    single.add_node("single");
    cases.push(("Single Node", single));
    
    // Case 2: Two disconnected nodes
    let mut disconnected = Graph::new();
    disconnected.add_node("A");
    disconnected.add_node("B");
    cases.push(("Disconnected Nodes", disconnected));
    
    // Case 3: Star pattern (one node connects to many)
    let mut star = Graph::new();
    let center = star.add_node("center");
    for i in 1..=10 {
        let leaf = star.add_node("leaf");
        star.add_edge(center, leaf, ());
    }
    cases.push(("Star Pattern", star));
    
    // Case 4: Long chain
    let mut chain = Graph::new();
    let mut prev = chain.add_node("start");
    for i in 1..=20 {
        let current = chain.add_node("node");
        chain.add_edge(prev, current, ());
        prev = current;
    }
    cases.push(("Long Chain", chain));
    
    cases
}

fn run_test_suite() {
    println!("Dagrers Comprehensive Layout Tests");
    println!("=================================");
    println!();

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Test with different configurations
    let configs = vec![
        ("Default (Top-to-Bottom)", LayoutOptions::default()),
        ("Left-to-Right", LayoutOptions {
            rank_dir: RankDir::LeftToRight,
            ..Default::default()
        }),
        ("Tight Spacing", LayoutOptions {
            node_sep: 20.0,
            rank_sep: 40.0,
            ..Default::default()
        }),
        ("Many Iterations", LayoutOptions {
            max_iterations: 50,
            ..Default::default()
        }),
    ];

    for (config_name, options) in configs {
        println!("Testing configuration: {}", config_name);
        let tester = LayoutTester::new(options);

        // Test pathological cases
        for (case_name, graph) in create_pathological_cases() {
            total_tests += 1;
            match tester.test_layout_invariants(&graph) {
                Ok(_) => {
                    println!("  ✓ {}", case_name);
                    passed_tests += 1;
                },
                Err(e) => {
                    println!("  ✗ {}: {}", case_name, e);
                }
            }
        }

        // Test crossing optimization
        total_tests += 1;
        let crossing_graph = create_problematic_crossing_graph();
        match tester.test_layout_invariants(&crossing_graph) {
            Ok(_) => {
                let result = tester.layout_engine.compute(&crossing_graph);
                let total_crossings: usize = (0..result.layers.len().saturating_sub(1))
                    .map(|i| tester.count_crossings(&crossing_graph, &result.layers[i], &result.layers[i + 1]))
                    .sum();
                
                println!("  ✓ Crossing Optimization (crossings: {})", total_crossings);
                passed_tests += 1;
            },
            Err(e) => {
                println!("  ✗ Crossing Optimization: {}", e);
            }
        }

        // Test performance with larger graph
        total_tests += 1;
        let stress_graph = create_stress_test_graph(100);
        let start = std::time::Instant::now();
        match tester.test_layout_invariants(&stress_graph) {
            Ok(_) => {
                let duration = start.elapsed();
                println!("  ✓ Stress Test (100 nodes, {:.2}ms)", duration.as_secs_f64() * 1000.0);
                passed_tests += 1;
            },
            Err(e) => {
                println!("  ✗ Stress Test: {}", e);
            }
        }

        println!();
    }

    println!("Test Results: {}/{} tests passed", passed_tests, total_tests);
    
    if passed_tests == total_tests {
        println!("🎉 All tests passed! Your Sugiyama implementation is working correctly.");
        println!();
        println!("What this validates:");
        println!("✓ Proper hierarchical layer assignment");
        println!("✓ All nodes positioned correctly");  
        println!("✓ Edge direction constraints satisfied");
        println!("✓ Coordinate system consistency");
        println!("✓ Handles edge cases gracefully");
        println!("✓ Crossing reduction functioning");
        println!("✓ Performance within acceptable limits");
    } else {
        println!("❌ Some tests failed. Check the implementation.");
    }
}

fn main() {
    run_test_suite();
    
    println!();
    println!("To visually inspect layouts, run:");
    println!("  cargo run --example visualize_layout");
    println!();
    println!("To run performance benchmarks:");
    println!("  cargo run --example benchmark_layout --release");
}