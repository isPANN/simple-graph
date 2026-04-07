use criterion::{criterion_group, criterion_main, Criterion};
use petgraph::graph::UnGraph;
use petgraph::visit::Walker;
use simple_graph::{algo, gen, CsrBuilder, CsrGraph};
use std::hint::black_box;

/// Build a petgraph UnGraph equivalent to a 100x100 grid.
fn petgraph_grid(rows: usize, cols: usize) -> UnGraph<(), ()> {
    let n = rows * cols;
    let mut g = UnGraph::with_capacity(n, 2 * n);
    for _ in 0..n {
        g.add_node(());
    }
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                g.add_edge((v as u32).into(), ((v + 1) as u32).into(), ());
            }
            if r + 1 < rows {
                g.add_edge((v as u32).into(), ((v + cols) as u32).into(), ());
            }
        }
    }
    g
}

/// Build a petgraph UnGraph complete graph K_n.
fn petgraph_complete(n: usize) -> UnGraph<(), ()> {
    let mut g = UnGraph::with_capacity(n, n * (n - 1) / 2);
    for _ in 0..n {
        g.add_node(());
    }
    for u in 0..n {
        for v in (u + 1)..n {
            g.add_edge((u as u32).into(), (v as u32).into(), ());
        }
    }
    g
}

fn bench_construction_vs(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_construction");

    group.bench_function("simple_graph_complete_100", |b| {
        b.iter(|| gen::complete(black_box(100)))
    });

    group.bench_function("petgraph_complete_100", |b| {
        b.iter(|| petgraph_complete(black_box(100)))
    });

    group.bench_function("simple_graph_grid_100x100", |b| {
        b.iter(|| gen::grid_2d(black_box(100), black_box(100)))
    });

    // CsrGraph via from_sorted_unique_edges (manual edge list)
    group.bench_function("csr_from_edges_grid_100x100", |b| {
        b.iter(|| {
            let rows = black_box(100);
            let cols = black_box(100);
            let n = rows * cols;
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
            CsrGraph::from_sorted_unique_edges(n, &edges)
        })
    });

    group.bench_function("csr_graph_grid_100x100", |b| {
        b.iter(|| gen::grid_2d_csr(black_box(100), black_box(100)))
    });

    group.bench_function("csr_from_edges_complete_100", |b| {
        b.iter(|| {
            let n = black_box(100);
            let mut edges = Vec::with_capacity(n * (n - 1) / 2);
            for u in 0..n as u32 {
                for v in (u + 1)..n as u32 {
                    edges.push((u, v));
                }
            }
            CsrGraph::from_sorted_unique_edges(n, &edges)
        })
    });

    group.bench_function("csr_graph_complete_100", |b| {
        b.iter(|| gen::complete_csr(black_box(100)))
    });

    // CsrBuilder — incremental add_edge like petgraph
    group.bench_function("csr_builder_grid_100x100", |b| {
        b.iter(|| {
            let rows = black_box(100);
            let cols = black_box(100);
            let n = rows * cols;
            let mut builder = CsrBuilder::with_capacity(n, 2 * n);
            for r in 0..rows {
                for c in 0..cols {
                    let v = (r * cols + c) as u32;
                    if c + 1 < cols {
                        builder.add_edge(v, v + 1);
                    }
                    if r + 1 < rows {
                        builder.add_edge(v, v + cols as u32);
                    }
                }
            }
            builder.build()
        })
    });

    group.bench_function("petgraph_grid_100x100", |b| {
        b.iter(|| petgraph_grid(black_box(100), black_box(100)))
    });

    group.finish();
}

fn bench_query_vs(c: &mut Criterion) {
    let sg = gen::complete(200);
    let csr = CsrGraph::from(&sg);
    let pg = petgraph_complete(200);

    let mut group = c.benchmark_group("vs_query");

    group.bench_function("simple_graph_has_edge", |b| {
        b.iter(|| sg.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("simple_graph_csr_has_edge", |b| {
        b.iter(|| csr.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("petgraph_contains_edge", |b| {
        b.iter(|| pg.contains_edge(black_box(50u32.into()), black_box(150u32.into())))
    });

    group.bench_function("simple_graph_neighbors_iter", |b| {
        b.iter(|| sg.neighbors(black_box(50)).iter().copied().sum::<u32>())
    });

    group.bench_function("simple_graph_csr_neighbors_iter", |b| {
        b.iter(|| csr.neighbors(black_box(50)).iter().copied().sum::<u32>())
    });

    group.bench_function("petgraph_neighbors_iter", |b| {
        b.iter(|| {
            pg.neighbors(black_box(50u32.into()))
                .map(|n| n.index() as u32)
                .sum::<u32>()
        })
    });

    group.finish();
}

fn bench_bfs_vs(c: &mut Criterion) {
    let sg = gen::grid_2d(100, 100);
    let csr = CsrGraph::from(&sg);
    let pg = petgraph_grid(100, 100);

    let mut group = c.benchmark_group("vs_bfs");

    group.bench_function("simple_graph_bfs", |b| {
        b.iter(|| algo::bfs(black_box(&sg), 0).count())
    });

    group.bench_function("simple_graph_csr_bfs", |b| {
        b.iter(|| algo::bfs(black_box(&csr), 0).count())
    });

    group.bench_function("petgraph_bfs", |b| {
        b.iter(|| {
            petgraph::visit::Bfs::new(&pg, black_box(0u32.into()))
                .iter(&pg)
                .count()
        })
    });

    group.finish();
}

fn bench_dfs_vs(c: &mut Criterion) {
    let sg = gen::grid_2d(100, 100);
    let csr = CsrGraph::from(&sg);
    let pg = petgraph_grid(100, 100);

    let mut group = c.benchmark_group("vs_dfs");

    group.bench_function("simple_graph_dfs", |b| {
        b.iter(|| algo::dfs(black_box(&sg), 0).count())
    });

    group.bench_function("simple_graph_csr_dfs", |b| {
        b.iter(|| algo::dfs(black_box(&csr), 0).count())
    });

    group.bench_function("petgraph_dfs", |b| {
        b.iter(|| {
            petgraph::visit::Dfs::new(&pg, black_box(0u32.into()))
                .iter(&pg)
                .count()
        })
    });

    group.finish();
}

fn bench_connected_components_vs(c: &mut Criterion) {
    let sg = gen::grid_2d(100, 100);
    let pg = petgraph_grid(100, 100);

    let mut group = c.benchmark_group("vs_connected_components");

    group.bench_function("simple_graph", |b| {
        b.iter(|| algo::connected_components(black_box(&sg)))
    });

    group.bench_function("petgraph", |b| {
        b.iter(|| petgraph::algo::connected_components(black_box(&pg)))
    });

    group.finish();
}

fn bench_edge_count_vs(c: &mut Criterion) {
    let sg = gen::grid_2d(100, 100);
    let csr = gen::grid_2d_csr(100, 100);
    let pg = petgraph_grid(100, 100);

    let mut group = c.benchmark_group("vs_edge_iteration");

    group.bench_function("simple_graph_edges", |b| b.iter(|| sg.edges().count()));

    group.bench_function("csr_edges", |b| b.iter(|| csr.edges().count()));

    group.bench_function("petgraph_edges", |b| {
        b.iter(|| pg.edge_references().count())
    });

    group.finish();
}

criterion_group!(
    vs_petgraph,
    bench_construction_vs,
    bench_query_vs,
    bench_bfs_vs,
    bench_dfs_vs,
    bench_connected_components_vs,
    bench_edge_count_vs,
);
criterion_main!(vs_petgraph);
