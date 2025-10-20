# Dagrers Project Context

## Purpose
Dagrers is a Rust library implementing the Sugiyama method for hierarchical graph layout calculation. The project provides efficient algorithms for laying out directed acyclic graphs (DAGs) with a focus on performance, correctness, and ease of use. The library is designed for integration into graph visualization systems, diagramming tools, and applications requiring automatic graph layout.

**Key Goals:**
- Implement a complete, production-ready Sugiyama framework
- Provide clean, ergonomic APIs for Rust developers
- Achieve high performance for large graphs (1000+ nodes)
- Maintain minimal dependencies and memory efficiency
- Support multiple layout directions and customization options

## Tech Stack

### Core Technologies
- **Rust 2024 Edition** - Primary implementation language
- **petgraph 0.8.3** - Graph data structures and algorithms
- **std collections** - HashMap, HashSet, VecDeque for internal algorithms

### Development Tools
- **Cargo** - Package management and build system
- **rustdoc** - API documentation generation
- **Clippy** - Linting and code quality
- **rustfmt** - Code formatting

### Testing Framework
- **Built-in Rust testing** - Unit and integration tests
- **Criterion.rs** - Benchmarking (planned)
- **Property-based testing** - QuickCheck integration (planned)

## Project Conventions

### Code Style
- **Formatting**: Standard rustfmt configuration
- **Naming**: 
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, structs, enums
  - `SCREAMING_SNAKE_CASE` for constants
- **Documentation**: All public APIs must have comprehensive rustdoc comments with examples
- **Error Handling**: Prefer returning Results over panicking
- **Generics**: Use meaningful type parameter names (N for Node data, E for Edge data)

### Architecture Patterns
- **Separation of Concerns**: Clear separation between algorithm phases
- **Immutable by Default**: Prefer immutable data structures where possible
- **Generic Design**: Generic over node and edge data types using petgraph's DiGraph
- **Configuration Objects**: Use structs for complex parameter sets (LayoutOptions)
- **Builder Pattern**: Considered for complex layout configurations

### Algorithm Implementation
- **Sugiyama Method Phases**:
  1. Cycle removal (detection and feedback arc removal)
  2. Layer assignment (longest path algorithm)
  3. Crossing reduction (barycenter and median heuristics)
  4. Coordinate assignment (centered positioning with proper spacing)
- **Iterative Improvement**: Algorithms should stop when no further improvement is possible
- **Performance**: Target O(V + E) for layer assignment, O(V² * I) for crossing reduction

### Testing Strategy
- **Unit Tests**: Each algorithm phase tested independently
- **Integration Tests**: End-to-end layout computation with various graph topologies
- **Property Tests**: Invariant validation (e.g., layer ordering, no overlapping nodes)
- **Performance Tests**: Benchmark against reference implementations
- **Edge Cases**: Empty graphs, single nodes, disconnected components, cycles

### Git Workflow
- **Main Branch**: Always stable, ready for release
- **Feature Branches**: `feature/description` for new functionality
- **Commit Messages**: Conventional commits format
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation changes
  - `perf:` for performance improvements
  - `test:` for test additions/changes
  - `refactor:` for code restructuring
- **PR Requirements**: All tests pass, documentation updated, code reviewed

## Domain Context

### Graph Layout Theory
- **Sugiyama Method**: The gold standard for hierarchical graph drawing
- **Crossing Minimization**: NP-hard problem, heuristics required
- **Layer Assignment**: Affects overall layout quality significantly
- **Coordinate Assignment**: Final positioning with aesthetic considerations

### Graph Types Supported
- **Directed Acyclic Graphs (DAGs)**: Primary focus
- **Directed Graphs with Cycles**: Future support with cycle removal
- **Disconnected Components**: Handled gracefully
- **Size Constraints**: Target 10K+ nodes, 50K+ edges

### Use Cases
- **Workflow Diagrams**: Business process visualization
- **Dependency Graphs**: Software architecture, build systems
- **Organizational Charts**: Hierarchical structures
- **Data Flow Diagrams**: System design documentation
- **Academic Graphs**: Research paper citations, course prerequisites

## Important Constraints

### Performance Requirements
- **Time Complexity**: Reasonable performance for graphs up to 10,000 nodes
- **Memory Usage**: Linear growth with graph size
- **Real-time**: Sub-second response for typical use cases (< 1000 nodes)

### Compatibility
- **MSRV**: Minimum Supported Rust Version 1.70+
- **No-std**: Future consideration for embedded use
- **Platform Support**: All platforms supported by Rust and petgraph

### Algorithm Limitations
- **Cycle Handling**: Currently assumes DAG input (cycle removal planned)
- **Layout Direction**: Top-to-bottom primary, left-to-right implemented
- **Node Shapes**: Currently treats all nodes as points (size-aware positioning planned)

### API Stability
- **Semantic Versioning**: Strict adherence to semver
- **Breaking Changes**: Only in major version increments
- **Deprecation**: 6-month notice for API changes

## External Dependencies

### Direct Dependencies
- **petgraph**: Graph data structures and basic algorithms
  - Well-maintained, performance-focused
  - Provides DiGraph, NodeIndex, EdgeIndex types
  - Handles graph traversal and basic operations

### Development Dependencies (Planned)
- **criterion**: Benchmarking framework
- **quickcheck**: Property-based testing
- **serde**: Serialization support (optional feature)

### Integration Targets
- **Web Frameworks**: Wasm compatibility for browser use
- **Visualization Libraries**: D3.js, Cytoscape.js, vis.js integration helpers
- **Graph Databases**: Neo4j, graph analysis tools
- **Diagramming Tools**: Mermaid, Graphviz compatibility layers

## Project Structure

### Module Organization
```
src/
├── lib.rs          # Public API exports
├── layout.rs       # Main Sugiyama implementation
├── crossing.rs     # Crossing reduction algorithms (planned)
├── layers.rs       # Layer assignment strategies (planned)
├── coordinates.rs  # Coordinate assignment methods (planned)
└── utils.rs        # Helper functions and utilities (planned)
```

### Documentation Structure
- **README.md**: Quick start and basic usage
- **docs/**: Detailed guides and tutorials
- **examples/**: Runnable example code
- **openspec/**: Project specifications and design decisions

## Quality Assurance

### Code Quality Standards
- **Clippy**: All warnings addressed (allow exceptions documented)
- **rustfmt**: Consistent formatting enforced
- **Documentation**: 100% public API coverage
- **Test Coverage**: Target 90%+ line coverage

### Performance Monitoring
- **Benchmarks**: Continuous performance tracking
- **Memory Profiling**: Regular memory usage analysis
- **Regression Detection**: Automated performance regression alerts

### Security Considerations
- **No Unsafe Code**: Core algorithms implemented in safe Rust
- **Input Validation**: Robust handling of malformed graphs
- **Resource Limits**: Protection against excessive memory/CPU usage