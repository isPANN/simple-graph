# Production Readiness (Issue #3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the production readiness checklist from issue #3 — benchmarks, fuzz testing, Cargo metadata, changelog, `forbid(unsafe_code)`, new algorithms, doc examples, and thread-safety assertions.

**Architecture:** Each checklist item is an independent task. The "must have" items come first, followed by "should have" items. New algorithms (DFS, shortest path, degree distribution) follow the existing pattern in `src/algo/` with a module per algorithm exposed through `src/algo/mod.rs`.

**Tech Stack:** Rust, criterion (benchmarks), cargo-fuzz / libfuzzer (fuzz testing)

---

## File Map

| Action | Path | Purpose |
|--------|------|---------|
| Modify | `Cargo.toml` | Add `repository`, `keywords`, `categories`, `rust-version`, criterion dev-dep |
| Modify | `src/lib.rs` | Add `#![forbid(unsafe_code)]` |
| Create | `CHANGELOG.md` | Changelog |
| Create | `benches/benchmarks.rs` | Criterion benchmarks |
| Create | `fuzz/Cargo.toml` | Fuzz crate manifest |
| Create | `fuzz/fuzz_targets/fuzz_edge_list.rs` | Fuzz target for `read_edge_list` |
| Create | `fuzz/fuzz_targets/fuzz_matrix_market.rs` | Fuzz target for `read_matrix_market` |
| Create | `src/algo/dfs.rs` | DFS iterator |
| Create | `src/algo/shortest_path.rs` | Unweighted shortest path (BFS-based) |
| Modify | `src/algo/mod.rs` | Re-export new algorithms |
| Modify | `src/graph.rs` | Add `degree_distribution` method + free function |
| Modify | Multiple `src/*.rs` files | Add `# Examples` doc blocks |
| Modify | `src/lib.rs` | Add `Send + Sync` static assertions |

---

### Task 1: `#![forbid(unsafe_code)]`

**Files:**
- Modify: `src/lib.rs:1`

- [ ] **Step 1: Add the attribute**

At the very top of `src/lib.rs`, before all other lines, add:

```rust
#![forbid(unsafe_code)]
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo test --all-features`
Expected: All tests pass. If any unsafe code existed, it would be caught here.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "chore: add #![forbid(unsafe_code)] safety guarantee"
```

---

### Task 2: Cargo.toml metadata

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add missing metadata fields**

Add these fields to the `[package]` section in `Cargo.toml`, after the existing `description` line:

```toml
repository = "https://github.com/isPANN/simple-graph"
keywords = ["graph", "undirected", "adjacency-list", "csr", "algorithms"]
categories = ["algorithms", "data-structures", "science"]
rust-version = "1.70"
```

MSRV 1.70 is chosen because edition 2021 requires 1.56+, and 1.70 is a conservative, widely-available baseline that supports all features used (partition_point, etc.).

- [ ] **Step 2: Verify it builds**

Run: `cargo check`
Expected: Successful compilation with no warnings.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add repository, keywords, categories, rust-version to Cargo.toml"
```

---

### Task 3: CHANGELOG.md

**Files:**
- Create: `CHANGELOG.md`

- [ ] **Step 1: Create CHANGELOG.md**

```markdown
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
- Unweighted shortest path (`algo::shortest_path`).
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
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: add CHANGELOG.md"
```

---

### Task 4: Criterion benchmarks

**Files:**
- Modify: `Cargo.toml` (add dev-dependency + `[[bench]]`)
- Create: `benches/benchmarks.rs`

- [ ] **Step 1: Add criterion to Cargo.toml**

Add to the `[dev-dependencies]` section:

```toml
criterion = { version = "0.5", features = ["html_reports"] }
```

Add at the end of `Cargo.toml`:

```toml
[[bench]]
name = "benchmarks"
harness = false
```

- [ ] **Step 2: Create the benchmark file**

