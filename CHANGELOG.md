# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-07

### Added
- `SimpleGraph` with sorted adjacency lists.
- `CsrGraph` compressed sparse row representation.
- `CsrBuilder` for incremental CSR construction.
- `Graph` trait with `nv`, `ne`, `has_vertex`, `has_edge`, `degree`, `neighbors`, `density`, `degree_sequence`, `degree_distribution`.
- BFS iterator, DFS iterator, `is_connected`, `connected_components`.
- Unweighted shortest path (`algo::shortest_path_lengths`).
- Graph generators: `complete`, `cycle`, `path`, `grid_2d`, `erdos_renyi`, `complete_csr`, `grid_2d_csr`.
- Edge-list and Matrix Market I/O.
- Optional serde support.
- Criterion benchmarks and petgraph comparison suite.
- Fuzz testing targets for `read_edge_list` and `read_matrix_market`.
- `#![forbid(unsafe_code)]` safety guarantee.
- `Send + Sync` compile-time static assertions.
