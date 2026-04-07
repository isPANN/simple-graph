use crate::graph::Graph;

const UNREACHABLE: u32 = u32::MAX;

/// Compute unweighted shortest-path distances from `source` to all reachable vertices.
///
/// Returns a `Vec<Option<u32>>` of length `nv()`. `result[v]` is `Some(d)` if
/// vertex `v` is reachable from `source` in `d` hops, or `None` if unreachable.
/// `result[source]` is `Some(0)`. If `source` is out of range, all entries are `None`.
///
/// # Examples
///
/// ```
/// use easygraph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
/// let dist = algo::shortest_path_lengths(&g, 0);
/// assert_eq!(dist, vec![Some(0), Some(1), Some(2), Some(3)]);
/// ```
pub fn shortest_path_lengths<G: Graph>(graph: &G, source: u32) -> Vec<Option<u32>> {
    let n = graph.nv();
    // Use u32 sentinel internally (4 bytes vs 8 for Option<u32>) for cache efficiency.
    let mut dist = vec![UNREACHABLE; n];
    if !graph.has_vertex(source) {
        return vec![None; n];
    }
    dist[source as usize] = 0;
    // Flat Vec + head pointer instead of VecDeque — BFS queues are monotonic.
    let mut queue = Vec::with_capacity(n);
    let mut head = 0usize;
    queue.push(source);
    while head < queue.len() {
        let u = queue[head];
        head += 1;
        let next_d = dist[u as usize] + 1;
        for &v in graph.neighbors(u) {
            if dist[v as usize] == UNREACHABLE {
                dist[v as usize] = next_d;
                queue.push(v);
            }
        }
    }
    dist.into_iter()
        .map(|d| if d == UNREACHABLE { None } else { Some(d) })
        .collect()
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
