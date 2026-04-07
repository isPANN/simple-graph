use crate::Graph;

/// An undirected simple graph with sorted adjacency lists.
///
/// Vertices are contiguous integers `0..nv()`. Each vertex stores a sorted
/// `Vec<u32>` of its neighbors. This mirrors Julia's `Graphs.jl` `SimpleGraph`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SimpleGraph {
    ne: usize,
    fadjlist: Vec<Vec<u32>>,
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::SimpleGraph;
    use serde::de::{self, Deserializer};
    use serde::Deserialize;

    /// Raw helper for deserialization — validated before becoming a `SimpleGraph`.
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
                // Check sorted, no duplicates, no self-loops, in range
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
                    // Check symmetry: u must appear in v's list
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

impl SimpleGraph {
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
    pub fn new(n: usize) -> Self {
        assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
        Self {
            ne: 0,
            fadjlist: vec![vec![]; n],
        }
    }

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
    pub fn from_edges(n: usize, edges: &[(u32, u32)]) -> Self {
        assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
        let mut fadjlist: Vec<Vec<u32>> = vec![vec![]; n];
        for &(u, v) in edges {
            assert_ne!(u, v, "self-loops not allowed");
            assert!((u as usize) < n && (v as usize) < n, "vertex out of range");
            fadjlist[u as usize].push(v);
            fadjlist[v as usize].push(u);
        }
        let mut ne = 0;
        for list in &mut fadjlist {
            list.sort_unstable();
            list.dedup();
            ne += list.len();
        }
        Self {
            ne: ne / 2,
            fadjlist,
        }
    }

    /// Number of vertices.
    #[inline]
    pub fn nv(&self) -> usize {
        self.fadjlist.len()
    }

    /// Number of edges.
    #[inline]
    pub fn ne(&self) -> usize {
        self.ne
    }

    /// Whether vertex `v` exists in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::new(3);
    /// assert!(g.has_vertex(2));
    /// assert!(!g.has_vertex(3));
    /// ```
    #[inline]
    pub fn has_vertex(&self, v: u32) -> bool {
        (v as usize) < self.fadjlist.len()
    }

