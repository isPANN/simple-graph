# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Criterion benchmarks for construction, query, BFS, and CSR conversion.
- Fuzz testing targets for `read_edge_list` and `read_matrix_market`.
- `#![forbid(unsafe_code)]` safety guarantee.
- Cargo.toml metadata: `repository`, `keywords`, `categories`, `rust-version`.
- DFS iterator (`algo::dfs`).
- Unweighted shortest path (`algo::shortest_path_lengths`).
- `degree_distribution` on the `Graph` trait.
- `# Examples` doc blocks on all public API methods.
- `Send + Sync` compile-time static assertions.

## [0.1.0] - 2026-04-07

### Added
- `SimpleGraph` with sorted adjacency lists.
- `CsrGraph` compressed sparse row representation.
- `Graph` trait with `nv`, `ne`, `has_vertex`, `has_edge`, `degree`, `neighbors`, `density`, `degree_sequence`.
- BFS iterator, `is_connected`, `connected_components`.
- Graph generators: `complete`, `cycle`, `path`, `grid_2d`, `erdos_renyi`.
- Edge-list and Matrix Market I/O.
- Optional serde support.
