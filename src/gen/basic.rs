use crate::{CsrGraph, SimpleGraph};

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
    assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
    // Direct adjacency list construction: vertex v's neighbors are [0..v, v+1..n].
    // Two branchless extends instead of a per-element branch.
    let ne = n * n.saturating_sub(1) / 2;
    let mut fadjlist = Vec::with_capacity(n);
    for v in 0..n as u32 {
        let mut nbrs = Vec::with_capacity(n - 1);
        nbrs.extend(0..v);
        nbrs.extend(v + 1..n as u32);
        fadjlist.push(nbrs);
    }
    SimpleGraph::from_raw(ne, fadjlist)
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
    assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
    if n <= 1 {
        return SimpleGraph::new(n);
    }
    let edges: Vec<(u32, u32)> = (0..n as u32 - 1).map(|i| (i, i + 1)).collect();
    SimpleGraph::from_sorted_unique_edges(n, &edges)
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
    assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
    if n == 0 {
        return SimpleGraph::new(0);
    }
    // Build adjacency lists directly — avoids intermediate edge Vec and extra passes.
    let ne = rows * (cols - 1) + (rows - 1) * cols;
    let cols_u32 = cols as u32;
    let mut fadjlist = Vec::with_capacity(n);
    for r in 0..rows {
        for c in 0..cols {
            let v = (r * cols + c) as u32;
            let deg = (if r > 0 { 1 } else { 0 })
                + (if c > 0 { 1 } else { 0 })
                + (if c + 1 < cols { 1 } else { 0 })
                + (if r + 1 < rows { 1 } else { 0 });
            let mut nbrs = Vec::with_capacity(deg);
            // Push in sorted order: up < left < right < down
            if r > 0 {
                nbrs.push(v - cols_u32);
            }
            if c > 0 {
                nbrs.push(v - 1);
            }
            if c + 1 < cols {
                nbrs.push(v + 1);
            }
            if r + 1 < rows {
                nbrs.push(v + cols_u32);
            }
            fadjlist.push(nbrs);
        }
    }
    SimpleGraph::from_raw(ne, fadjlist)
}

/// Complete graph K_n as a [`CsrGraph`].
///
/// Builds the CSR structure directly — no intermediate edge list.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::complete_csr(4);
/// assert_eq!(g.ne(), 6);
/// ```
pub fn complete_csr(n: usize) -> CsrGraph {
    assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
    let ne = n * n.saturating_sub(1) / 2;
    let mut offsets = Vec::with_capacity(n + 1);
    let mut targets = Vec::with_capacity(ne * 2);
    for v in 0..n as u32 {
        offsets.push(targets.len());
        targets.extend(0..v);
        targets.extend(v + 1..n as u32);
    }
    offsets.push(targets.len());
    CsrGraph::from_raw_parts(n, ne, offsets, targets)
}

/// 2D grid graph with `rows` x `cols` vertices as a [`CsrGraph`].
///
/// Builds the CSR structure directly in a single pass — no intermediate
/// edge list, degree counting, or prefix sum.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
///
/// let g = gen::grid_2d_csr(3, 4);
/// assert_eq!(g.nv(), 12);
/// ```
pub fn grid_2d_csr(rows: usize, cols: usize) -> CsrGraph {
    let n = rows * cols;
    assert!(n <= u32::MAX as usize, "vertex count exceeds u32::MAX");
    if n == 0 {
        return CsrGraph::from_raw_parts(0, 0, vec![0], Vec::new());
    }
    let ne = rows * (cols - 1) + (rows - 1) * cols;
    let cols_u32 = cols as u32;
    let mut offsets = Vec::with_capacity(n + 1);
    let mut targets = Vec::with_capacity(ne * 2);
    for r in 0..rows {
        for c in 0..cols {
            offsets.push(targets.len());
            let v = (r * cols + c) as u32;
            if r > 0 {
                targets.push(v - cols_u32);
            }
            if c > 0 {
                targets.push(v - 1);
            }
            if c + 1 < cols {
                targets.push(v + 1);
            }
            if r + 1 < rows {
                targets.push(v + cols_u32);
            }
        }
    }
    offsets.push(targets.len());
    CsrGraph::from_raw_parts(n, ne, offsets, targets)
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

    #[test]
    fn test_complete_csr_matches_simple() {
        use crate::Graph;
        let sg = complete(5);
        let csr = complete_csr(5);
        assert_eq!(csr.nv(), sg.nv());
        assert_eq!(csr.ne(), sg.ne());
        for u in 0..5u32 {
            assert_eq!(csr.neighbors(u), sg.neighbors(u));
        }
    }

    #[test]
    fn test_grid_2d_csr_matches_simple() {
        use crate::Graph;
        let sg = grid_2d(10, 10);
        let csr = grid_2d_csr(10, 10);
        assert_eq!(csr.nv(), sg.nv());
        assert_eq!(csr.ne(), sg.ne());
        for u in 0..100u32 {
            assert_eq!(csr.neighbors(u), sg.neighbors(u));
        }
    }

    #[test]
    fn test_grid_2d_csr_edge_cases() {
        assert_eq!(grid_2d_csr(0, 0).nv(), 0);
        assert_eq!(grid_2d_csr(1, 1).nv(), 1);
        assert_eq!(grid_2d_csr(1, 1).ne(), 0);
        let g = grid_2d_csr(1, 5);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 4);
        let g = grid_2d_csr(5, 1);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 4);
    }
}