Create `benches/benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_graph::{algo, gen, CsrGraph, Graph, SimpleGraph};

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    group.bench_function("from_edges_100", |b| {
        let edges: Vec<(u32, u32)> = (0..99).map(|i| (i, i + 1)).collect();
        b.iter(|| SimpleGraph::from_edges(black_box(100), black_box(&edges)))
    });

    group.bench_function("from_edges_10000", |b| {
        let edges: Vec<(u32, u32)> = (0..9999).map(|i| (i, i + 1)).collect();
        b.iter(|| SimpleGraph::from_edges(black_box(10_000), black_box(&edges)))
    });

    group.bench_function("complete_100", |b| {
        b.iter(|| gen::complete(black_box(100)))
    });

    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let g = gen::complete(200);
    let csr = CsrGraph::from(&g);

    let mut group = c.benchmark_group("query");

    group.bench_function("has_edge_simple", |b| {
        b.iter(|| g.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("has_edge_csr", |b| {
        b.iter(|| csr.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("neighbors_simple", |b| {
        b.iter(|| g.neighbors(black_box(50)))
    });

    group.bench_function("neighbors_csr", |b| {
        b.iter(|| csr.neighbors(black_box(50)))
    });

    group.bench_function("degree_sequence", |b| {
        b.iter(|| g.degree_sequence())
    });

    group.finish();
}

fn bench_bfs(c: &mut Criterion) {
    let g = gen::grid_2d(100, 100);
    let csr = CsrGraph::from(&g);

    let mut group = c.benchmark_group("bfs");

    group.bench_function("bfs_grid_100x100_simple", |b| {
        b.iter(|| algo::bfs(black_box(&g), 0).count())
    });

    group.bench_function("bfs_grid_100x100_csr", |b| {
        b.iter(|| algo::bfs(black_box(&csr), 0).count())
    });

    group.bench_function("connected_components_grid", |b| {
        b.iter(|| algo::connected_components(black_box(&g)))
    });

    group.finish();
}

fn bench_csr_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_conversion");

    let g_small = gen::complete(100);
    group.bench_function("to_csr_complete_100", |b| {
        b.iter(|| CsrGraph::from(black_box(&g_small)))
    });

    let g_large = gen::grid_2d(100, 100);
    group.bench_function("to_csr_grid_100x100", |b| {
        b.iter(|| CsrGraph::from(black_box(&g_large)))
    });

    let csr = CsrGraph::from(&g_small);
    group.bench_function("to_simple_complete_100", |b| {
        b.iter(|| black_box(&csr).to_simple_graph())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_query,
    bench_bfs,
    bench_csr_conversion
);
criterion_main!(benches);
```

- [ ] **Step 3: Verify benchmarks compile and run**

Run: `cargo bench -- --sample-size 10`
Expected: All benchmark groups run successfully with timing output.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml benches/benchmarks.rs
git commit -m "perf: add criterion benchmarks for construction, query, BFS, CSR conversion"
```

---

### Task 5: Fuzz testing

**Files:**
- Create: `fuzz/Cargo.toml`
- Create: `fuzz/fuzz_targets/fuzz_edge_list.rs`
- Create: `fuzz/fuzz_targets/fuzz_matrix_market.rs`

- [ ] **Step 1: Create `fuzz/Cargo.toml`**

```toml
[package]
name = "simple-graph-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"

[dependencies.simple-graph]
path = ".."

[[bin]]
name = "fuzz_edge_list"
path = "fuzz_targets/fuzz_edge_list.rs"
doc = false

[[bin]]
name = "fuzz_matrix_market"
path = "fuzz_targets/fuzz_matrix_market.rs"
doc = false

[workspace]
```

- [ ] **Step 2: Create `fuzz/fuzz_targets/fuzz_edge_list.rs`**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = simple_graph::io::read_edge_list(data);
});
```

- [ ] **Step 3: Create `fuzz/fuzz_targets/fuzz_matrix_market.rs`**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = simple_graph::io::read_matrix_market(data);
});
```

- [ ] **Step 4: Verify fuzz targets compile**

Run: `cd /Users/xiweipan/Codes/simple-graph && cargo check --manifest-path fuzz/Cargo.toml`
Expected: Successful compilation. (Full fuzz runs require `cargo-fuzz` and nightly, but compilation should work.)

- [ ] **Step 5: Commit**

```bash
git add fuzz/
git commit -m "test: add cargo-fuzz targets for read_edge_list and read_matrix_market"
```

---

### Task 6: DFS algorithm

**Files:**
- Create: `src/algo/dfs.rs`
- Modify: `src/algo/mod.rs`

- [ ] **Step 1: Write tests in the new DFS module**

Create `src/algo/dfs.rs`:

```rust
use crate::graph::Graph;