    /// Whether edge `(u, v)` exists. Returns `false` if either vertex is out
    /// of range. Checks the shorter neighbor list for O(log min(d(u), d(v))).
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1)]);
    /// assert!(g.has_edge(0, 1));
    /// assert!(g.has_edge(1, 0)); // undirected
    /// assert!(!g.has_edge(0, 2));
    /// ```
    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        if !self.has_vertex(u) || !self.has_vertex(v) {
            return false;
        }
        let (u, v) = if self.fadjlist[u as usize].len() <= self.fadjlist[v as usize].len() {
            (u, v)
        } else {
            (v, u)
        };
        self.fadjlist[u as usize].binary_search(&v).is_ok()
    }

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
    pub fn add_edge(&mut self, u: u32, v: u32) {
        assert_ne!(u, v, "self-loops not allowed");
        assert!(
            self.has_vertex(u) && self.has_vertex(v),
            "vertex out of range"
        );
        if let Err(pos) = self.fadjlist[u as usize].binary_search(&v) {
            self.fadjlist[u as usize].insert(pos, v);
            let pos2 = self.fadjlist[v as usize].binary_search(&u).unwrap_err();
            self.fadjlist[v as usize].insert(pos2, u);
            self.ne += 1;
        }
    }

    /// Remove an undirected edge. No-op if the edge does not exist or either
    /// vertex is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let mut g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// g.rem_edge(0, 1);
    /// assert!(!g.has_edge(0, 1));
    /// assert_eq!(g.ne(), 1);
    /// ```
    pub fn rem_edge(&mut self, u: u32, v: u32) {
        if !self.has_vertex(u) || !self.has_vertex(v) {
            return;
        }
        if let Ok(pos) = self.fadjlist[u as usize].binary_search(&v) {
            self.fadjlist[u as usize].remove(pos);
            let pos2 = self.fadjlist[v as usize].binary_search(&u).unwrap();
            self.fadjlist[v as usize].remove(pos2);
            self.ne -= 1;
        }
    }

    /// Degree of vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1), (0, 2)]);
    /// assert_eq!(g.degree(0), 2);
    /// assert_eq!(g.degree(1), 1);
    /// ```
    #[inline]
    pub fn degree(&self, v: u32) -> usize {
        self.fadjlist[v as usize].len()
    }

    /// Sorted neighbor slice of vertex `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 3), (0, 1)]);
    /// assert_eq!(g.neighbors(0), &[1, 3]); // sorted
    /// ```
    #[inline]
    pub fn neighbors(&self, v: u32) -> &[u32] {
        &self.fadjlist[v as usize]
    }

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
    pub fn edges(&self) -> crate::iter::Edges<'_, Self> {
        crate::iter::edges(self)
    }

    /// Add a new isolated vertex and return its index.
    ///
    /// # Panics
    /// Panics if the graph already has `u32::MAX` vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let mut g = SimpleGraph::new(2);
    /// let v = g.add_vertex();
    /// assert_eq!(v, 2);
    /// assert_eq!(g.nv(), 3);
    /// ```
    pub fn add_vertex(&mut self) -> u32 {
        assert!(
            self.fadjlist.len() < u32::MAX as usize,
            "vertex count would exceed u32::MAX"
        );
        let v = self.fadjlist.len() as u32;
        self.fadjlist.push(vec![]);
        v
    }

    /// Remove vertices and compact indices.
    ///
    /// Duplicate entries in `to_remove` are silently collapsed. Out-of-range
    /// entries are ignored. Returns `(new_graph, vmap)` where
    /// `vmap[new_idx] = old_idx`. Kept vertices appear in ascending order of
    /// their original index.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
    /// let (g2, vmap) = g.rem_vertices(&[1]);
    /// assert_eq!(g2.nv(), 3);
    /// assert_eq!(vmap, vec![0, 2, 3]);
    /// ```
    pub fn rem_vertices(&self, to_remove: &[u32]) -> (Self, Vec<u32>) {
        let n = self.nv();
        let mut remove_set = vec![false; n];
        for &v in to_remove {
            if (v as usize) < n {
                remove_set[v as usize] = true;
            }
        }
        self.build_subgraph(|v| !remove_set[v])
    }

    /// Return the subgraph induced by the given vertex set.
    ///
    /// Duplicate entries in `vertices` are silently collapsed. Out-of-range
    /// entries are ignored. Returns `(subgraph, vmap)` where
    /// `vmap[new_idx] = old_idx`. Kept vertices appear in ascending order of
    /// their original index regardless of input order.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::from_edges(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    /// let (sub, vmap) = g.induced_subgraph(&[1, 2, 3]);
    /// assert_eq!(sub.nv(), 3);
    /// assert_eq!(sub.ne(), 2);
    /// assert_eq!(vmap, vec![1, 2, 3]);
    /// ```
    pub fn induced_subgraph(&self, vertices: &[u32]) -> (Self, Vec<u32>) {
        let n = self.nv();
        let mut keep = vec![false; n];
        for &v in vertices {
            if (v as usize) < n {
                keep[v as usize] = true;
            }
        }
        self.build_subgraph(|v| keep[v])
    }

    /// Shared helper: build a compacted subgraph from a vertex predicate.
    fn build_subgraph(&self, keep: impl Fn(usize) -> bool) -> (Self, Vec<u32>) {
        let n = self.nv();
        let mut old_to_new = vec![u32::MAX; n];
        let mut vmap: Vec<u32> = Vec::new();
        let mut new_idx = 0u32;
        for (old, slot) in old_to_new.iter_mut().enumerate() {
            if keep(old) {
                *slot = new_idx;
                vmap.push(old as u32);
                new_idx += 1;
            }
        }
        let mut fadjlist = Vec::with_capacity(new_idx as usize);
        let mut ne = 0usize;
        for &old_v in &vmap {
            let new_nbrs: Vec<u32> = self.fadjlist[old_v as usize]
                .iter()
                .filter_map(|&nbr| {
                    let new = old_to_new[nbr as usize];
                    if new != u32::MAX {
                        Some(new)
                    } else {
                        None
                    }
                })
                .collect();
            ne += new_nbrs.len();
            fadjlist.push(new_nbrs);
        }
        (
            SimpleGraph {
                ne: ne / 2,
                fadjlist,
            },
            vmap,
        )
    }
}

impl SimpleGraph {
    pub(crate) fn from_csr(offsets: &[usize], targets: &[u32], ne: usize) -> Self {
        let n = if offsets.is_empty() {
            0
        } else {
            offsets.len() - 1
        };
        let mut fadjlist = Vec::with_capacity(n);
        for v in 0..n {
            fadjlist.push(targets[offsets[v]..offsets[v + 1]].to_vec());
        }
        Self { ne, fadjlist }
    }
}

