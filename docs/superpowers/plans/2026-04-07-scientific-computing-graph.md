# Scientific Computing Graph Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evolve simple-graph into a scientific-computing-ready undirected graph library with a shared trait, CSR storage, core algorithms, I/O, and basic generators.

**Architecture:** A read-only `Graph` trait abstracts over `SimpleGraph` (mutable, sorted adjacency lists) and `CsrGraph` (immutable, contiguous memory). Algorithms and iterators are generic over `<G: Graph>` (monomorphized, not `dyn`) for maximum performance. I/O uses edge-list and Matrix Market formats. Generators produce `SimpleGraph` which can be converted to CSR for analysis.

**Tech Stack:** Rust 2021 edition, serde (optional feature), `rand` (optional, for generators), no other required dependencies.

**Task dependency graph:**
- Task 1 (Graph trait) — foundation, everything depends on it
- Task 2 (CsrGraph) — depends on Task 1
- Task 3 (Generalize Edges) — depends on Task 2 (tests use CsrGraph)
- Tasks 4, 5, 6, 9 — can run in parallel after Task 3
- Task 7 (generators) — depends on Task 4 (tests use `is_connected`)
- Task 8 (Erdos-Renyi) — depends on Task 7 + Task 9 (feature table)

---

## File Structure

```
src/
  lib.rs              — re-exports, module declarations
  graph.rs            — Graph trait definition + density/degree_sequence
  simple_graph.rs     — existing SimpleGraph (add Graph impl, try_from_edges)
  csr.rs              — CsrGraph struct + Graph impl + From<&SimpleGraph> + to_simple_graph()
  iter.rs             — Edges iterator (generic over G: Graph)
  algo/
    mod.rs            — algorithm module re-exports
    bfs.rs            — BFS traversal, connected_components, is_connected
  io/
    mod.rs            — I/O module re-exports
    edge_list.rs      — edge list text format read/write
    matrix_market.rs  — Matrix Market symmetric pattern format read/write
  gen/
    mod.rs            — generator module re-exports
    basic.rs          — complete, cycle, path, grid_2d
    random.rs         — erdos_renyi (behind "rand" feature)
Cargo.toml            — optional deps: serde (default), rand
```

---

### Task 1: Graph Trait

**Files:**
- Create: `src/graph.rs`
- Modify: `src/lib.rs`
- Modify: `src/simple_graph.rs`
- Test: `src/graph.rs` (inline tests)

- [ ] **Step 1: Write the failing test**

In `src/graph.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleGraph;

    fn assert_graph_basics(g: &impl Graph) {
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 2);
        assert!(g.has_vertex(0));
        assert!(!g.has_vertex(3));
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 0));
        assert!(!g.has_edge(0, 2));
        assert_eq!(g.degree(0), 1);
        assert_eq!(g.neighbors(0), &[1]);
    }

    #[test]
    fn test_simple_graph_implements_graph_trait() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        assert_graph_basics(&g);
    }

    #[test]
    fn test_density() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let d = density(&g);
        // 4 edges / 6 possible = 0.666...
        assert!((d - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_density_empty() {
        let g = SimpleGraph::new(0);
        assert_eq!(density(&g), 0.0);
    }

    #[test]
    fn test_density_single() {
        let g = SimpleGraph::new(1);
        assert_eq!(density(&g), 0.0);
    }

    #[test]
    fn test_degree_sequence() {
        // 0-1, 0-2, 0-3, 1-2 => degrees: 3,2,2,1
        let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3), (1, 2)]);
        assert_eq!(degree_sequence(&g), vec![1, 2, 2, 3]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test graph::tests -- --nocapture`
Expected: FAIL — `Graph` trait does not exist yet.

- [ ] **Step 3: Define the Graph trait, helpers, and implement for SimpleGraph**

In `src/graph.rs`:

```rust
/// Read-only interface shared by all graph representations.
pub trait Graph {
    /// Number of vertices.
    fn nv(&self) -> usize;
    /// Number of edges.
    fn ne(&self) -> usize;
    /// Whether vertex `v` exists.
    fn has_vertex(&self, v: u32) -> bool;
    /// Whether edge `(u, v)` exists.
    fn has_edge(&self, u: u32, v: u32) -> bool;
    /// Degree of vertex `v`.
    fn degree(&self, v: u32) -> usize;
    /// Sorted neighbor slice of vertex `v`.
    fn neighbors(&self, v: u32) -> &[u32];
}

/// Graph density: `ne / (nv choose 2)`. Returns 0.0 for graphs with < 2 vertices.
pub fn density(g: &impl Graph) -> f64 {
    let n = g.nv();
    if n < 2 {
        return 0.0;
    }
    let max_edges = n * (n - 1) / 2;
    g.ne() as f64 / max_edges as f64
}

/// Sorted degree sequence (ascending).
pub fn degree_sequence(g: &impl Graph) -> Vec<usize> {
    let mut seq: Vec<usize> = (0..g.nv() as u32).map(|v| g.degree(v)).collect();
    seq.sort_unstable();
    seq
}
```

In `src/simple_graph.rs`, add the trait impl (use fully qualified calls to avoid recursion):