/// DFS iterator from a source vertex (iterative, using an explicit stack).
pub struct Dfs<'a, G: Graph + ?Sized> {
    graph: &'a G,
    stack: Vec<u32>,
    visited: Vec<bool>,
}

/// Return a DFS iterator starting from `source`.
pub fn dfs<G: Graph>(graph: &G, source: u32) -> Dfs<'_, G> {
    let n = graph.nv();
    let mut visited = vec![false; n];
    let mut stack = Vec::new();
    if graph.has_vertex(source) {
        visited[source as usize] = true;
        stack.push(source);
    }
    Dfs {
        graph,
        stack,
        visited,
    }
}

impl<'a, G: Graph + ?Sized> Iterator for Dfs<'a, G> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        let u = self.stack.pop()?;
        // Iterate neighbors in reverse so that the smallest neighbor
        // is visited first (popped from top of stack).
        for &v in self.graph.neighbors(u).iter().rev() {
            if !self.visited[v as usize] {
                self.visited[v as usize] = true;
                self.stack.push(v);
            }
        }
        Some(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CsrGraph, SimpleGraph};

    #[test]
    fn test_dfs_visits_all_connected() {
        // Path: 0-1-2-3
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let order: Vec<u32> = dfs(&g, 0).collect();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], 0);
        // DFS goes deep: 0 -> 1 -> 2 -> 3
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_dfs_disconnected() {
        let g = SimpleGraph::new(4);
        let order: Vec<u32> = dfs(&g, 0).collect();
        assert_eq!(order, vec![0]);
    }

    #[test]
    fn test_dfs_on_csr() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        let order: Vec<u32> = dfs(&csr, 0).collect();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], 0);
    }

    #[test]
    fn test_dfs_invalid_source() {
        let g = SimpleGraph::new(3);
        let order: Vec<u32> = dfs(&g, 10).collect();
        assert!(order.is_empty());
    }

    #[test]
    fn test_dfs_triangle() {
        // 0-1, 0-2, 1-2
        let g = SimpleGraph::from_edges(3, &[(0, 1), (0, 2), (1, 2)]);
        let order: Vec<u32> = dfs(&g, 0).collect();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], 0);
        // Smallest neighbor first: 0 pushes [2, 1], pops 1, 1 pushes [2] (already visited skip), pops 2
        assert_eq!(order, vec![0, 1, 2]);
    }
}
```

- [ ] **Step 2: Re-export from mod.rs**

Add to `src/algo/mod.rs`:

```rust
mod dfs;
pub use dfs::{dfs, Dfs};
```

- [ ] **Step 3: Run tests**

Run: `cargo test algo::dfs`
Expected: All 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/algo/dfs.rs src/algo/mod.rs
git commit -m "feat: add DFS iterator"
```

---

### Task 7: Unweighted shortest path

**Files:**
- Create: `src/algo/shortest_path.rs`
- Modify: `src/algo/mod.rs`

- [ ] **Step 1: Create the module with implementation and tests**

Create `src/algo/shortest_path.rs`:

```rust
use crate::graph::Graph;
use std::collections::VecDeque;

/// Compute unweighted shortest-path distances from `source` to all reachable vertices.
///
/// Returns a `Vec<Option<u32>>` of length `nv()`. `result[v]` is `Some(d)` if
/// vertex `v` is reachable from `source` in `d` hops, or `None` if unreachable.
/// `result[source]` is `Some(0)`.
pub fn shortest_path_lengths<G: Graph>(graph: &G, source: u32) -> Vec<Option<u32>> {
    let n = graph.nv();
    let mut dist: Vec<Option<u32>> = vec![None; n];
    if !graph.has_vertex(source) {
        return dist;
    }
    dist[source as usize] = Some(0);
    let mut queue = VecDeque::new();
    queue.push_back(source);
    while let Some(u) = queue.pop_front() {
        let d = dist[u as usize].unwrap();
        for &v in graph.neighbors(u) {
            if dist[v as usize].is_none() {
                dist[v as usize] = Some(d + 1);
                queue.push_back(v);
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CsrGraph, SimpleGraph};

    #[test]
    fn test_path_graph() {
        // 0-1-2-3
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let d = shortest_path_lengths(&g, 0);
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn test_disconnected() {
        // 0-1, 2-3 (two components)
        let g = SimpleGraph::from_edges(4, &[(0, 1), (2, 3)]);
        let d = shortest_path_lengths(&g, 0);
        assert_eq!(d, vec![Some(0), Some(1), None, None]);
    }

    #[test]
    fn test_triangle() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
        let d = shortest_path_lengths(&g, 0);
        assert_eq!(d, vec![Some(0), Some(1), Some(1)]);
    }

    #[test]
    fn test_on_csr() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        let d = shortest_path_lengths(&csr, 0);
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn test_invalid_source() {
        let g = SimpleGraph::new(3);
        let d = shortest_path_lengths(&g, 10);
        assert_eq!(d, vec![None, None, None]);
    }

    #[test]
    fn test_single_vertex() {
        let g = SimpleGraph::new(1);
        let d = shortest_path_lengths(&g, 0);
        assert_eq!(d, vec![Some(0)]);
    }
}
```

