use crate::SimpleGraph;
use rand::Rng;
use rand::RngExt;

/// Random graph G(n, p): each of the `n*(n-1)/2` possible edges is included
/// independently with probability `p`.
///
/// # Examples
///
/// ```
/// use simple_graph::gen;
/// use rand::SeedableRng;
///
/// let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
/// let g = gen::erdos_renyi(10, 0.5, &mut rng);
/// assert_eq!(g.nv(), 10);
/// ```
pub fn erdos_renyi(n: usize, p: f64, rng: &mut impl Rng) -> SimpleGraph {
    let mut edges = Vec::new();
    for u in 0..n as u32 {
        for v in (u + 1)..n as u32 {
            if rng.random_bool(p) {
                edges.push((u, v));
            }
        }
    }
    SimpleGraph::from_edges(n, &edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_erdos_renyi_empty() {
        let mut rng = SmallRng::seed_from_u64(42);
        let g = erdos_renyi(10, 0.0, &mut rng);
        assert_eq!(g.nv(), 10);
        assert_eq!(g.ne(), 0);
    }

    #[test]
    fn test_erdos_renyi_complete() {
        let mut rng = SmallRng::seed_from_u64(42);
        let g = erdos_renyi(5, 1.0, &mut rng);
        assert_eq!(g.nv(), 5);
        assert_eq!(g.ne(), 10);
    }

    #[test]
    fn test_erdos_renyi_reproducible() {
        let g1 = erdos_renyi(100, 0.1, &mut SmallRng::seed_from_u64(123));
        let g2 = erdos_renyi(100, 0.1, &mut SmallRng::seed_from_u64(123));
        assert_eq!(g1, g2);
    }

    #[test]
    fn test_erdos_renyi_reasonable_density() {
        let mut rng = SmallRng::seed_from_u64(99);
        let g = erdos_renyi(200, 0.05, &mut rng);
        assert!(g.ne() > 500);
        assert!(g.ne() < 1500);
    }
}
