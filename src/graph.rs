/// Read-only interface shared by all graph representations.
pub trait Graph {
    /// Number of vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::new(5);
    /// assert_eq!(g.nv(), 5);
    /// ```
    fn nv(&self) -> usize;
    /// Number of edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
    /// assert_eq!(g.ne(), 2);
    /// ```
    fn ne(&self) -> usize;
    /// Whether vertex `v` exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::new(3);
    /// assert!(g.has_vertex(0));
    /// assert!(!g.has_vertex(3));
    /// ```
    fn has_vertex(&self, v: u32) -> bool;
    /// Whether edge `(u, v)` exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1)]);
    /// assert!(g.has_edge(0, 1));
    /// assert!(!g.has_edge(0, 2));
    /// ```
    fn has_edge(&self, u: u32, v: u32) -> bool;
    /// Degree of vertex `v`.
    ///
    /// # Panics
    /// Panics if `v` is out of range. Use `has_vertex` to check first.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 1), (0, 2)]);
    /// assert_eq!(g.degree(0), 2);
    /// ```
    fn degree(&self, v: u32) -> usize;
    /// Sorted neighbor slice of vertex `v`.
    ///
    /// # Panics
    /// Panics if `v` is out of range. Use `has_vertex` to check first.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(3, &[(0, 2), (0, 1)]);
    /// assert_eq!(g.neighbors(0), &[1, 2]); // sorted
    /// ```
    fn neighbors(&self, v: u32) -> &[u32];

    /// Graph density: `ne / (nv choose 2)`. Returns 0.0 for graphs with < 2 vertices.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2), (1, 3)]);
    /// assert!((g.density() - 1.0).abs() < 1e-10);
    /// ```
    fn density(&self) -> f64 {
        let n = self.nv();
        if n < 2 {
            return 0.0;
        }
        let n = n as f64;
        self.ne() as f64 / (n * (n - 1.0) / 2.0)
    }

    /// Sorted degree sequence (ascending).
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
    /// assert_eq!(g.degree_sequence(), vec![1, 1, 1, 3]);
    /// ```
    fn degree_sequence(&self) -> Vec<usize> {
        let mut seq: Vec<usize> = (0..self.nv() as u32).map(|v| self.degree(v)).collect();
        seq.sort_unstable();
        seq
    }

    /// Degree distribution: `result[d]` is the number of vertices with degree `d`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_graph::{SimpleGraph, Graph};
    ///
    /// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
    /// assert_eq!(g.degree_distribution(), vec![0, 3, 0, 1]);
    /// ```
    fn degree_distribution(&self) -> Vec<usize> {
        let mut dist = vec![0usize; self.nv()];
        for v in 0..self.nv() as u32 {
            let d = self.degree(v);
            if d >= dist.len() {
                dist.resize(d + 1, 0);
            }
            dist[d] += 1;
        }
        // Trim trailing zeros
        while dist.last() == Some(&0) {
            dist.pop();
        }
        dist
    }
}

/// Graph density: `ne / (nv choose 2)`. Returns 0.0 for graphs with < 2 vertices.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, density};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2), (0, 2)]);
/// assert!((density(&g) - 1.0).abs() < 1e-10);
/// ```
pub fn density(g: &impl Graph) -> f64 {
    g.density()
}

/// Sorted degree sequence (ascending).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, degree_sequence};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
/// assert_eq!(degree_sequence(&g), vec![1, 1, 1, 3]);
/// ```
pub fn degree_sequence(g: &impl Graph) -> Vec<usize> {
    g.degree_sequence()
}

/// Degree distribution: `result[d]` is the number of vertices with degree `d`.
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, Graph};
///
/// let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
/// let dist = g.degree_distribution();
/// assert_eq!(dist, vec![0, 3, 0, 1]); // three degree-1, one degree-3
/// ```
pub fn degree_distribution(g: &impl Graph) -> Vec<usize> {
    g.degree_distribution()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleGraph;

    fn assert_graph_basics(g: &impl Graph) {
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 2);
        assert!(g.has_vertex(0));
        assert!(!g.has_vertex(3));
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 0));
        assert!(!g.has_edge(0, 2));
        assert_eq!(g.degree(0), 1);
        assert_eq!(g.neighbors(0), &[1]);
    }

    #[test]
    fn test_simple_graph_implements_graph_trait() {
        let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        assert_graph_basics(&g);
    }

    #[test]
    fn test_density() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let d = density(&g);
        assert!((d - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_density_empty() {
        let g = SimpleGraph::new(0);
        assert_eq!(density(&g), 0.0);
    }

    #[test]
    fn test_density_single() {
        let g = SimpleGraph::new(1);
        assert_eq!(density(&g), 0.0);
    }

    #[test]
    fn test_degree_sequence() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3), (1, 2)]);
        assert_eq!(degree_sequence(&g), vec![1, 2, 2, 3]);
    }

    #[test]
    fn test_trait_method_syntax() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        // Can use method syntax via the trait
        assert!((g.density() - 2.0 / 3.0).abs() < 1e-10);
        assert_eq!(g.degree_sequence(), vec![2, 2, 2, 2]);
    }

    #[test]
    fn test_degree_distribution() {
        // Star graph: vertex 0 has degree 3, vertices 1,2,3 have degree 1
        let g = SimpleGraph::from_edges(4, &[(0, 1), (0, 2), (0, 3)]);
        let dist = degree_distribution(&g);
        assert_eq!(dist, vec![0, 3, 0, 1]);
    }

    #[test]
    fn test_degree_distribution_empty() {
        let g = SimpleGraph::new(0);
        let dist = degree_distribution(&g);
        assert!(dist.is_empty());
    }

    #[test]
    fn test_degree_distribution_isolated() {
        let g = SimpleGraph::new(3);
        let dist = degree_distribution(&g);
        assert_eq!(dist, vec![3]);
    }
}