- [ ] **Step 2: Re-export from mod.rs**

Add to `src/algo/mod.rs`:

```rust
mod shortest_path;
pub use shortest_path::shortest_path_lengths;
```

- [ ] **Step 3: Run tests**

Run: `cargo test algo::shortest_path`
Expected: All 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/algo/shortest_path.rs src/algo/mod.rs
git commit -m "feat: add unweighted shortest path (BFS-based)"
```

---

### Task 8: Degree distribution

**Files:**
- Modify: `src/graph.rs`

- [ ] **Step 1: Add `degree_distribution` to the `Graph` trait and as a free function**

In `src/graph.rs`, add this method to the `Graph` trait (after `degree_sequence`):

```rust
    /// Degree distribution: `result[d]` is the number of vertices with degree `d`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
    /// assert_eq!(g.degree_distribution(), vec![0, 3, 0, 1]);
    /// ```
    fn degree_distribution(&self) -> Vec<usize> {
        let mut dist = vec![0usize; self.nv()];
        for v in 0..self.nv() as u32 {
            let d = self.degree(v);
            if d >= dist.len() {
                dist.resize(d + 1, 0);
            }
            dist[d] += 1;
        }
        // Trim trailing zeros
        while dist.last() == Some(&0) {
            dist.pop();
        }
        dist
    }
