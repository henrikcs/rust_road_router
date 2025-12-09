# Expand SUMO Nodes Feature

This document describes the `expand-sumo-nodes` feature that enables node expansion for SUMO graph conversion.

## Overview

The `expand-sumo-nodes` feature controls whether SUMO graphs are converted with node expansion. When enabled, the converter creates internal nodes for each edge to model turn restrictions and turn costs through connection edges.

### Without Node Expansion (Default)

- Each SUMO node becomes a single node in the graph
- Only the original edges from SUMO are included
- Simpler graph structure with fewer nodes and edges
- Suitable for basic routing without turn restrictions

### With Node Expansion (Feature Enabled)

- Each SUMO edge creates two internal nodes (one at each endpoint)
- Connection edges are added between internal nodes to represent allowed turns
- More complex graph structure that can model turn restrictions and turn costs
- Required for accurate routing with turn costs

## Usage

### Building

To build **without** node expansion (default):

```bash
cargo build --release
# or specifically for fastdta:
cargo build --release -p fastdta
```

To build **with** node expansion:

```bash
cargo build --release --features expand-sumo-nodes
# or specifically for fastdta:
cargo build --release -p fastdta --features expand-sumo-nodes
```

### Testing

Run tests without node expansion:

```bash
cargo test
# or specifically:
cargo test -p conversion
cargo test -p fastdta
```

Run tests with node expansion:

```bash
cargo test --features expand-sumo-nodes
# or specifically:
cargo test -p conversion --features expand-sumo-nodes
cargo test -p fastdta --features expand-sumo-nodes
```

### Command Line

When using command-line tools built with the feature, all graph conversions will use the mode it was compiled with. To switch modes, rebuild the tool with or without the feature.

Example:

```bash
# Build a router with node expansion
cargo build --release -p fastdta --features expand-sumo-nodes

# Use the router (it will use expanded nodes automatically)
./target/release/sumo-tdcch-router --input-dir ./import/sumo/example --input-prefix example
```

## Implementation Details

### Key Changes

### Cargo.toml Files

- `conversion/Cargo.toml`: Defines the `expand-sumo-nodes` feature
- `fastdta/Cargo.toml`: Propagates the feature to the conversion dependency; Enables adaptation for expanded nodes in `fastdta/src/query.rs`.

### Graph Structure Differences

Without expansion:

- Nodes: Original SUMO nodes
- Edges: Original SUMO edges
- Example: 3 nodes, 2 edges → 3 nodes, 2 edges

With expansion:

- Nodes: 2 internal nodes per edge
- Edges: Original edges + connection edges between internal nodes
- Example: 3 nodes, 2 edges → 4 internal nodes, 2 original edges + N connection edges
