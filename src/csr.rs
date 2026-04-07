use crate::graph::Graph;
use crate::SimpleGraph;

/// Compressed Sparse Row graph — immutable, contiguous-memory storage.
///
/// Built from a [`SimpleGraph`] via `From`. All neighbor data lives in a single
/// `Vec<u32>`, indexed by a `Vec<usize>` of offsets. Zero pointer indirection
/// per vertex, ideal for read-heavy scientific workloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrGraph {
    nv: usize,
    ne: usize,
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<u32>,
}

impl CsrGraph {
    /// Number of vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let sg = SimpleGraph::from_edges(3, &[(0, 1)]);
    /// let csr = CsrGraph::from(&sg);
    /// assert_eq!(csr.nv(), 3);
    /// ```
    #[inline]
    pub fn nv(&self) -> usize {
        self.nv
    }

    /// Number of edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// let csr = CsrGraph::from(&sg);
    /// assert_eq!(csr.ne(), 2);
    /// ```
    #[inline]
    pub fn ne(&self) -> usize {
        self.ne
    }

    /// Whether vertex `v` exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let csr = CsrGraph::from(&SimpleGraph::new(3));
    /// assert!(csr.has_vertex(2));
    /// assert!(!csr.has_vertex(3));
    /// ```
    #[inline]
    pub fn has_vertex(&self, v: u32) -> bool {
        (v as usize) < self.nv()
    }

    /// Whether edge `(u, v)` exists. Returns `false` for out-of-range vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let csr = CsrGraph::from(&SimpleGraph::from_edges(3, &[(0, 1)]));
    /// assert!(csr.has_edge(0, 1));
    /// assert!(!csr.has_edge(0, 2));
    /// ```
    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        if !self.has_vertex(u) || !self.has_vertex(v) {
            return false;
        }
        self.neighbors(u).binary_search(&v).is_ok()
    }

    /// Degree of vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let csr = CsrGraph::from(&SimpleGraph::from_edges(3, &[(0, 1), (0, 2)]));
    /// assert_eq!(csr.degree(0), 2);
    /// ```
    #[inline]
    pub fn degree(&self, v: u32) -> usize {
        let vi = v as usize;
        self.offsets[vi + 1] - self.offsets[vi]
    }

    /// Sorted neighbor slice of vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let csr = CsrGraph::from(&SimpleGraph::from_edges(3, &[(0, 2), (0, 1)]));
    /// assert_eq!(csr.neighbors(0), &[1, 2]);
    /// ```
    #[inline]
    pub fn neighbors(&self, v: u32) -> &[u32] {
        let vi = v as usize;
        &self.targets[self.offsets[vi]..self.offsets[vi + 1]]
    }

    /// Internal constructor from pre-built CSR arrays.
    /// Caller must guarantee: targets sorted per vertex, symmetric, no self-loops.
    pub(crate) fn from_raw_parts(
        nv: usize,
        ne: usize,
        offsets: Vec<usize>,
        targets: Vec<u32>,
    ) -> Self {
        Self {
            nv,
            ne,
            offsets,
            targets,
        }
    }

    /// Iterator over all edges `(u, v)` with `u < v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let csr = CsrGraph::from(&SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]));
    /// let edges: Vec<_> = csr.edges().collect();
    /// assert_eq!(edges, vec![(0, 1), (1, 2)]);
    /// ```
    pub fn edges(&self) -> crate::iter::Edges<'_, Self> {
        crate::iter::edges(self)
    }

    /// Convert back to a mutable [`SimpleGraph`].
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::{SimpleGraph, CsrGraph};
    ///
    /// let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// let csr = CsrGraph::from(&sg);
    /// assert_eq!(csr.to_simple_graph(), sg);
    /// ```
    pub fn to_simple_graph(&self) -> SimpleGraph {
        SimpleGraph::from_csr(&self.offsets, &self.targets, self.ne)
    }
}

impl CsrGraph {
    /// Build a CsrGraph directly from edge pairs, bypassing SimpleGraph.
    ///
    /// Edges must be canonical (`u < v`), sorted, and unique. This performs
    /// only 3 heap allocations regardless of graph size.
    ///
    /// # Panics
    /// Panics if edges contain self-loops or out-of-range vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use easygraph::CsrGraph;
    ///
    /// let csr = CsrGraph::from_sorted_unique_edges(3, &[(0, 1), (1, 2)]);
    /// assert_eq!(csr.nv(), 3);
    /// assert_eq!(csr.ne(), 2);
    /// ```
    pub fn from_sorted_unique_edges(n: usize, edges: &[(u32, u32)]) -> Self {
        // Validate upfront — keep asserts out of the hot counting/filling loops.
        for &(u, v) in edges {
            assert_ne!(u, v, "self-loops not allowed");
            assert!((u as usize) < n && (v as usize) < n, "vertex out of range");
        }
        Self::from_sorted_unique_edges_unchecked(n, edges)
    }

    /// Internal builder — caller guarantees no self-loops and all vertices in range.
    fn from_sorted_unique_edges_unchecked(n: usize, edges: &[(u32, u32)]) -> Self {
        let mut deg = vec![0usize; n];
        for &(u, v) in edges {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
        }
        // Build offsets via prefix sum with running scalar (avoids iterator adaptor).
        let mut offsets = Vec::with_capacity(n + 1);
        let mut running = 0usize;
        offsets.push(0);
        for &d in &deg {
            running += d;
            offsets.push(running);
        }
        // Fill flat targets using write cursors (reuse deg allocation).
        let mut targets = vec![0u32; running];
        let mut cursor = deg;
        cursor.copy_from_slice(&offsets[..n]);
        for &(u, v) in edges {
            targets[cursor[u as usize]] = v;
            cursor[u as usize] += 1;
            targets[cursor[v as usize]] = u;
            cursor[v as usize] += 1;
        }
        CsrGraph {
            nv: n,
            ne: edges.len(),
            offsets,
            targets,
        }
    }
}

