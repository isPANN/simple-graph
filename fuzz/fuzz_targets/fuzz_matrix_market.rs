#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(g) = simple_graph::io::read_matrix_market(data) {
        // Verify graph invariants on any successfully parsed graph
        let n = g.nv();
        for v in 0..n as u32 {
            assert!(g.has_vertex(v));
            let nbrs = g.neighbors(v);
            // Neighbors must be sorted
            assert!(nbrs.windows(2).all(|w| w[0] < w[1]));
            // All neighbors must be valid vertices
            for &u in nbrs {
                assert!(g.has_vertex(u));
                // Symmetry: if v has neighbor u, u must have neighbor v
                assert!(g.has_edge(u, v));
            }
            // No self-loops
            assert!(!nbrs.contains(&v));
        }
        // Edge count consistency
        let sum_degrees: usize = (0..n as u32).map(|v| g.degree(v)).sum();
        assert_eq!(sum_degrees, 2 * g.ne());
    }
});