```rust
use crate::Graph;

impl Graph for SimpleGraph {
    #[inline]
    fn nv(&self) -> usize { SimpleGraph::nv(self) }
    #[inline]
    fn ne(&self) -> usize { SimpleGraph::ne(self) }
    #[inline]
    fn has_vertex(&self, v: u32) -> bool { SimpleGraph::has_vertex(self, v) }
    fn has_edge(&self, u: u32, v: u32) -> bool { SimpleGraph::has_edge(self, u, v) }
    #[inline]
    fn degree(&self, v: u32) -> usize { SimpleGraph::degree(self, v) }
    #[inline]
    fn neighbors(&self, v: u32) -> &[u32] { SimpleGraph::neighbors(self, v) }
}
```

In `src/lib.rs`, add:

```rust
mod graph;
pub use graph::{density, degree_sequence, Graph};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All 15 existing tests + 5 new tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/graph.rs src/lib.rs src/simple_graph.rs
git commit -m "feat: add Graph trait with density and degree_sequence"
```

---

### Task 2: CsrGraph

**Files:**
- Create: `src/csr.rs`
- Modify: `src/lib.rs`
- Test: `src/csr.rs` (inline tests)

- [ ] **Step 1: Write the failing tests**

In `src/csr.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, SimpleGraph};

    #[test]
    fn test_csr_from_simple_graph() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.nv(), 4);
        assert_eq!(csr.ne(), 3);
    }

    #[test]
    fn test_csr_has_edge() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        assert!(csr.has_edge(0, 1));
        assert!(csr.has_edge(1, 0));
        assert!(!csr.has_edge(0, 2));
    }

    #[test]
    fn test_csr_neighbors() {
        let sg = SimpleGraph::from_edges(4, &[(0, 3), (0, 1), (0, 2)]);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.neighbors(0), &[1, 2, 3]);
        assert_eq!(csr.neighbors(1), &[0]);
        assert_eq!(csr.degree(0), 3);
    }

    #[test]
    fn test_csr_empty() {
        let sg = SimpleGraph::new(0);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.nv(), 0);
        assert_eq!(csr.ne(), 0);
    }

    #[test]
    fn test_csr_oob_returns_false() {
        let sg = SimpleGraph::from_edges(2, &[(0, 1)]);
        let csr = CsrGraph::from(&sg);
        assert!(!csr.has_edge(5, 0));
        assert!(!csr.has_vertex(2));
    }

    #[test]
    fn test_csr_graph_trait() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        fn check(g: &impl Graph) {
            assert_eq!(g.nv(), 3);
            assert_eq!(g.ne(), 2);
            assert!(g.has_edge(0, 1));
        }
        check(&csr);
    }

    #[test]
    fn test_csr_invariants() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        // offsets length is nv + 1
        assert_eq!(csr.offsets.len(), 5);
        // offsets are monotonically non-decreasing
        for i in 1..csr.offsets.len() {
            assert!(csr.offsets[i] >= csr.offsets[i - 1]);
        }
        // targets length equals 2 * ne (each edge stored twice)
        assert_eq!(csr.targets.len(), 2 * csr.ne());
    }

    #[test]
    fn test_csr_to_simple_graph() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        let sg2 = csr.to_simple_graph();
        assert_eq!(sg, sg2);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test csr -- --nocapture`
Expected: FAIL — `CsrGraph` does not exist.

- [ ] **Step 3: Implement CsrGraph**

In `src/csr.rs`:

```rust
use crate::graph::Graph;
use crate::SimpleGraph;

/// Compressed Sparse Row graph — immutable, contiguous-memory storage.
///
/// Built from a `SimpleGraph` via `From`. All neighbor data lives in a single
/// `Vec<u32>`, indexed by a `Vec<usize>` of offsets. Zero pointer indirection
/// per vertex, ideal for read-heavy scientific workloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrGraph {
    ne: usize,
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<u32>,
}

impl CsrGraph {
    /// Number of vertices.
    #[inline]
    pub fn nv(&self) -> usize {
        if self.offsets.is_empty() { 0 } else { self.offsets.len() - 1 }
    }

    /// Number of edges.
    #[inline]
    pub fn ne(&self) -> usize {
        self.ne
    }

    /// Whether vertex `v` exists.
    #[inline]
    pub fn has_vertex(&self, v: u32) -> bool {
        (v as usize) < self.nv()
    }

    /// Whether edge `(u, v)` exists. Returns `false` for out-of-range vertices.
    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        if !self.has_vertex(u) || !self.has_vertex(v) {
            return false;
        }
        self.neighbors(u).binary_search(&v).is_ok()
    }

    /// Degree of vertex `v`.
    #[inline]
    pub fn degree(&self, v: u32) -> usize {
        let vi = v as usize;
        self.offsets[vi + 1] - self.offsets[vi]
    }

    /// Sorted neighbor slice of vertex `v`.
    #[inline]
    pub fn neighbors(&self, v: u32) -> &[u32] {
        let vi = v as usize;
        &self.targets[self.offsets[vi]..self.offsets[vi + 1]]
    }

    /// Convert back to a mutable `SimpleGraph`.
    pub fn to_simple_graph(&self) -> SimpleGraph {
        SimpleGraph::from_csr(&self.offsets, &self.targets, self.ne)
    }
}

impl From<&SimpleGraph> for CsrGraph {
    fn from(sg: &SimpleGraph) -> Self {
        let n = sg.nv();
        let mut offsets = Vec::with_capacity(n + 1);
        let total: usize = (0..n).map(|v| sg.neighbors(v as u32).len()).sum();
        let mut targets = Vec::with_capacity(total);
        let mut offset = 0;
        for v in 0..n {
            offsets.push(offset);
            let nbrs = sg.neighbors(v as u32);
            targets.extend_from_slice(nbrs);
            offset += nbrs.len();
        }
        offsets.push(offset);
        CsrGraph {
            ne: sg.ne(),
            offsets,
            targets,
        }
    }
}

impl Graph for CsrGraph {
    #[inline]
    fn nv(&self) -> usize { CsrGraph::nv(self) }
    #[inline]
    fn ne(&self) -> usize { CsrGraph::ne(self) }
    #[inline]
    fn has_vertex(&self, v: u32) -> bool { CsrGraph::has_vertex(self, v) }
    fn has_edge(&self, u: u32, v: u32) -> bool { CsrGraph::has_edge(self, u, v) }
    #[inline]
    fn degree(&self, v: u32) -> usize { CsrGraph::degree(self, v) }
    #[inline]
    fn neighbors(&self, v: u32) -> &[u32] { CsrGraph::neighbors(self, v) }
}
```

