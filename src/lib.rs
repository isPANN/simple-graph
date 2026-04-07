#![forbid(unsafe_code)]

pub mod algo;
mod csr;
pub mod gen;
mod graph;
pub mod io;
mod iter;
mod simple_graph;

pub use csr::{CsrBuilder, CsrGraph};
pub use graph::{degree_distribution, degree_sequence, density, Graph};
pub use iter::{edges, Edges};
pub use simple_graph::SimpleGraph;

// Compile-time assertions: all public types are Send + Sync.
#[allow(dead_code)]
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assertions() {
        assert_send_sync::<SimpleGraph>();
        assert_send_sync::<CsrGraph>();
        assert_send_sync::<Edges<'_, SimpleGraph>>();
        assert_send_sync::<Edges<'_, CsrGraph>>();
        assert_send_sync::<algo::Bfs<'_, SimpleGraph>>();
        assert_send_sync::<algo::Bfs<'_, CsrGraph>>();
        assert_send_sync::<algo::Dfs<'_, SimpleGraph>>();
        assert_send_sync::<algo::Dfs<'_, CsrGraph>>();
        assert_send_sync::<CsrBuilder>();
    }
};
