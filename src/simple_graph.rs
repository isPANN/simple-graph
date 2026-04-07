use serde::{Deserialize, Serialize};

use crate::Edges;

/// An undirected simple graph with sorted adjacency lists.
///
/// Vertices are contiguous integers `0..nv()`. Each vertex stores a sorted
/// `Vec<u32>` of its neighbors. This mirrors Julia's `Graphs.jl` `SimpleGraph`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleGraph {
    ne: usize,
    fadjlist: Vec<Vec<u32>>,
}

impl SimpleGraph {
    /// Create an empty graph with `n` vertices and no edges.
    pub fn new(n: usize) -> Self {
        Self {
            ne: 0,
            fadjlist: vec![vec![]; n],
        }
    }

    /// Create a graph from a list of edges.
    pub fn from_edges(n: usize, edges: &[(u32, u32)]) -> Self {
        let mut g = Self::new(n);
        for &(u, v) in edges {
            g.add_edge(u, v);
        }
        g
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
    #[inline]
    pub fn has_vertex(&self, v: u32) -> bool {
        (v as usize) < self.fadjlist.len()
    }

    /// Whether edge `(u, v)` exists. Checks the shorter neighbor list for
    /// O(log min(d(u), d(v))).
    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        let (u, v) = if self.degree(u) <= self.degree(v) {
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

    /// Remove an undirected edge. No-op if the edge does not exist.
    pub fn rem_edge(&mut self, u: u32, v: u32) {
        if let Ok(pos) = self.fadjlist[u as usize].binary_search(&v) {
            self.fadjlist[u as usize].remove(pos);
            let pos2 = self.fadjlist[v as usize].binary_search(&u).unwrap();
            self.fadjlist[v as usize].remove(pos2);
            self.ne -= 1;
        }
    }

    /// Degree of vertex `v`.
    #[inline]
    pub fn degree(&self, v: u32) -> usize {
        self.fadjlist[v as usize].len()
    }

    /// Sorted neighbor slice of vertex `v`.
    #[inline]
    pub fn neighbors(&self, v: u32) -> &[u32] {
        &self.fadjlist[v as usize]
    }

    /// Iterator over all edges `(u, v)` with `u < v`.
    pub fn edges(&self) -> Edges<'_> {
        Edges::new(self)
    }

    /// Add a new isolated vertex and return its index.
    pub fn add_vertex(&mut self) -> u32 {
        let v = self.fadjlist.len() as u32;
        self.fadjlist.push(vec![]);
        v
    }

    /// Remove vertices and compact indices.
    ///
    /// Returns `(new_graph, vmap)` where `vmap[new_idx] = old_idx`.
    pub fn rem_vertices(&self, to_remove: &[u32]) -> (Self, Vec<u32>) {
        let n = self.nv();
        let mut old_to_new = vec![u32::MAX; n];
        let mut remove_set = vec![false; n];
        for &v in to_remove {
            remove_set[v as usize] = true;
        }
        let mut vmap: Vec<u32> = Vec::with_capacity(n - to_remove.len());
        let mut new_idx = 0u32;
        for old in 0..n {
            if !remove_set[old] {
                old_to_new[old] = new_idx;
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

    /// Return the subgraph induced by the given vertex set.
    ///
    /// Returns `(subgraph, vmap)` where `vmap[new_idx] = old_idx`.
    pub fn induced_subgraph(&self, vertices: &[u32]) -> (Self, Vec<u32>) {
        let n = self.nv();
        let mut keep = vec![false; n];
        for &v in vertices {
            keep[v as usize] = true;
        }
        let to_remove: Vec<u32> = (0..n as u32).filter(|&v| !keep[v as usize]).collect();
        self.rem_vertices(&to_remove)
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
    fn test_serde_roundtrip() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let json = serde_json::to_string(&g).unwrap();
        let g2: SimpleGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(g, g2);
    }
}