```

Add this free function after the existing `degree_sequence` free function:

```rust
/// Degree distribution: `result[d]` is the number of vertices with degree `d`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, Graph};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
/// let dist = g.degree_distribution();
/// assert_eq!(dist, vec![0, 3, 0, 1]); // three degree-1, one degree-3
/// ```
pub fn degree_distribution(g: &impl Graph) -> Vec<usize> {
    g.degree_distribution()
}
```

Update the re-export in `src/lib.rs`:

```rust
pub use graph::{degree_distribution, degree_sequence, density, Graph};
```

- [ ] **Step 2: Add tests**

Add these tests to the existing `mod tests` in `src/graph.rs`:

```rust
    #[test]
    fn test_degree_distribution() {
        // Star graph: vertex 0 has degree 3, vertices 1,2,3 have degree 1
        let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
        let dist = degree_distribution(&g);
        assert_eq!(dist, vec![0, 3, 0, 1]);
    }

    #[test]
    fn test_degree_distribution_empty() {
        let g = SimpleGraph::new(0);
        let dist = degree_distribution(&g);
        assert!(dist.is_empty());
    }

    #[test]
    fn test_degree_distribution_isolated() {
        let g = SimpleGraph::new(3);
        let dist = degree_distribution(&g);
        assert_eq!(dist, vec![3]);
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test graph::tests`
Expected: All tests pass including new ones.

- [ ] **Step 4: Commit**

```bash
git add src/graph.rs src/lib.rs
git commit -m "feat: add degree_distribution to Graph trait"
```

---

### Task 9: Doc examples

**Files:**
- Modify: `src/simple_graph.rs` (key methods)
- Modify: `src/csr.rs` (key methods)
- Modify: `src/graph.rs` (trait methods)
- Modify: `src/algo/bfs.rs` (public functions)
- Modify: `src/algo/dfs.rs` (public function)
- Modify: `src/algo/shortest_path.rs` (public function)
- Modify: `src/io/edge_list.rs` (public functions)
- Modify: `src/io/matrix_market.rs` (public functions)
- Modify: `src/gen/basic.rs` (public functions)

- [ ] **Step 1: Add doc examples to `SimpleGraph` methods**

Add doc examples to the key methods in `src/simple_graph.rs`. Place these just below the existing doc comments for each method:

For `SimpleGraph::new`:
```rust
    /// Create an empty graph with `n` vertices and no edges.
    ///
    /// # Panics
    /// Panics if `n > u32::MAX as usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::new(5);
    /// assert_eq!(g.nv(), 5);
    /// assert_eq!(g.ne(), 0);
    /// ```
```

For `SimpleGraph::from_edges`:
```rust
    /// Create a graph from a list of edges.
    ///
    /// Duplicate edges are silently collapsed. Panics on self-loops or
    /// out-of-range vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// assert_eq!(g.ne(), 2);
    /// assert!(g.has_edge(0, 1));
    /// ```
```

For `SimpleGraph::add_edge`:
```rust
    /// Add undirected edge. No-op if the edge already exists.
    ///
    /// # Panics
    /// Panics on self-loops or if a vertex is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let mut g = SimpleGraph::new(3);
    /// g.add_edge(0, 1);
    /// assert!(g.has_edge(0, 1));
    /// assert!(g.has_edge(1, 0)); // undirected
    /// ```
```

For `SimpleGraph::edges`:
```rust
    /// Iterator over all edges `(u, v)` with `u < v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// let edges: Vec<_> = g.edges().collect();
    /// assert_eq!(edges, vec![(0, 1), (1, 2)]);
    /// ```
```

- [ ] **Step 2: Add doc examples to `Graph` trait methods**

In `src/graph.rs`, add examples to `density` and `degree_sequence` trait default methods:

For `density`:
```rust
    /// Graph density: `ne / (nv choose 2)`. Returns 0.0 for graphs with < 2 vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2), (1, 3)]);
    /// assert!((g.density() - 1.0).abs() < 1e-10);
    /// ```
```

For `degree_sequence`:
```rust
    /// Sorted degree sequence (ascending).
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
    /// assert_eq!(g.degree_sequence(), vec![1, 1, 1, 3]);
    /// ```
```

- [ ] **Step 3: Add doc examples to algorithm functions**

For `bfs` in `src/algo/bfs.rs`:
```rust
/// Return a BFS iterator starting from `source`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (1, 3)]);
/// let order: Vec<u32> = algo::bfs(&g, 0).collect();
/// assert_eq!(order, vec![0, 1, 2, 3]);
/// ```
```

For `is_connected` in `src/algo/bfs.rs`:
```rust
/// Whether the graph is connected (empty and single-vertex graphs are connected).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
/// assert!(algo::is_connected(&g));
/// ```
```

For `connected_components` in `src/algo/bfs.rs`:
```rust
/// Assign a component label to each vertex. Labels are `0, 1, 2, ...` assigned
/// in order of discovery. Returns a `Vec<u32>` of length `nv()`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (2, 3)]);
/// let labels = algo::connected_components(&g);
/// assert_eq!(labels[0], labels[1]); // same component
/// assert_ne!(labels[0], labels[2]); // different components
/// ```
```

For `dfs` in `src/algo/dfs.rs`:
```rust
/// Return a DFS iterator starting from `source`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
/// let order: Vec<u32> = algo::dfs(&g, 0).collect();
/// assert_eq!(order.len(), 4);
/// ```
```

For `shortest_path_lengths` in `src/algo/shortest_path.rs`:
```rust
/// Compute unweighted shortest-path distances from `source` to all reachable vertices.
///
/// Returns a `Vec<Option<u32>>` of length `nv()`. `result[v]` is `Some(d)` if
/// vertex `v` is reachable from `source` in `d` hops, or `None` if unreachable.
/// `result[source]` is `Some(0)`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
/// let dist = algo::shortest_path_lengths(&g, 0);
/// assert_eq!(dist, vec![Some(0), Some(1), Some(2), Some(3)]);
/// ```
```

- [ ] **Step 4: Add doc examples to I/O functions**

For `write_edge_list` in `src/io/edge_list.rs`:
```rust
/// Write a graph in edge-list format.
/// Format: first line is `nv ne`, followed by one `u v` line per edge (u < v).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, io};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
/// let mut buf = Vec::new();
/// io::write_edge_list(&g, &mut buf).unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.starts_with("3 2"));
/// ```
```

For `read_edge_list` in `src/io/edge_list.rs`:
```rust
/// Read a graph from edge-list format.
/// Lines starting with `#` or `%` are skipped. First non-comment line must be
/// `nv ne`. Remaining lines are `u v` edges. Returns error if declared edge
/// count doesn't match, or if edges contain self-loops/out-of-range vertices.
///
/// # Examples
///
/// ```
/// use simple_graph::io;
///
/// let input = b"3 2\n0 1\n1 2\n";
/// let g = io::read_edge_list(&input[..]).unwrap();
/// assert_eq!(g.nv(), 3);
/// assert_eq!(g.ne(), 2);
/// ```
```

For `write_matrix_market` in `src/io/matrix_market.rs`:
```rust
/// Write a graph in Matrix Market symmetric coordinate pattern format.
/// Uses 1-based indexing. Writes the lower triangle (row > col).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, io};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1)]);
/// let mut buf = Vec::new();
/// io::write_matrix_market(&g, &mut buf).unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.starts_with("%%MatrixMarket"));
/// ```
```

For `read_matrix_market` in `src/io/matrix_market.rs`:
```rust
/// Read a graph from Matrix Market symmetric coordinate pattern format.
/// Validates banner for "coordinate pattern symmetric". Converts 1-based to 0-based.
/// Errors on invalid format, count mismatches, self-loops, OOB vertices.
///
/// # Examples
///
/// ```
/// use simple_graph::io;
///
/// let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n1 2\n";
/// let g = io::read_matrix_market(&input[..]).unwrap();
/// assert_eq!(g.nv(), 3);
/// assert!(g.has_edge(0, 1));
/// ```
```

- [ ] **Step 5: Add doc examples to generators**

For `complete` in `src/gen/basic.rs`:
```rust
/// Complete graph K_n.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::complete(4);
/// assert_eq!(g.ne(), 6); // 4 choose 2
/// ```
```

For `cycle` in `src/gen/basic.rs`:
```rust
/// Cycle graph C_n (n >= 3).
///
/// # Panics
/// Panics if `n < 3`.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::cycle(5);
/// assert_eq!(g.ne(), 5);
/// assert!(g.has_edge(0, 4)); // wraps around
/// ```
```

For `path` in `src/gen/basic.rs`:
```rust
/// Path graph P_n (n vertices, n-1 edges).
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::path(4);
/// assert_eq!(g.ne(), 3);
/// ```
```

For `grid_2d` in `src/gen/basic.rs`:
```rust
/// 2D grid graph with `rows` x `cols` vertices.
/// Vertex index for position (r, c) is `r * cols + c`.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::grid_2d(3, 4);
/// assert_eq!(g.nv(), 12);
/// ```
```

- [ ] **Step 6: Run doc tests**

Run: `cargo test --doc --all-features`
Expected: All doc tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/
git commit -m "docs: add Examples blocks to all public API methods"
```

---

### Task 10: `Send + Sync` static assertions

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add compile-time assertions**

At the bottom of `src/lib.rs`, add:

```rust
// Compile-time assertions: all public types are Send + Sync.
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assertions() {
        assert_send_sync::<SimpleGraph>();
        assert_send_sync::<CsrGraph>();
        assert_send_sync::<Edges<'_, SimpleGraph>>();
        assert_send_sync::<Edges<'_, CsrGraph>>();
        assert_send_sync::<algo::Bfs<'_, SimpleGraph>>();
        assert_send_sync::<algo::Bfs<'_, CsrGraph>>();
        assert_send_sync::<algo::Dfs<'_, SimpleGraph>>();
        assert_send_sync::<algo::Dfs<'_, CsrGraph>>();
    }
};
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles successfully. If any type fails `Send + Sync`, this is a compile error.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "chore: add Send + Sync compile-time static assertions"
```

---

## Summary

| Task | Checklist Item | Category |
|------|---------------|----------|
| 1 | `#![forbid(unsafe_code)]` | Must have |
| 2 | Cargo.toml metadata | Must have |
| 3 | CHANGELOG.md | Must have |
| 4 | Criterion benchmarks | Must have |
| 5 | Fuzz testing | Must have |
| 6 | DFS algorithm | Should have |
| 7 | Unweighted shortest path | Should have |
| 8 | Degree distribution | Should have |
| 9 | Doc examples | Should have |
| 10 | Send + Sync assertions | Should have |
