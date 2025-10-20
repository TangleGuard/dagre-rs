use petgraph::prelude::*;
use std::collections::{HashMap, HashSet};

/// Configuration options for graph layout calculation
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    /// Primary layout direction
    pub rank_dir: RankDir,
    /// Minimum horizontal separation between nodes in the same layer (pixels)
    pub node_sep: f32,
    /// Minimum vertical separation between layers (pixels)
    pub rank_sep: f32,
    /// Maximum number of iterations for crossing reduction
    pub max_iterations: usize,
}

/// Layout direction for the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankDir {
    /// Nodes flow from top to bottom
    TopToBottom,
    /// Nodes flow from left to right
    LeftToRight,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            rank_dir: RankDir::TopToBottom,
            node_sep: 50.0,
            rank_sep: 100.0,
            max_iterations: 24,
        }
    }
}

/// Result of layout calculation containing node positions and layer information
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// Final positions for each node as (x, y) coordinates
    pub node_positions: HashMap<NodeIndex, (f32, f32)>,
    /// Nodes organized by layers, from first to last
    pub layers: Vec<Vec<NodeIndex>>,
    /// Total width of the layout
    pub width: f32,
    /// Total height of the layout
    pub height: f32,
}

/// Main layout engine implementing the Sugiyama method
pub struct DagreLayout {
    /// Layout configuration options
    pub options: LayoutOptions,
}

impl DagreLayout {
    /// Create a new layout engine with default options
    pub fn new() -> Self {
        Self {
            options: LayoutOptions::default(),
        }
    }

    /// Create a new layout engine with custom options
    pub fn with_options(options: LayoutOptions) -> Self {
        Self { options }
    }

    /// Compute the layout for a directed graph using the Sugiyama method
    ///
    /// This method implements the four phases of the Sugiyama algorithm:
    /// 1. Cycle removal (assumes DAG input for now)
    /// 2. Layer assignment using longest path
    /// 3. Crossing reduction using barycenter heuristic
    /// 4. Coordinate assignment with proper spacing
    ///
    /// # Arguments
    /// * `graph` - The directed graph to layout
    ///
    /// # Returns
    /// A `LayoutResult` containing node positions and metadata
    ///
    /// # Example
    /// ```
    /// use dagrers::{DagreLayout, LayoutOptions, RankDir};
    /// use petgraph::Graph;
    ///
    /// let mut graph = Graph::new();
    /// let a = graph.add_node("A");
    /// let b = graph.add_node("B");
    /// graph.add_edge(a, b, ());
    ///
    /// let layout = DagreLayout::new();
    /// let result = layout.compute(&graph);
    /// ```
    pub fn compute<N, E>(&self, graph: &DiGraph<N, E>) -> LayoutResult {
        // Phase 1: Cycle removal (assume DAG for now)
        // TODO: Implement cycle detection and removal

        // Phase 2: Layer assignment
        let mut layers = self.assign_layers_longest_path(graph);

        // Phase 3: Crossing reduction
        self.reduce_crossings(graph, &mut layers);

        // Phase 4: Coordinate assignment
        let (node_positions, width, height) = self.assign_coordinates(&layers);

        LayoutResult {
            node_positions,
            layers,
            width,
            height,
        }
    }

