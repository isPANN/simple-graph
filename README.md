# simple-graph

[![CI](https://github.com/isPANN/simple-graph/actions/workflows/ci.yml/badge.svg)](https://github.com/isPANN/simple-graph/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Lightweight undirected graph library for Rust, modeled on Julia's [Graphs.jl](https://github.com/JuliaGraphs/Graphs.jl).

## Features

- **Two representations** -- `SimpleGraph` (mutable, sorted adjacency lists) and `CsrGraph` (immutable, contiguous memory CSR layout)
- **`Graph` trait** -- generic algorithms work on both representations with zero-cost monomorphized dispatch
- **Algorithms** -- BFS, connected components, connectivity testing
- **I/O** -- edge list and Matrix Market (symmetric pattern) formats
- **Generators** -- complete, cycle, path, grid, Erdos-Renyi
- **Optional dependencies** -- `serde` (default), `rand` (for random generators)

## Quick start

```rust
use simple_graph::{SimpleGraph, CsrGraph, Graph, algo, gen};

// Build a graph
let g = SimpleGraph::from_edges(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
assert!(algo::is_connected(&g));
assert_eq!(g.density(), 4.0 / 10.0);

// Convert to CSR for fast read-only access
let csr = CsrGraph::from(&g);
assert_eq!(csr.neighbors(1), &[0, 2]);

// Generate graphs
let grid = gen::grid_2d(10, 10);
assert_eq!(grid.nv(), 100);
```

## Optional features

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | yes | Serialize/deserialize `SimpleGraph` |
| `rand`  | no  | `gen::erdos_renyi` random graph generator |

```toml
[dependencies]
simple-graph = { version = "0.1", features = ["rand"] }
```

## License

MIT
