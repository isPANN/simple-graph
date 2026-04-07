use crate::graph::Graph;
use crate::SimpleGraph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrGraph {
    ne: usize,
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<u32>,
}

impl CsrGraph {
    #[inline]
    pub fn nv(&self) -> usize {
        if self.offsets.is_empty() { 0 } else { self.offsets.len() - 1 }
    }

    #[inline]
    pub fn ne(&self) -> usize { self.ne }

    #[inline]
    pub fn has_vertex(&self, v: u32) -> bool { (v as usize) < self.nv() }

    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        if !self.has_vertex(u) || !self.has_vertex(v) { return false; }
        self.neighbors(u).binary_search(&v).is_ok()
    }

    #[inline]
    pub fn degree(&self, v: u32) -> usize {
        let vi = v as usize;
        self.offsets[vi + 1] - self.offsets[vi]
    }

    #[inline]
    pub fn neighbors(&self, v: u32) -> &[u32] {
        let vi = v as usize;
        &self.targets[self.offsets[vi]..self.offsets[vi + 1]]
    }

    pub fn to_simple_graph(&self) -> SimpleGraph {
        SimpleGraph::from_csr(&self.offsets, &self.targets, self.ne)
    }
}

impl From<&SimpleGraph> for CsrGraph {
    fn from(sg: &SimpleGraph) -> Self {
        let n = sg.nv();
        let mut offsets = Vec::with_capacity(n + 1);
        let total: usize = (0..n).map(|v| sg.neighbors(v as u32).len()).sum();
        let mut targets = Vec::with_capacity(total);
        let mut offset = 0;
        for v in 0..n {
            offsets.push(offset);
            let nbrs = sg.neighbors(v as u32);
            targets.extend_from_slice(nbrs);
            offset += nbrs.len();
        }
        offsets.push(offset);
        CsrGraph { ne: sg.ne(), offsets, targets }
    }
}

impl Graph for CsrGraph {
    #[inline] fn nv(&self) -> usize { CsrGraph::nv(self) }
    #[inline] fn ne(&self) -> usize { CsrGraph::ne(self) }
    #[inline] fn has_vertex(&self, v: u32) -> bool { CsrGraph::has_vertex(self, v) }
    fn has_edge(&self, u: u32, v: u32) -> bool { CsrGraph::has_edge(self, u, v) }
    #[inline] fn degree(&self, v: u32) -> usize { CsrGraph::degree(self, v) }
    #[inline] fn neighbors(&self, v: u32) -> &[u32] { CsrGraph::neighbors(self, v) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, SimpleGraph};

    #[test]
    fn test_csr_from_simple_graph() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.nv(), 4);
        assert_eq!(csr.ne(), 3);
    }

    #[test]
    fn test_csr_has_edge() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        assert!(csr.has_edge(0, 1));
        assert!(csr.has_edge(1, 0));
        assert!(!csr.has_edge(0, 2));
    }

    #[test]
    fn test_csr_neighbors() {
        let sg = SimpleGraph::from_edges(4, &[(0, 3), (0, 1), (0, 2)]);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.neighbors(0), &[1, 2, 3]);
        assert_eq!(csr.neighbors(1), &[0]);
        assert_eq!(csr.degree(0), 3);
    }

    #[test]
    fn test_csr_empty() {
        let sg = SimpleGraph::new(0);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.nv(), 0);
        assert_eq!(csr.ne(), 0);
    }

    #[test]
    fn test_csr_oob_returns_false() {
        let sg = SimpleGraph::from_edges(2, &[(0, 1)]);
        let csr = CsrGraph::from(&sg);
        assert!(!csr.has_edge(5, 0));
        assert!(!csr.has_vertex(2));
    }

    #[test]
    fn test_csr_graph_trait() {
        let sg = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
        let csr = CsrGraph::from(&sg);
        fn check(g: &impl Graph) {
            assert_eq!(g.nv(), 3);
            assert_eq!(g.ne(), 2);
            assert!(g.has_edge(0, 1));
        }
        check(&csr);
    }

    #[test]
    fn test_csr_invariants() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        assert_eq!(csr.offsets.len(), 5);
        for i in 1..csr.offsets.len() {
            assert!(csr.offsets[i] >= csr.offsets[i - 1]);
        }
        assert_eq!(csr.targets.len(), 2 * csr.ne());
    }

    #[test]
    fn test_csr_to_simple_graph() {
        let sg = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let csr = CsrGraph::from(&sg);
        let sg2 = csr.to_simple_graph();
        assert_eq!(sg, sg2);
    }
}