    /// Assign nodes to layers using longest path algorithm
    /// This creates more balanced layouts than simple topological sorting
    fn assign_layers_longest_path<N, E>(&self, graph: &DiGraph<N, E>) -> Vec<Vec<NodeIndex>> {
        let mut distances = HashMap::new();
        let mut visited = HashSet::new();

        // Find all source nodes (no incoming edges)
        let sources: Vec<_> = graph
            .node_indices()
            .filter(|&n| graph.neighbors_directed(n, Incoming).count() == 0)
            .collect();

        // If no sources found, pick an arbitrary starting node
        let sources = if sources.is_empty() {
            graph.node_indices().take(1).collect()
        } else {
            sources
        };

        // Calculate longest paths from sources using DFS
        for &source in &sources {
            self.dfs_longest_path(graph, source, 0, &mut distances, &mut visited);
        }

        // Handle any remaining unvisited nodes (disconnected components)
        for node in graph.node_indices() {
            if !distances.contains_key(&node) {
                self.dfs_longest_path(graph, node, 0, &mut distances, &mut visited);
            }
        }

        // Group nodes by their layer (distance)
        let max_layer = distances.values().copied().max().unwrap_or(0);
        let mut layers = vec![Vec::new(); max_layer + 1];

        for (&node, &layer) in &distances {
            layers[layer].push(node);
        }

        // Remove empty layers
        layers
            .into_iter()
            .filter(|layer| !layer.is_empty())
            .collect()
    }

    /// Depth-first search to calculate longest path distances
    fn dfs_longest_path<N, E>(
        &self,
        graph: &DiGraph<N, E>,
        node: NodeIndex,
        current_distance: usize,
        distances: &mut HashMap<NodeIndex, usize>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        if visited.contains(&node) {
            return;
        }

        visited.insert(node);

        // Update distance if we found a longer path
        let best_distance = distances.get(&node).copied().unwrap_or(0);
        let new_distance = current_distance.max(best_distance);
        distances.insert(node, new_distance);

        // Recursively visit successors
        for successor in graph.neighbors_directed(node, Outgoing) {
            self.dfs_longest_path(graph, successor, new_distance + 1, distances, visited);
        }
    }

    /// Reduce edge crossings using the barycenter heuristic
    /// This iteratively reorders nodes within layers to minimize crossings
    fn reduce_crossings<N, E>(&self, graph: &DiGraph<N, E>, layers: &mut Vec<Vec<NodeIndex>>) {
        if layers.len() < 2 {
            return;
        }

        for _ in 0..self.options.max_iterations {
            let mut improved = false;

            // Forward pass: order layers 1..n based on their predecessors
            for i in 1..layers.len() {
                let new_order = self.order_by_barycenter(graph, &layers[i], &layers[i - 1], true);
                if new_order != layers[i] {
                    layers[i] = new_order;
                    improved = true;
                }
            }

            // Backward pass: order layers n-1..0 based on their successors
            for i in (0..layers.len() - 1).rev() {
                let new_order = self.order_by_barycenter(graph, &layers[i], &layers[i + 1], false);
                if new_order != layers[i] {
                    layers[i] = new_order;
                    improved = true;
                }
            }

            // If no improvement, we can stop early
            if !improved {
                break;
            }
        }
    }

    /// Order nodes in a layer based on barycenter of connected nodes in adjacent layer
    fn order_by_barycenter<N, E>(
        &self,
        graph: &DiGraph<N, E>,
        layer: &[NodeIndex],
        adjacent_layer: &[NodeIndex],
        use_predecessors: bool,
    ) -> Vec<NodeIndex> {
        // Create position map for adjacent layer
        let positions: HashMap<NodeIndex, usize> = adjacent_layer
            .iter()
            .enumerate()
            .map(|(pos, &node)| (node, pos))
            .collect();

        // Calculate barycenter for each node in current layer
        let mut node_barycenters: Vec<(NodeIndex, f32)> = layer
            .iter()
            .map(|&node| {
                let connected_positions: Vec<usize> = if use_predecessors {
                    graph
                        .neighbors_directed(node, Incoming)
                        .filter_map(|pred| positions.get(&pred))
                        .copied()
                        .collect()
                } else {
                    graph
                        .neighbors_directed(node, Outgoing)
                        .filter_map(|succ| positions.get(&succ))
                        .copied()
                        .collect()
                };

                let barycenter = if connected_positions.is_empty() {
                    // No connections, maintain relative position
                    layer.iter().position(|&n| n == node).unwrap() as f32
                } else {
                    connected_positions.iter().sum::<usize>() as f32
                        / connected_positions.len() as f32
                };

                (node, barycenter)
            })
            .collect();

        // Sort by barycenter, maintaining stable order for ties
        node_barycenters.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        node_barycenters.into_iter().map(|(node, _)| node).collect()
    }

