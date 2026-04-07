use std::iter::FusedIterator;

use crate::graph::Graph;

/// Iterator over edges of a graph implementing [`Graph`], yielding `(u, v)` with `u < v`.
pub struct Edges<'a, G: Graph + ?Sized> {
    graph: &'a G,
    u: u32,
    nv: u32,
    nbrs: &'a [u32],
    idx: usize,
}

/// Create an iterator over all edges `(u, v)` with `u < v` for any [`Graph`].
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, edges};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
/// let e: Vec<_> = edges(&g).collect();
/// assert_eq!(e, vec![(0, 1), (1, 2)]);
/// ```
pub fn edges<G: Graph>(graph: &G) -> Edges<'_, G> {
    let nv = graph.nv() as u32;
    let mut iter = Edges {
        graph,
        u: 0,
        nv,
        nbrs: &[],
        idx: 0,
    };
    iter.advance_vertex();
    iter
}

impl<'a, G: Graph + ?Sized> Edges<'a, G> {
    /// Load the neighbor slice for the current vertex and skip neighbors ≤ `self.u`.
    fn advance_vertex(&mut self) {
        if self.u < self.nv {
            self.nbrs = self.graph.neighbors(self.u);
            self.idx = self.nbrs.partition_point(|&v| v <= self.u);
        }
    }
}

impl<'a, G: Graph + ?Sized> Iterator for Edges<'a, G> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        while self.u < self.nv {
            if self.idx < self.nbrs.len() {
                let v = self.nbrs[self.idx];
                self.idx += 1;
                return Some((self.u, v));
            }
            self.u += 1;
            self.advance_vertex();
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.graph.ne()))
    }
}

impl<'a, G: Graph + ?Sized> FusedIterator for Edges<'a, G> {}

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
