use crate::graph::Graph;
use std::collections::VecDeque;

/// BFS iterator from a source vertex.
pub struct Bfs<'a, G: Graph + ?Sized> {
    graph: &'a G,
    queue: VecDeque<u32>,
    visited: Vec<bool>,
}

/// Return a BFS iterator starting from `source`.
///
/// If `source` is out of range, the iterator yields no elements.
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
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, algo};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
/// assert!(algo::is_connected(&g));
/// ```
pub fn is_connected<G: Graph>(graph: &G) -> bool {
    let n = graph.nv();
    if n <= 1 {
        return true;
    }
    bfs(graph, 0).count() == n
}

/// Assign a component label to each vertex. Labels are `0, 1, 2, ...` assigned
/// in order of discovery. Returns a `Vec<u32>` of length `nv()`.
///
/// Uses a single shared visited array and queue to avoid per-component
/// allocation overhead.
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
pub fn connected_components<G: Graph>(graph: &G) -> Vec<u32> {
    let n = graph.nv();
    let mut labels = vec![u32::MAX; n];
    let mut queue = VecDeque::new();
    let mut component = 0u32;
    for start in 0..n as u32 {
        if labels[start as usize] != u32::MAX {
            continue;
        }
        labels[start as usize] = component;
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            for &v in graph.neighbors(u) {
                if labels[v as usize] == u32::MAX {
                    labels[v as usize] = component;
                    queue.push_back(v);
                }
            }
        }
        component += 1;
    }
    labels
}

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
