use criterion::{criterion_group, criterion_main, Criterion};
use simple_graph::{algo, gen, CsrGraph, Graph, SimpleGraph};
use std::hint::black_box;

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    group.bench_function("from_edges_100", |b| {
        let edges: Vec<(u32, u32)> = (0..99).map(|i| (i, i + 1)).collect();
        b.iter(|| SimpleGraph::from_edges(black_box(100), black_box(&edges)))
    });

    group.bench_function("from_edges_10000", |b| {
        let edges: Vec<(u32, u32)> = (0..9999).map(|i| (i, i + 1)).collect();
        b.iter(|| SimpleGraph::from_edges(black_box(10_000), black_box(&edges)))
    });

    group.bench_function("complete_100", |b| b.iter(|| gen::complete(black_box(100))));

    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let g = gen::complete(200);
    let csr = CsrGraph::from(&g);

    let mut group = c.benchmark_group("query");

    group.bench_function("has_edge_simple", |b| {
        b.iter(|| g.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("has_edge_csr", |b| {
        b.iter(|| csr.has_edge(black_box(50), black_box(150)))
    });

    group.bench_function("neighbors_simple", |b| {
        b.iter(|| g.neighbors(black_box(50)))
    });

    group.bench_function("neighbors_csr", |b| b.iter(|| csr.neighbors(black_box(50))));

    group.bench_function("degree_sequence", |b| b.iter(|| g.degree_sequence()));

    group.bench_function("degree_distribution", |b| {
        b.iter(|| g.degree_distribution())
    });

    group.finish();
}

fn bench_algorithms(c: &mut Criterion) {
    let g = gen::grid_2d(100, 100);
    let csr = CsrGraph::from(&g);

    let mut group = c.benchmark_group("algorithms");

    group.bench_function("bfs_grid_100x100_simple", |b| {
        b.iter(|| algo::bfs(black_box(&g), 0).count())
    });

    group.bench_function("bfs_grid_100x100_csr", |b| {
        b.iter(|| algo::bfs(black_box(&csr), 0).count())
    });

    group.bench_function("connected_components_grid", |b| {
        b.iter(|| algo::connected_components(black_box(&g)))
    });

    group.bench_function("dfs_grid_100x100_simple", |b| {
        b.iter(|| algo::dfs(black_box(&g), 0).count())
    });

    group.bench_function("dfs_grid_100x100_csr", |b| {
        b.iter(|| algo::dfs(black_box(&csr), 0).count())
    });

    group.bench_function("shortest_path_grid_100x100", |b| {
        b.iter(|| algo::shortest_path_lengths(black_box(&g), 0))
    });

    group.finish();
}

fn bench_csr_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_conversion");

    let g_small = gen::complete(100);
    group.bench_function("to_csr_complete_100", |b| {
        b.iter(|| CsrGraph::from(black_box(&g_small)))
    });

    let g_large = gen::grid_2d(100, 100);
    group.bench_function("to_csr_grid_100x100", |b| {
        b.iter(|| CsrGraph::from(black_box(&g_large)))
    });

    let csr = CsrGraph::from(&g_small);
    group.bench_function("to_simple_complete_100", |b| {
        b.iter(|| black_box(&csr).to_simple_graph())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_construction,
    bench_query,
    bench_algorithms,
    bench_csr_conversion
);
criterion_main!(benches);