    /// Assign final coordinates to nodes with proper spacing
    fn assign_coordinates(
        &self,
        layers: &[Vec<NodeIndex>],
    ) -> (HashMap<NodeIndex, (f32, f32)>, f32, f32) {
        let mut positions = HashMap::new();
        let LayoutOptions {
            rank_dir,
            node_sep,
            rank_sep,
            ..
        } = &self.options;

        let max_layer_width = layers.iter().map(|layer| layer.len()).max().unwrap_or(0) as f32;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_width = layer.len() as f32;

            // Center the layer
            let start_offset = (max_layer_width - layer_width) * node_sep * 0.5;

            for (node_idx, &node) in layer.iter().enumerate() {
                let (x, y) = match rank_dir {
                    RankDir::TopToBottom => (
                        start_offset + node_idx as f32 * node_sep,
                        layer_idx as f32 * rank_sep,
                    ),
                    RankDir::LeftToRight => (
                        layer_idx as f32 * rank_sep,
                        start_offset + node_idx as f32 * node_sep,
                    ),
                };
                positions.insert(node, (x, y));
            }
        }

        // Calculate total dimensions
        let (width, height) = match rank_dir {
            RankDir::TopToBottom => (max_layer_width * node_sep, layers.len() as f32 * rank_sep),
            RankDir::LeftToRight => (layers.len() as f32 * rank_sep, max_layer_width * node_sep),
        };

        (positions, width, height)
    }
}

impl Default for DagreLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::Graph;

    #[test]
    fn test_simple_chain() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());

        let layout = DagreLayout::new();
        let result = layout.compute(&graph);

        assert_eq!(result.layers.len(), 3);
        assert_eq!(result.layers[0], vec![a]);
        assert_eq!(result.layers[1], vec![b]);
        assert_eq!(result.layers[2], vec![c]);
    }

    #[test]
    fn test_diamond_pattern() {
        let mut graph = Graph::new();
        let start = graph.add_node("start");
        let left = graph.add_node("left");
        let right = graph.add_node("right");
        let end = graph.add_node("end");

        graph.add_edge(start, left, ());
        graph.add_edge(start, right, ());
        graph.add_edge(left, end, ());
        graph.add_edge(right, end, ());

        let layout = DagreLayout::new();
        let result = layout.compute(&graph);

        assert_eq!(result.layers.len(), 3);
        assert_eq!(result.layers[0], vec![start]);
        assert_eq!(result.layers[1].len(), 2);
        assert!(result.layers[1].contains(&left));
        assert!(result.layers[1].contains(&right));
        assert_eq!(result.layers[2], vec![end]);
    }

    #[test]
    fn test_left_to_right_layout() {
        let mut graph = Graph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        graph.add_edge(a, b, ());

        let options = LayoutOptions {
            rank_dir: RankDir::LeftToRight,
            ..Default::default()
        };
        let layout = DagreLayout::with_options(options);
        let result = layout.compute(&graph);

        let pos_a = result.node_positions[&a];
        let pos_b = result.node_positions[&b];

        // In left-to-right layout, B should be to the right of A
        assert!(pos_b.0 > pos_a.0);
    }

    #[test]
    fn test_empty_graph() {
        let graph: Graph<&str, (), petgraph::Directed> = Graph::new();
        let layout = DagreLayout::new();
        let result = layout.compute(&graph);

        assert!(result.layers.is_empty());
        assert!(result.node_positions.is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut graph: Graph<&str, (), petgraph::Directed> = Graph::new();
        let node = graph.add_node("single");

        let layout = DagreLayout::new();
        let result = layout.compute(&graph);

        assert_eq!(result.layers.len(), 1);
        assert_eq!(result.layers[0], vec![node]);
        assert_eq!(result.node_positions[&node], (0.0, 0.0));
    }
}