In `src/simple_graph.rs`, add a `pub(crate)` constructor for CsrGraph reverse conversion:

```rust
impl SimpleGraph {
    /// Construct from CSR data (used by CsrGraph::to_simple_graph).
    pub(crate) fn from_csr(offsets: &[usize], targets: &[u32], ne: usize) -> Self {
        let n = if offsets.is_empty() { 0 } else { offsets.len() - 1 };
        let mut fadjlist = Vec::with_capacity(n);
        for v in 0..n {
            fadjlist.push(targets[offsets[v]..offsets[v + 1]].to_vec());
        }
        Self { ne, fadjlist }
    }
}
```

In `src/lib.rs`, add:

```rust
mod csr;
pub use csr::CsrGraph;
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All existing + 9 new CSR tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/csr.rs src/simple_graph.rs src/lib.rs
git commit -m "feat: add CsrGraph with contiguous memory layout and reverse conversion"
```

---

### Task 3: Generalize Edges Iterator + Add edges() to Trait

**Files:**
- Modify: `src/iter.rs`
- Modify: `src/graph.rs`
- Modify: `src/csr.rs`
- Modify: `src/simple_graph.rs`
- Test: `src/iter.rs` (inline tests)

- [ ] **Step 1: Write the failing test**

Add to `src/iter.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CsrGraph, Graph, SimpleGraph};

    #[test]
    fn test_edges_over_csr() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        let edges: Vec<(u32, u32)> = edges(&csr).collect();
        assert_eq!(edges, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn test_edges_generic() {
        fn collect_edges(g: &impl Graph) -> Vec<(u32, u32)> {
            edges(g).collect()
        }
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        assert_eq!(collect_edges(&g), vec![(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&g);
        assert_eq!(collect_edges(&csr), vec![(0, 1), (1, 2)]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test iter::tests -- --nocapture`
Expected: FAIL — `edges` free function does not exist.

- [ ] **Step 3: Rewrite Edges as generic, add free function**

Replace `src/iter.rs`:

```rust
use std::iter::FusedIterator;

use crate::graph::Graph;

/// Iterator over edges of any [`Graph`], yielding `(u, v)` with `u < v`.
pub struct Edges<'a, G: Graph + ?Sized> {
    graph: &'a G,
    u: u32,
    idx: usize,
}

/// Create an edge iterator for any graph.
pub fn edges<G: Graph>(graph: &G) -> Edges<'_, G> {
    let mut iter = Edges {
        graph,
        u: 0,
        idx: 0,
    };
    iter.skip_to_upper();
    iter
}

impl<'a, G: Graph + ?Sized> Edges<'a, G> {
    fn skip_to_upper(&mut self) {
        let nv = self.graph.nv() as u32;
        if self.u < nv {
            let nbrs = self.graph.neighbors(self.u);
            self.idx = nbrs.partition_point(|&v| v <= self.u);
        }
    }
}

impl<'a, G: Graph + ?Sized> Iterator for Edges<'a, G> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let nv = self.graph.nv() as u32;
        while self.u < nv {
            let nbrs = self.graph.neighbors(self.u);
            if self.idx < nbrs.len() {
                let v = nbrs[self.idx];
                self.idx += 1;
                return Some((self.u, v));
            }
            self.u += 1;
            self.skip_to_upper();
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.graph.ne()))
    }
}

impl<'a, G: Graph + ?Sized> FusedIterator for Edges<'a, G> {}
```

Update `src/simple_graph.rs` — change `edges()` method and add `IntoIterator`:

```rust
use crate::iter;

impl SimpleGraph {
    /// Iterator over all edges `(u, v)` with `u < v`.
    pub fn edges(&self) -> iter::Edges<'_, Self> {
        iter::edges(self)
    }
}

impl<'a> IntoIterator for &'a SimpleGraph {
    type Item = (u32, u32);
    type IntoIter = iter::Edges<'a, SimpleGraph>;
    fn into_iter(self) -> Self::IntoIter { self.edges() }
}
```

