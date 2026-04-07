use crate::graph::Graph;

/// DFS iterator from a source vertex (iterative, using an explicit stack).
pub struct Dfs<'a, G: Graph + ?Sized> {
    graph: &'a G,
    stack: Vec<u32>,
    visited: Vec<bool>,
}

/// Return a DFS iterator starting from `source`.
///
/// If `source` is out of range, the iterator yields no elements.
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
