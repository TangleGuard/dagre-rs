use dagrers::{DagreLayout, LayoutOptions, LayoutResult, RankDir};
use petgraph::{Graph, graph::NodeIndex};
use std::collections::HashMap;
use std::fs;

/// Simple SVG generator for visualizing graph layouts
struct SvgRenderer {
    width: f32,
    height: f32,
    padding: f32,
}

impl SvgRenderer {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width: width + 100.0, // Add padding
            height: height + 100.0,
            padding: 50.0,
        }
    }

    fn render_layout<N: std::fmt::Display, E>(
        &self,
        graph: &Graph<N, E>,
        layout: &LayoutResult,
        title: &str,
    ) -> String {
        let mut svg = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
<style>
.node {{ fill: #e1f5fe; stroke: #0277bd; stroke-width: 2; }}
.node-text {{ font-family: Arial, sans-serif; font-size: 14px; text-anchor: middle; dominant-baseline: middle; }}
.edge {{ stroke: #424242; stroke-width: 1.5; marker-end: url(#arrowhead); }}
.title {{ font-family: Arial, sans-serif; font-size: 18px; font-weight: bold; text-anchor: middle; }}
.layer-line {{ stroke: #e0e0e0; stroke-width: 1; stroke-dasharray: 5,5; }}
</style>
<defs>
<marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">
<polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#424242\"/>
</marker>
</defs>
",
            self.width, self.height
        );

        // Title
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"25\" class=\"title\">{}</text>",
            self.width / 2.0,
            title
        ));

        // Draw layer separator lines
        for (i, layer) in layout.layers.iter().enumerate() {
            if layer.is_empty() {
                continue;
            }
            
            let first_pos = layout.node_positions[&layer[0]];
            let y = first_pos.1 + self.padding;
            
            svg.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"layer-line\" />",
                self.padding,
                y,
                self.width - self.padding,
                y
            ));
            
            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" class=\"node-text\" style=\"fill: #757575; font-size: 12px;\">Layer {}</text>",
                15.0,
                y - 5.0,
                i
            ));
        }

        // Draw edges first (so they appear behind nodes)
        for edge in graph.edge_indices() {
            if let Some((source, target)) = graph.edge_endpoints(edge) {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                    layout.node_positions.get(&source),
                    layout.node_positions.get(&target),
                ) {
                    svg.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"edge\" />",
                        x1 + self.padding,
                        y1 + self.padding,
                        x2 + self.padding,
                        y2 + self.padding
                    ));
                }
            }
        }

        // Draw nodes
        for (node_idx, &(x, y)) in &layout.node_positions {
            let node_data = &graph[*node_idx];
            
            // Node circle
            svg.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"20\" class=\"node\" />",
                x + self.padding,
                y + self.padding
            ));
            
            // Node label
            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" class=\"node-text\">{}</text>",
                x + self.padding,
                y + self.padding,
                node_data
            ));
        }

        // Layout info
        svg.push_str(&format!(
            "<text x=\"10\" y=\"{}\" class=\"node-text\" style=\"fill: #757575; font-size: 12px;\">Nodes: {} | Layers: {} | Size: {:.0}×{:.0}</text>",
            self.height - 10.0,
            graph.node_count(),
            layout.layers.len(),
            layout.width,
            layout.height
        ));

        svg.push_str("</svg>");
        svg
    }
}

fn create_simple_chain() -> Graph<&'static str, ()> {
    let mut graph = Graph::new();
    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, d, ());

    graph
}

fn create_diamond_pattern() -> Graph<&'static str, ()> {
    let mut graph = Graph::new();
    let start = graph.add_node("Start");
    let left = graph.add_node("Left");
    let right = graph.add_node("Right");
    let end = graph.add_node("End");

    graph.add_edge(start, left, ());
    graph.add_edge(start, right, ());
    graph.add_edge(left, end, ());
    graph.add_edge(right, end, ());

    graph
}

fn create_complex_dag() -> Graph<&'static str, ()> {
    let mut graph = Graph::new();
    let root = graph.add_node("Root");
    let a1 = graph.add_node("A1");
    let a2 = graph.add_node("A2");
    let b1 = graph.add_node("B1");
    let b2 = graph.add_node("B2");
    let b3 = graph.add_node("B3");
    let c1 = graph.add_node("C1");
    let c2 = graph.add_node("C2");
    let end = graph.add_node("End");

    // First level
    graph.add_edge(root, a1, ());
    graph.add_edge(root, a2, ());

    // Second level - creates crossing potential
    graph.add_edge(a1, b1, ());
    graph.add_edge(a1, b2, ());
    graph.add_edge(a2, b2, ());
    graph.add_edge(a2, b3, ());

    // Third level
    graph.add_edge(b1, c1, ());
    graph.add_edge(b2, c1, ());
    graph.add_edge(b2, c2, ());
    graph.add_edge(b3, c2, ());

    // Final convergence
    graph.add_edge(c1, end, ());
    graph.add_edge(c2, end, ());

    graph
}

