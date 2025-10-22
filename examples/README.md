# Dagrers Examples

This directory contains examples demonstrating the Dagrers graph layout library.

## Running Examples

To run the visualization example:

```bash
cargo run --example visualize_layout
```

This will generate several SVG files in `examples/output/` showing different graph layouts that you can open in any web browser or SVG viewer.

## Available Examples

### `visualize_layout.rs`

Generates SVG visualizations of various graph patterns to test and demonstrate the Sugiyama layout algorithm:

- **Simple Chain**: Linear sequence of connected nodes
- **Diamond Pattern**: Diverge and converge pattern
- **Complex DAG**: Multi-level graph with crossing opportunities
- **Wide Graph**: Graph with many parallel branches
- **Different Directions**: Same graphs rendered top-to-bottom vs left-to-right
- **Spacing Variations**: Different node and rank separations

The example outputs:
- Console statistics (nodes, edges, layers, crossings)
- SVG files for visual inspection
- Layout property verification

## What to Look For

When examining the generated SVG files, verify:

1. **Hierarchical Structure**: Nodes flow in the correct direction
2. **Layer Assignment**: Related nodes are properly grouped in layers
3. **Crossing Minimization**: Edge crossings are minimized where possible
4. **Spacing**: Nodes have appropriate separation
5. **Edge Routing**: Edges connect nodes correctly with arrows

## Generated Files

The visualization example creates these SVG files in `examples/output/`:
- `simple_chain_(top-to-bottom).svg`
- `diamond_pattern_(top-to-bottom).svg` 
- `complex_dag_(top-to-bottom).svg`
- `wide_graph_(top-to-bottom).svg`
- `complex_dag_(left-to-right).svg`
- `tight_spacing.svg`
- `wide_spacing.svg`

Each SVG includes:
- Node circles with labels
- Directed edges with arrows
- Layer separator lines
- Layout statistics
- Title and dimensions

## Testing Your Layout

Use these examples to verify your Sugiyama implementation:

1. Run the examples: `cargo run --example visualize_layout`
2. Open the SVG files from `examples/output/` in a browser
3. Check that layouts look hierarchical and well-organized
4. Verify edge crossings are minimized
5. Confirm spacing and direction options work correctly

The console output also shows quantitative metrics like edge crossings count to help evaluate layout quality.