Update `src/csr.rs` — add `edges()` method and `IntoIterator`:

```rust
use crate::iter;

impl CsrGraph {
    /// Iterator over all edges `(u, v)` with `u < v`.
    pub fn edges(&self) -> iter::Edges<'_, Self> {
        iter::edges(self)
    }
}

impl<'a> IntoIterator for &'a CsrGraph {
    type Item = (u32, u32);
    type IntoIter = iter::Edges<'a, CsrGraph>;
    fn into_iter(self) -> Self::IntoIter { self.edges() }
}
```

Update `src/lib.rs` exports:

```rust
pub use iter::{edges, Edges};
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass including new generic edge tests.

- [ ] **Step 5: Commit**

```bash
git add src/iter.rs src/simple_graph.rs src/csr.rs src/lib.rs
git commit -m "feat: generalize Edges iterator with monomorphized generics"
```

---

### Task 4: BFS and Connected Components

**Files:**
- Create: `src/algo/mod.rs`
- Create: `src/algo/bfs.rs`
- Modify: `src/lib.rs`
- Test: `src/algo/bfs.rs` (inline tests)

- [ ] **Step 1: Write the failing tests**

In `src/algo/bfs.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CsrGraph, SimpleGraph};

    #[test]
    fn test_bfs_order() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (1, 3)]);
        let order: Vec<u32> = bfs(&g, 0).collect();
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_bfs_disconnected() {
        let g = SimpleGraph::new(4);
        let order: Vec<u32> = bfs(&g, 0).collect();
        assert_eq!(order, vec![0]);
    }

    #[test]
    fn test_bfs_on_csr() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (1, 3)]);
        let csr = CsrGraph::from(&sg);
        let order: Vec<u32> = bfs(&csr, 0).collect();
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_is_connected_true() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        assert!(is_connected(&g));
    }

    #[test]
    fn test_is_connected_false() {
        let g = SimpleGraph::new(3);
        assert!(!is_connected(&g));
    }

    #[test]
    fn test_is_connected_empty() {
        let g = SimpleGraph::new(0);
        assert!(is_connected(&g));
    }

    #[test]
    fn test_is_connected_single() {
        let g = SimpleGraph::new(1);
        assert!(is_connected(&g));
    }

    #[test]
    fn test_is_connected_csr() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        assert!(is_connected(&csr));
    }

    #[test]
    fn test_connected_components() {
        // 0-1-2  3-4  5
        let g = SimpleGraph::from_edges(6, &[(0, 1), (1, 2), (3, 4)]);
        let labels = connected_components(&g);
        assert_eq!(labels.len(), 6);
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert_eq!(labels[3], labels[4]);
        assert_ne!(labels[0], labels[3]);
        assert_ne!(labels[0], labels[5]);
        assert_ne!(labels[3], labels[5]);
    }

    #[test]
    fn test_connected_components_count() {
        let g = SimpleGraph::from_edges(6, &[(0, 1), (1, 2), (3, 4)]);
        let labels = connected_components(&g);
        let mut unique: Vec<u32> = labels.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 3);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test algo::bfs -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement BFS, is_connected, connected_components**

In `src/algo/bfs.rs`:

```rust
use std::collections::VecDeque;

use crate::graph::Graph;

/// BFS iterator from a source vertex.
pub struct Bfs<'a, G: Graph + ?Sized> {
    graph: &'a G,
    queue: VecDeque<u32>,
    visited: Vec<bool>,
}

/// Return a BFS iterator starting from `source`.
pub fn bfs<G: Graph>(graph: &G, source: u32) -> Bfs<'_, G> {
    let n = graph.nv();
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    if graph.has_vertex(source) {
        visited[source as usize] = true;
        queue.push_back(source);
    }
    Bfs {
        graph,
        queue,
        visited,
    }
}

impl<'a, G: Graph + ?Sized> Iterator for Bfs<'a, G> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let u = self.queue.pop_front()?;
        for &v in self.graph.neighbors(u) {
            if !self.visited[v as usize] {
                self.visited[v as usize] = true;
                self.queue.push_back(v);
            }
        }
        Some(u)
    }
}

/// Whether the graph is connected (empty and single-vertex graphs are connected).
pub fn is_connected<G: Graph>(graph: &G) -> bool {
    let n = graph.nv();
    if n <= 1 {
        return true;
    }
    bfs(graph, 0).count() == n
}

/// Assign a component label to each vertex. Labels are `0, 1, 2, ...` assigned
/// in order of discovery. Returns a `Vec<u32>` of length `nv()`.
pub fn connected_components<G: Graph>(graph: &G) -> Vec<u32> {
    let n = graph.nv();
    let mut labels = vec![u32::MAX; n];
    let mut component = 0u32;
    for start in 0..n as u32 {
        if labels[start as usize] != u32::MAX {
            continue;
        }
        for v in bfs(graph, start) {
            labels[v as usize] = component;
        }
        component += 1;
    }
    labels
}
```

In `src/algo/mod.rs`:

```rust
mod bfs;

pub use bfs::{bfs, connected_components, is_connected, Bfs};
```

In `src/lib.rs`, add:

```rust
pub mod algo;
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/algo/mod.rs src/algo/bfs.rs src/lib.rs
git commit -m "feat: add BFS, is_connected, connected_components (generic)"
```

---

### Task 5: try_from_edges + Edge List I/O

**Files:**
- Modify: `src/simple_graph.rs` (add `try_from_edges`)
- Create: `src/io/mod.rs`
- Create: `src/io/edge_list.rs`
- Modify: `src/lib.rs`
- Test: `src/io/edge_list.rs` (inline tests)

- [ ] **Step 1: Write the failing tests for try_from_edges**

In `src/simple_graph.rs` test module, add:

```rust
#[test]
fn test_try_from_edges_ok() {
    let g = SimpleGraph::try_from_edges(3, &[(0, 1), (1, 2)]).unwrap();
    assert_eq!(g.nv(), 3);
    assert_eq!(g.ne(), 2);
}

#[test]
fn test_try_from_edges_self_loop() {
    assert!(SimpleGraph::try_from_edges(3, &[(0, 0)]).is_err());
}

#[test]
fn test_try_from_edges_oob() {
    assert!(SimpleGraph::try_from_edges(3, &[(0, 5)]).is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_try_from_edges -- --nocapture`
Expected: FAIL — method does not exist.

- [ ] **Step 3: Implement try_from_edges**

In `src/simple_graph.rs`:

```rust
impl SimpleGraph {
    /// Fallible version of `from_edges`. Returns an error string on self-loops
    /// or out-of-range vertices.
    pub fn try_from_edges(n: usize, edges: &[(u32, u32)]) -> Result<Self, String> {
        let mut fadjlist: Vec<Vec<u32>> = vec![vec![]; n];
        for &(u, v) in edges {
            if u == v {
                return Err(format!("self-loop on vertex {}", u));
            }
            if (u as usize) >= n || (v as usize) >= n {
                return Err(format!("vertex out of range: ({}, {}), n={}", u, v, n));
            }
            fadjlist[u as usize].push(v);
            fadjlist[v as usize].push(u);
        }
        let mut ne = 0;
        for list in &mut fadjlist {
            list.sort_unstable();
            list.dedup();
            ne += list.len();
        }
        Ok(Self {
            ne: ne / 2,
            fadjlist,
        })
    }
}
```

- [ ] **Step 4: Run try_from_edges tests**

Run: `cargo test test_try_from_edges`
Expected: PASS

- [ ] **Step 5: Write the failing I/O tests**

In `src/io/edge_list.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleGraph;

    #[test]
    fn test_write_read_roundtrip() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let mut buf = Vec::new();
        write_edge_list(&g, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(text.lines().count(), 4); // header + 3 edges
        let g2 = read_edge_list(text.as_bytes()).unwrap();
        assert_eq!(g2.nv(), 4);
        assert_eq!(g2.ne(), 3);
        assert!(g2.has_edge(0, 1));
        assert!(g2.has_edge(2, 3));
    }

    #[test]
    fn test_read_with_comments() {
        let input = b"# comment\n3 1\n0 1\n";
        let g = read_edge_list(&input[..]).unwrap();
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_empty_graph_roundtrip() {
        let g = SimpleGraph::new(5);
        let mut buf = Vec::new();
        write_edge_list(&g, &mut buf).unwrap();
        let g2 = read_edge_list(&buf[..]).unwrap();
        assert_eq!(g2.nv(), 5);
        assert_eq!(g2.ne(), 0);
    }

    #[test]
    fn test_read_self_loop_error() {
        let input = b"3 1\n1 1\n";
        assert!(read_edge_list(&input[..]).is_err());
    }

    #[test]
    fn test_read_oob_error() {
        let input = b"3 1\n0 5\n";
        assert!(read_edge_list(&input[..]).is_err());
    }

    #[test]
    fn test_read_count_mismatch_error() {
        let input = b"3 2\n0 1\n"; // declares 2 edges, only 1 present
        assert!(read_edge_list(&input[..]).is_err());
    }
}
```

- [ ] **Step 6: Run I/O tests to verify they fail**

Run: `cargo test io::edge_list -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 7: Implement edge list read/write**

In `src/io/edge_list.rs`:

```rust
use std::io::{self, BufRead, Write};

use crate::{graph::Graph, SimpleGraph};

/// Write a graph in edge-list format.
///
/// Format: first line is `nv ne`, followed by one `u v` line per edge (u < v).
pub fn write_edge_list<G: Graph>(graph: &G, mut w: impl Write) -> io::Result<()> {
    writeln!(w, "{} {}", graph.nv(), graph.ne())?;
    for v in 0..graph.nv() as u32 {
        for &u in graph.neighbors(v) {
            if u > v {
                writeln!(w, "{} {}", v, u)?;
            }
        }
    }
    Ok(())
}