impl SimpleGraph {
    /// Fallible version of `from_edges`. Returns an error string on self-loops
    /// or out-of-range vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::SimpleGraph;
    ///
    /// let g = SimpleGraph::try_from_edges(3, &[(0, 1), (1, 2)]).unwrap();
    /// assert_eq!(g.ne(), 2);
    ///
    /// // Self-loops are rejected
    /// assert!(SimpleGraph::try_from_edges(3, &[(0, 0)]).is_err());
    /// ```
    pub fn try_from_edges(n: usize, edges: &[(u32, u32)]) -> Result<Self, String> {
        if n > u32::MAX as usize {
            return Err(format!("vertex count {} exceeds u32::MAX", n));
        }
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

impl Graph for SimpleGraph {
    #[inline]
    fn nv(&self) -> usize {
        SimpleGraph::nv(self)
    }
    #[inline]
    fn ne(&self) -> usize {
        SimpleGraph::ne(self)
    }
    #[inline]
    fn has_vertex(&self, v: u32) -> bool {
        SimpleGraph::has_vertex(self, v)
    }
    fn has_edge(&self, u: u32, v: u32) -> bool {
        SimpleGraph::has_edge(self, u, v)
    }
    #[inline]
    fn degree(&self, v: u32) -> usize {
        SimpleGraph::degree(self, v)
    }
    #[inline]
    fn neighbors(&self, v: u32) -> &[u32] {
        SimpleGraph::neighbors(self, v)
    }
}

impl Default for SimpleGraph {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<'a> IntoIterator for &'a SimpleGraph {
    type Item = (u32, u32);
    type IntoIter = crate::iter::Edges<'a, SimpleGraph>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = SimpleGraph::new(0);
        assert_eq!(g.nv(), 0);
        assert_eq!(g.ne(), 0);
    }

    #[test]
    fn test_add_edge_and_has_edge() {
        let mut g = SimpleGraph::new(4);
        assert!(!g.has_edge(0, 1));
        g.add_edge(0, 1);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 0));
        assert_eq!(g.ne(), 1);
        g.add_edge(0, 1);
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_neighbors_sorted() {
        let mut g = SimpleGraph::new(5);
        g.add_edge(0, 3);
        g.add_edge(0, 1);
        g.add_edge(0, 4);
        assert_eq!(g.neighbors(0), &[1, 3, 4]);
    }

    #[test]
    fn test_degree() {
        let mut g = SimpleGraph::new(3);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        assert_eq!(g.degree(0), 2);
        assert_eq!(g.degree(1), 1);
    }

    #[test]
    fn test_has_vertex() {
        let g = SimpleGraph::new(3);
        assert!(g.has_vertex(0));
        assert!(g.has_vertex(2));
        assert!(!g.has_vertex(3));
    }

    #[test]
    fn test_from_edges() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(g.nv(), 4);
        assert_eq!(g.ne(), 4);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(3, 0));
        assert!(!g.has_edge(0, 2));
    }

    #[test]
    fn test_edges_iterator() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let edges: Vec<(u32, u32)> = g.edges().collect();
        assert_eq!(edges, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn test_rem_edge() {
        let mut g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        assert_eq!(g.ne(), 2);
        g.rem_edge(0, 1);
        assert!(!g.has_edge(0, 1));
        assert_eq!(g.ne(), 1);
        g.rem_edge(0, 1);
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_add_vertex() {
        let mut g = SimpleGraph::new(2);
        assert_eq!(g.nv(), 2);
        g.add_vertex();
        assert_eq!(g.nv(), 3);
        g.add_edge(0, 2);
        assert!(g.has_edge(0, 2));
    }

    #[test]
    fn test_rem_vertices_basic() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let (g2, vmap) = g.rem_vertices(&[1, 3]);
        assert_eq!(g2.nv(), 2);
        assert_eq!(g2.ne(), 0);
        assert_eq!(vmap, vec![0, 2]);
    }

    #[test]
    fn test_rem_vertices_preserves_edges() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
        let (g2, vmap) = g.rem_vertices(&[1]);
        assert_eq!(g2.nv(), 2);
        assert_eq!(g2.ne(), 1);
        assert!(g2.has_edge(0, 1));
        assert_eq!(vmap, vec![0, 2]);
    }

    #[test]
    fn test_rem_vertices_empty_removal() {
        let g = SimpleGraph::from_edges(3, &[(0, 1)]);
        let (g2, vmap) = g.rem_vertices(&[]);
        assert_eq!(g2.nv(), 3);
        assert_eq!(g2.ne(), 1);
        assert_eq!(vmap, vec![0, 1, 2]);
    }

    #[test]
    fn test_rem_vertices_all() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
        let (g2, vmap) = g.rem_vertices(&[0, 1, 2]);
        assert_eq!(g2.nv(), 0);
        assert_eq!(g2.ne(), 0);
        assert!(vmap.is_empty());
    }

    #[test]
    fn test_induced_subgraph() {
        let g = SimpleGraph::from_edges(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (0, 4)]);
        let (sub, vmap) = g.induced_subgraph(&[1, 2, 3]);
        assert_eq!(sub.nv(), 3);
        assert_eq!(sub.ne(), 2);
        assert!(sub.has_edge(0, 1));
        assert!(sub.has_edge(1, 2));
        assert!(!sub.has_edge(0, 2));
        assert_eq!(vmap, vec![1, 2, 3]);
    }

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

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let json = serde_json::to_string(&g).unwrap();
        let g2: SimpleGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }
}
