use crate::graph::Graph;
use std::collections::VecDeque;

/// Compute unweighted shortest-path distances from `source` to all reachable vertices.
///
/// Returns a `Vec<Option<u32>>` of length `nv()`. `result[v]` is `Some(d)` if
/// vertex `v` is reachable from `source` in `d` hops, or `None` if unreachable.
/// `result[source]` is `Some(0)`. If `source` is out of range, all entries are `None`.
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