fn create_wide_graph() -> Graph<String, ()> {
    let mut graph = Graph::new();
    let root = graph.add_node("Root".to_string());
    
    // Create many branches
    let mut level1_nodes = Vec::new();
    for i in 1..=8 {
        let node = graph.add_node(format!("L1-{}", i));
        graph.add_edge(root, node, ());
        level1_nodes.push(node);
    }
    
    // Second level with crossing patterns
    let mut level2_nodes = Vec::new();
    for i in 1..=5 {
        let node = graph.add_node(format!("L2-{}", i));
        level2_nodes.push(node);
    }
    
    // Connect with crossing pattern
    graph.add_edge(level1_nodes[0], level2_nodes[2], ());
    graph.add_edge(level1_nodes[1], level2_nodes[0], ());
    graph.add_edge(level1_nodes[2], level2_nodes[4], ());
    graph.add_edge(level1_nodes[3], level2_nodes[1], ());
    graph.add_edge(level1_nodes[4], level2_nodes[3], ());
    graph.add_edge(level1_nodes[5], level2_nodes[2], ());
    graph.add_edge(level1_nodes[6], level2_nodes[4], ());
    graph.add_edge(level1_nodes[7], level2_nodes[1], ());
    
    // Final convergence
    let end = graph.add_node("End".to_string());
    for &node in &level2_nodes {
        graph.add_edge(node, end, ());
    }

    graph
}

fn test_layout(name: &str, graph: Graph<impl std::fmt::Display, ()>, options: LayoutOptions) {
    println!("Testing layout: {}", name);
    
    let layout_engine = DagreLayout::with_options(options.clone());
    let result = layout_engine.compute(&graph);
    
    // Print layout statistics
    println!("  Nodes: {}", graph.node_count());
    println!("  Edges: {}", graph.edge_count());
    println!("  Layers: {}", result.layers.len());
    println!("  Layout size: {:.1} × {:.1}", result.width, result.height);
    println!("  Direction: {:?}", options.rank_dir);
    
    // Check layout properties
    let mut total_crossings = 0;
    for i in 0..result.layers.len().saturating_sub(1) {
        let crossings = count_crossings(&graph, &result.layers[i], &result.layers[i + 1]);
        total_crossings += crossings;
    }
    println!("  Edge crossings: {}", total_crossings);
    
    // Generate SVG
    let renderer = SvgRenderer::new(result.width, result.height);
    let svg_content = renderer.render_layout(&graph, &result, name);
    
    // Create output directory if it doesn't exist
    let output_dir = "examples/output";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    
    let filename = format!("{}/{}.svg", output_dir, name.replace(" ", "_").to_lowercase());
    fs::write(&filename, svg_content)
        .expect("Failed to write SVG file");
    
    println!("  Saved: {}", filename);
    println!();
}

fn count_crossings<N, E>(
    graph: &Graph<N, E>, 
    upper_layer: &[NodeIndex], 
    lower_layer: &[NodeIndex]
) -> usize {
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
            
            // Count inversions between the two sets of connections
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

fn main() {
    println!("Dagrers Layout Visualization Tests");
    println!("==================================");
    println!();

    // Test different graph patterns with top-to-bottom layout
    test_layout(
        "Simple Chain (Top-to-Bottom)",
        create_simple_chain(),
        LayoutOptions::default(),
    );

    test_layout(
        "Diamond Pattern (Top-to-Bottom)",
        create_diamond_pattern(),
        LayoutOptions::default(),
    );

    test_layout(
        "Complex DAG (Top-to-Bottom)",
        create_complex_dag(),
        LayoutOptions::default(),
    );

    test_layout(
        "Wide Graph (Top-to-Bottom)",
        create_wide_graph(),
        LayoutOptions::default(),
    );

    // Test left-to-right layouts
    test_layout(
        "Complex DAG (Left-to-Right)",
        create_complex_dag(),
        LayoutOptions {
            rank_dir: RankDir::LeftToRight,
            ..Default::default()
        },
    );

    // Test different spacing options
    test_layout(
        "Tight Spacing",
        create_complex_dag(),
        LayoutOptions {
            node_sep: 30.0,
            rank_sep: 60.0,
            ..Default::default()
        },
    );

    test_layout(
        "Wide Spacing",
        create_complex_dag(),
        LayoutOptions {
            node_sep: 80.0,
            rank_sep: 150.0,
            ..Default::default()
        },
    );

    println!("All tests completed! Check the generated SVG files to visually inspect the layouts.");
    println!("Look for:");
    println!("- Proper hierarchical structure");
    println!("- Minimized edge crossings");
    println!("- Appropriate spacing");
    println!("- Correct flow direction");
}