/// Read a graph from edge-list format.
///
/// Lines starting with `#` or `%` are skipped. First non-comment line must be
/// `nv ne`. Remaining lines are `u v` edges. Returns an error if the declared
/// edge count does not match, or if edges contain self-loops or out-of-range
/// vertices.
pub fn read_edge_list(r: impl BufRead) -> io::Result<SimpleGraph> {
    let mut lines = r.lines();
    let (nv, ne_declared) = loop {
        let line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty input"))??;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let nv: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nv"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let ne: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing ne"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        break (nv, ne);
    };
    let mut edges = Vec::new();
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let u: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing u"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let v: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing v"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        edges.push((u, v));
    }
    if edges.len() != ne_declared {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "declared {} edges but found {}",
                ne_declared,
                edges.len()
            ),
        ));
    }
    SimpleGraph::try_from_edges(nv, &edges)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

In `src/io/mod.rs`:

```rust
mod edge_list;

pub use edge_list::{read_edge_list, write_edge_list};
```

In `src/lib.rs`, add:

```rust
pub mod io;
```

- [ ] **Step 8: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/simple_graph.rs src/io/mod.rs src/io/edge_list.rs src/lib.rs
git commit -m "feat: add try_from_edges and edge list I/O with validation"
```

---

### Task 6: Matrix Market I/O

**Files:**
- Create: `src/io/matrix_market.rs`
- Modify: `src/io/mod.rs`
- Test: `src/io/matrix_market.rs` (inline tests)

- [ ] **Step 1: Write the failing tests**

In `src/io/matrix_market.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleGraph;

    #[test]
    fn test_write_read_roundtrip() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let mut buf = Vec::new();
        write_matrix_market(&g, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("%%MatrixMarket"));
        let g2 = read_matrix_market(text.as_bytes()).unwrap();
        assert_eq!(g2.nv(), 4);
        assert_eq!(g2.ne(), 3);
        assert!(g2.has_edge(0, 1));
        assert!(g2.has_edge(2, 3));
    }

    #[test]
    fn test_read_1indexed() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 2\n1 2\n2 3\n";
        let g = read_matrix_market(&input[..]).unwrap();
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 2);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 2));
    }

    #[test]
    fn test_empty_graph() {
        let g = SimpleGraph::new(3);
        let mut buf = Vec::new();
        write_matrix_market(&g, &mut buf).unwrap();
        let g2 = read_matrix_market(&buf[..]).unwrap();
        assert_eq!(g2.nv(), 3);
        assert_eq!(g2.ne(), 0);
    }

    #[test]
    fn test_reject_non_symmetric() {
        let input = b"%%MatrixMarket matrix coordinate pattern general\n3 3 1\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_non_square() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 4 1\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_zero_index() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n0 1\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_count_mismatch() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 2\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test io::matrix_market -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement Matrix Market read/write**

In `src/io/matrix_market.rs`:

```rust
use std::io::{self, BufRead, Write};

use crate::{graph::Graph, SimpleGraph};

/// Write a graph in Matrix Market symmetric coordinate pattern format.
///
/// Uses 1-based indexing per the MM spec. Writes the lower triangle (row > col).
pub fn write_matrix_market<G: Graph>(graph: &G, mut w: impl Write) -> io::Result<()> {
    writeln!(w, "%%MatrixMarket matrix coordinate pattern symmetric")?;
    let n = graph.nv();
    writeln!(w, "{} {} {}", n, n, graph.ne())?;
    for v in 0..n as u32 {
        for &u in graph.neighbors(v) {
            if u > v {
                writeln!(w, "{} {}", u + 1, v + 1)?;
            }
        }
    }
    Ok(())
}

/// Read a graph from Matrix Market symmetric coordinate pattern format.
///
/// Validates the banner line for "coordinate pattern symmetric". Converts
/// 1-based indices to 0-based. Returns errors on invalid format, count
/// mismatches, self-loops, and out-of-range vertices.
pub fn read_matrix_market(r: impl BufRead) -> io::Result<SimpleGraph> {
    let mut lines = r.lines();

    // Parse and validate banner
    let banner = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty input"))??;
    let banner_lower = banner.trim().to_lowercase();
    if !banner_lower.starts_with("%%matrixmarket") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing %%MatrixMarket banner",
        ));
    }
    if !banner_lower.contains("symmetric") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only symmetric Matrix Market format is supported",
        ));
    }
    if !banner_lower.contains("coordinate") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only coordinate Matrix Market format is supported",
        ));
    }

    // Parse size line (skip comment lines starting with %)
    let (nv, nnz_declared) = loop {
        let line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing size line"))??;
        let trimmed = line.trim();
        if trimmed.starts_with('%') || trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let nrow: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nrow"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let ncol: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing ncol"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let nnz: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nnz"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if nrow != ncol {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-square matrix for undirected graph",
            ));
        }
        break (nrow, nnz);
    };

    // Parse entries
    let mut edges = Vec::new();
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let row: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing row"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let col: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing col"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if row == 0 || col == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Matrix Market uses 1-based indices",
            ));
        }
        edges.push((row - 1, col - 1));
    }

    if edges.len() != nnz_declared {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("declared {} entries but found {}", nnz_declared, edges.len()),
        ));
    }

    SimpleGraph::try_from_edges(nv, &edges)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

In `src/io/mod.rs`, add:

```rust
mod matrix_market;

pub use matrix_market::{read_matrix_market, write_matrix_market};
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/io/matrix_market.rs src/io/mod.rs
git commit -m "feat: add Matrix Market I/O with banner validation"
```

---

### Task 7: Basic Graph Generators

**Files:**
- Create: `src/gen/mod.rs`
- Create: `src/gen/basic.rs`
- Modify: `src/lib.rs`
- Test: `src/gen/basic.rs` (inline tests)

- [ ] **Step 1: Write the failing tests**

In `src/gen/basic.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo::is_connected;

    #[test]
    fn test_complete() {
        let g = complete(5);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 10);
        assert!(g.has_edge(0, 4));
        assert!(g.has_edge(2, 3));
    }

    #[test]
    fn test_complete_0_and_1() {
        assert_eq!(complete(0).ne(), 0);
        assert_eq!(complete(0).nv(), 0);
        assert_eq!(complete(1).ne(), 0);
        assert_eq!(complete(1).nv(), 1);
    }

    #[test]
    fn test_cycle() {
        let g = cycle(5);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 5);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(4, 0));
        assert!(!g.has_edge(0, 2));
    }

    #[test]
    fn test_cycle_3() {
        let g = cycle(3);
        assert_eq!(g.ne(), 3);
        assert!(is_connected(&g));
    }

    #[test]
    #[should_panic]
    fn test_cycle_2_panics() {
        cycle(2);
    }

    #[test]
    #[should_panic]
    fn test_cycle_0_panics() {
        cycle(0);
    }

    #[test]
    fn test_path() {
        let g = path(5);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 4);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(3, 4));
        assert!(!g.has_edge(0, 4));
        assert!(is_connected(&g));
    }

    #[test]
    fn test_path_0_and_1() {
        assert_eq!(path(0).nv(), 0);
        assert_eq!(path(1).nv(), 1);
        assert_eq!(path(1).ne(), 0);
    }

    #[test]
    fn test_grid_2d() {
        let g = grid_2d(3, 4);
        assert_eq!(g.nv(), 12);
        // horizontal: 3 * 3 = 9, vertical: 2 * 4 = 8, total = 17
        assert_eq!(g.ne(), 17);
        assert!(is_connected(&g));
    }

    #[test]
    fn test_grid_2d_1x1() {
        let g = grid_2d(1, 1);
        assert_eq!(g.nv(), 1);
        assert_eq!(g.ne(), 0);
    }

    #[test]
    fn test_grid_2d_0() {
        assert_eq!(grid_2d(0, 5).nv(), 0);
        assert_eq!(grid_2d(5, 0).nv(), 0);
        assert_eq!(grid_2d(0, 0).nv(), 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test gen::basic -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement generators**

In `src/gen/basic.rs`:

```rust
use crate::SimpleGraph;

/// Complete graph K_n.
pub fn complete(n: usize) -> SimpleGraph {
    let mut edges = Vec::with_capacity(n * n.saturating_sub(1) / 2);
    for u in 0..n as u32 {
        for v in (u + 1)..n as u32 {
            edges.push((u, v));
        }
    }
    SimpleGraph::from_edges(n, &edges)
}

/// Cycle graph C_n (n >= 3).
///
/// # Panics
/// Panics if `n < 3`.
pub fn cycle(n: usize) -> SimpleGraph {
    assert!(n >= 3, "cycle requires at least 3 vertices");
    let mut edges = Vec::with_capacity(n);
    for i in 0..n as u32 {
        edges.push((i, (i + 1) % n as u32));
    }
    SimpleGraph::from_edges(n, &edges)
}

/// Path graph P_n (n vertices, n-1 edges).
pub fn path(n: usize) -> SimpleGraph {
    if n <= 1 {
        return SimpleGraph::new(n);
    }
    let edges: Vec<(u32, u32)> = (0..n as u32 - 1).map(|i| (i, i + 1)).collect();
    SimpleGraph::from_edges(n, &edges)
}

/// 2D grid graph with `rows` x `cols` vertices.
///
/// Vertex index for position (r, c) is `r * cols + c`.
pub fn grid_2d(rows: usize, cols: usize) -> SimpleGraph {
    let n = rows * cols;
    if n == 0 {
        return SimpleGraph::new(0);
    }
    let mut edges = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = (r * cols + c) as u32;
            if c + 1 < cols {
                edges.push((v, v + 1));
            }
            if r + 1 < rows {
                edges.push((v, v + cols as u32));
            }
        }
    }
    SimpleGraph::from_edges(n, &edges)
}
```

In `src/gen/mod.rs`:

```rust
mod basic;

pub use basic::{complete, cycle, grid_2d, path};
```

In `src/lib.rs`, add:

```rust
pub mod gen;
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/gen/mod.rs src/gen/basic.rs src/lib.rs
git commit -m "feat: add basic graph generators (complete, cycle, path, grid_2d)"
```

---

### Task 8: Erdos-Renyi Generator (optional rand feature)

**Files:**
- Create: `src/gen/random.rs`
- Modify: `src/gen/mod.rs`
- Modify: `Cargo.toml`
- Test: `src/gen/random.rs` (inline tests)

- [ ] **Step 1: Write the failing tests**

In `src/gen/random.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn test_erdos_renyi_empty() {
        let mut rng = SmallRng::seed_from_u64(42);
        let g = erdos_renyi(10, 0.0, &mut rng);
        assert_eq!(g.nv(), 10);
        assert_eq!(g.ne(), 0);
    }

    #[test]
    fn test_erdos_renyi_complete() {
        let mut rng = SmallRng::seed_from_u64(42);
        let g = erdos_renyi(5, 1.0, &mut rng);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 10);
    }

    #[test]
    fn test_erdos_renyi_reproducible() {
        let g1 = erdos_renyi(100, 0.1, &mut SmallRng::seed_from_u64(123));
        let g2 = erdos_renyi(100, 0.1, &mut SmallRng::seed_from_u64(123));
        assert_eq!(g1, g2);
    }

    #[test]
    fn test_erdos_renyi_reasonable_density() {
        let mut rng = SmallRng::seed_from_u64(99);
        let g = erdos_renyi(200, 0.05, &mut rng);
        assert!(g.ne() > 500);
        assert!(g.ne() < 1500);
    }
}
```

- [ ] **Step 2: Update Cargo.toml with both optional features**

```toml
[package]
name = "simple-graph"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Lightweight undirected graph with sorted adjacency lists, modeled on Graphs.jl"

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
rand = { version = "0.9", optional = true }

[dev-dependencies]
serde_json = "1"
serde = { version = "1", features = ["derive"] }

[features]
default = ["serde"]
serde = ["dep:serde"]
rand = ["dep:rand"]
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --features rand gen::random -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 4: Implement erdos_renyi**

In `src/gen/random.rs`:

```rust
use rand::Rng;

use crate::SimpleGraph;

/// Erdos-Renyi random graph G(n, p).
///
/// Each possible edge is included independently with probability `p`.
/// Pass an `Rng` for reproducibility.
pub fn erdos_renyi(n: usize, p: f64, rng: &mut impl Rng) -> SimpleGraph {
    let mut edges = Vec::new();
    for u in 0..n as u32 {
        for v in (u + 1)..n as u32 {
            if rng.random::<f64>() < p {
                edges.push((u, v));
            }
        }
    }
    SimpleGraph::from_edges(n, &edges)
}
```

In `src/gen/mod.rs`, add:

```rust
#[cfg(feature = "rand")]
mod random;

#[cfg(feature = "rand")]
pub use random::erdos_renyi;
```

- [ ] **Step 5: Run all tests**

Run: `cargo test --features rand`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/gen/random.rs src/gen/mod.rs Cargo.toml
git commit -m "feat: add erdos_renyi generator behind rand feature"
```

---

### Task 9: Make serde Optional

**Files:**
- Modify: `src/simple_graph.rs`
- Test: existing serde test gated behind feature

Note: Cargo.toml was already updated in Task 8 to make serde optional with `default = ["serde"]`.

- [ ] **Step 1: Gate serde code in simple_graph.rs**

Change the struct derives:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SimpleGraph {
    ne: usize,
    fadjlist: Vec<Vec<u32>>,
}
```

Move the serde imports and Deserialize impl into a gated module:

```rust
#[cfg(feature = "serde")]
mod serde_impl {
    use super::SimpleGraph;
    use serde::de::{self, Deserializer};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct SimpleGraphRaw {
        ne: usize,
        fadjlist: Vec<Vec<u32>>,
    }

    impl<'de> Deserialize<'de> for SimpleGraph {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let raw = SimpleGraphRaw::deserialize(deserializer)?;
            let n = raw.fadjlist.len();
            let mut edge_count = 0usize;
            for (u, nbrs) in raw.fadjlist.iter().enumerate() {
                for (i, &v) in nbrs.iter().enumerate() {
                    if v as usize >= n {
                        return Err(de::Error::custom("neighbor out of range"));
                    }
                    if v as usize == u {
                        return Err(de::Error::custom("self-loop detected"));
                    }
                    if i > 0 && nbrs[i - 1] >= v {
                        return Err(de::Error::custom(
                            "adjacency list not sorted or has duplicates",
                        ));
                    }
                    if raw.fadjlist[v as usize].binary_search(&(u as u32)).is_err() {
                        return Err(de::Error::custom("asymmetric edge"));
                    }
                }
                edge_count += nbrs.len();
            }
            if edge_count / 2 != raw.ne {
                return Err(de::Error::custom("incorrect edge count"));
            }
            Ok(SimpleGraph {
                ne: raw.ne,
                fadjlist: raw.fadjlist,
            })
        }
    }
}
```

Remove the top-level `use serde::...` imports that were there before.

- [ ] **Step 2: Gate the serde test**

```rust
#[cfg(feature = "serde")]
#[test]
fn test_serde_roundtrip() {
    let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    let json = serde_json::to_string(&g).unwrap();
    let g2: SimpleGraph = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}
```

- [ ] **Step 3: Run tests with serde (default)**

Run: `cargo test`
Expected: All tests pass including serde roundtrip.

- [ ] **Step 4: Run tests without serde**

Run: `cargo test --no-default-features`
Expected: All tests pass except serde test (gated out). No compile errors.

- [ ] **Step 5: Commit**

```bash
git add src/simple_graph.rs
git commit -m "refactor: make serde an optional feature (enabled by default)"
```
