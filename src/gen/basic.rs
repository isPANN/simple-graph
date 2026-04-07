use crate::SimpleGraph;

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
pub fn cycle(n: usize) -> SimpleGraph {
    assert!(n >= 3, "cycle requires at least 3 vertices");
    let mut edges = Vec::with_capacity(n);
    for i in 0..n as u32 {
        edges.push((i, (i + 1) % n as u32));
    }
    SimpleGraph::from_edges(n, &edges)
}

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
pub fn path(n: usize) -> SimpleGraph {
    if n <= 1 {
        return SimpleGraph::new(n);
    }
    let edges: Vec<(u32, u32)> = (0..n as u32 - 1).map(|i| (i, i + 1)).collect();
    SimpleGraph::from_edges(n, &edges)
}

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
