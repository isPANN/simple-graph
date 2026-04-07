/// Read-only interface shared by all graph representations.
pub trait Graph {
    fn nv(&self) -> usize;
    fn ne(&self) -> usize;
    fn has_vertex(&self, v: u32) -> bool;
    fn has_edge(&self, u: u32, v: u32) -> bool;
    fn degree(&self, v: u32) -> usize;
    fn neighbors(&self, v: u32) -> &[u32];
}

/// Graph density: `ne / (nv choose 2)`. Returns 0.0 for graphs with < 2 vertices.
pub fn density(g: &impl Graph) -> f64 {
    let n = g.nv();
    if n < 2 {
        return 0.0;
    }
    let max_edges = n * (n - 1) / 2;
    g.ne() as f64 / max_edges as f64
}

/// Sorted degree sequence (ascending).
pub fn degree_sequence(g: &impl Graph) -> Vec<usize> {
    let mut seq: Vec<usize> = (0..g.nv() as u32).map(|v| g.degree(v)).collect();
    seq.sort_unstable();
    seq
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
}
