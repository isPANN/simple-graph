use crate::SimpleGraph;

/// Iterator over edges of a [`SimpleGraph`], yielding `(u, v)` with `u < v`.
pub struct Edges<'a> {
    graph: &'a SimpleGraph,
    u: u32,
    idx: usize,
}

impl<'a> Edges<'a> {
    pub(crate) fn new(graph: &'a SimpleGraph) -> Self {
        Self {
            graph,
            u: 0,
            idx: 0,
        }
    }
}

impl<'a> Iterator for Edges<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let nv = self.graph.nv() as u32;
        while self.u < nv {
            let nbrs = self.graph.neighbors(self.u);
            while self.idx < nbrs.len() {
                let v = nbrs[self.idx];
                self.idx += 1;
                if v > self.u {
                    return Some((self.u, v));
                }
            }
            self.u += 1;
            self.idx = 0;
        }
        None
    }
}