/// Incremental builder for [`CsrGraph`]. Collects edges one at a time,
/// then builds the CSR representation in a single pass.
///
/// # Examples
///
/// ```
/// use easygraph::CsrBuilder;
///
/// let mut builder = CsrBuilder::new(4);
/// builder.add_edge(0, 1);
/// builder.add_edge(1, 2);
/// builder.add_edge(2, 3);
/// let csr = builder.build();
/// assert_eq!(csr.nv(), 4);
/// assert_eq!(csr.ne(), 3);
/// assert!(csr.has_edge(0, 1));
/// ```
pub struct CsrBuilder {
    nv: usize,
    edges: Vec<(u32, u32)>,
    deg: Vec<usize>,
    sorted: bool,
}

impl CsrBuilder {
    /// Create a builder for a graph with `nv` vertices.
    pub fn new(nv: usize) -> Self {
        CsrBuilder {
            nv,
            edges: Vec::new(),
            deg: vec![0; nv],
            sorted: true,
        }
    }

    /// Create a builder with pre-allocated edge capacity.
    pub fn with_capacity(nv: usize, edge_capacity: usize) -> Self {
        CsrBuilder {
            nv,
            edges: Vec::with_capacity(edge_capacity),
            deg: vec![0; nv],
            sorted: true,
        }
    }

    /// Add an undirected edge. Duplicates are collapsed during `build()`.
    ///
    /// # Panics
    /// Panics on self-loops or out-of-range vertices.
    pub fn add_edge(&mut self, u: u32, v: u32) {
        assert_ne!(u, v, "self-loops not allowed");
        assert!(
            (u as usize) < self.nv && (v as usize) < self.nv,
            "vertex out of range"
        );
        let (u, v) = if u < v { (u, v) } else { (v, u) };
        if self.sorted {
            if let Some(&last) = self.edges.last() {
                if (u, v) < last {
                    self.sorted = false;
                }
            }
        }
        self.deg[u as usize] += 1;
        self.deg[v as usize] += 1;
        self.edges.push((u, v));
    }

    /// Build the CsrGraph. Sorts and deduplicates edges.
    pub fn build(mut self) -> CsrGraph {
        if !self.sorted {
            self.edges.sort_unstable();
        }
        let old_len = self.edges.len();
        self.edges.dedup();
        let deduped = self.edges.len() != old_len;

        if deduped {
            // Degrees are stale after dedup — recount.
            self.deg.fill(0);
            for &(u, v) in &self.edges {
                self.deg[u as usize] += 1;
                self.deg[v as usize] += 1;
            }
        }

        // Build offsets from pre-counted degrees (skip from_sorted_unique_edges).
        let n = self.nv;
        let ne = self.edges.len();
        let mut offsets = Vec::with_capacity(n + 1);
        let mut running = 0usize;
        offsets.push(0);
        for &d in &self.deg {
            running += d;
            offsets.push(running);
        }

        // Fill targets using write cursors (reuse deg allocation).
        let mut targets = vec![0u32; running];
        let mut cursor = self.deg;
        cursor.copy_from_slice(&offsets[..n]);
        for &(u, v) in &self.edges {
            targets[cursor[u as usize]] = v;
            cursor[u as usize] += 1;
            targets[cursor[v as usize]] = u;
            cursor[v as usize] += 1;
        }

        CsrGraph {
            nv: n,
            ne,
            offsets,
            targets,
        }
    }
}

impl From<&SimpleGraph> for CsrGraph {
    fn from(sg: &SimpleGraph) -> Self {
        let n = sg.nv();
        let mut offsets = Vec::with_capacity(n + 1);
        // 2*ne is the total number of directed edges — avoids an extra scan.
        let mut targets = Vec::with_capacity(sg.ne() * 2);
        let mut offset = 0;
        for v in 0..n {
            offsets.push(offset);
            let nbrs = sg.neighbors(v as u32);
            targets.extend_from_slice(nbrs);
            offset += nbrs.len();
        }
        offsets.push(offset);
        CsrGraph {
            nv: n,
            ne: sg.ne(),
            offsets,
            targets,
        }
    }
}

impl<'a> IntoIterator for &'a CsrGraph {
    type Item = (u32, u32);
    type IntoIter = crate::iter::Edges<'a, CsrGraph>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges()
    }
}

impl Graph for CsrGraph {
    #[inline]
    fn nv(&self) -> usize {
        CsrGraph::nv(self)
    }
    #[inline]
    fn ne(&self) -> usize {
        CsrGraph::ne(self)
    }
    #[inline]
    fn has_vertex(&self, v: u32) -> bool {
        CsrGraph::has_vertex(self, v)
    }
    fn has_edge(&self, u: u32, v: u32) -> bool {
        CsrGraph::has_edge(self, u, v)
    }
    #[inline]
    fn degree(&self, v: u32) -> usize {
        CsrGraph::degree(self, v)
    }
    #[inline]
    fn neighbors(&self, v: u32) -> &[u32] {
        CsrGraph::neighbors(self, v)
    }
}

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
        assert_eq!(csr.offsets.len(), 5);
        for i in 1..csr.offsets.len() {
            assert!(csr.offsets[i] >= csr.offsets[i - 1]);
        }
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
