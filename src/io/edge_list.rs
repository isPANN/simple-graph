use crate::{Graph, SimpleGraph};
use std::io::{self, BufRead, Write};

/// Write a graph in edge-list format.
/// Format: first line is `nv ne`, followed by one `u v` line per edge (u < v).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, io};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1), (1, 2)]);
/// let mut buf = Vec::new();
/// io::write_edge_list(&g, &mut buf).unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.starts_with("3 2"));
/// ```
pub fn write_edge_list<G: Graph>(graph: &G, mut w: impl Write) -> io::Result<()> {
    writeln!(w, "{} {}", graph.nv(), graph.ne())?;
    for v in 0..graph.nv() as u32 {
        for &u in graph.neighbors(v) {
            if u > v {
                writeln!(w, "{} {}", v, u)?;
            }
        }
    }
    Ok(())
}

/// Read a graph from edge-list format.
/// Lines starting with `#` or `%` are skipped. First non-comment line must be
/// `nv ne`. Remaining lines are `u v` edges. Returns error if declared edge
/// count doesn't match, or if edges contain self-loops/out-of-range vertices.
///
/// # Examples
///
/// ```
/// use simple_graph::io;
///
/// let input = b"3 2\n0 1\n1 2\n";
/// let g = io::read_edge_list(&input[..]).unwrap();
/// assert_eq!(g.nv(), 3);
/// assert_eq!(g.ne(), 2);
/// ```
pub fn read_edge_list(r: impl BufRead) -> io::Result<SimpleGraph> {
    let mut lines = r.lines();
    let (nv, ne_declared) = loop {
        let line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty input"))??;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let nv: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nv"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let ne: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing ne"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        break (nv, ne);
    };
    let mut edges = Vec::new();
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let u: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing u"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let v: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing v"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        edges.push((u, v));
    }
    // Canonicalize to (min, max) and dedup to get unique undirected edges
    let mut canonical: Vec<(u32, u32)> = edges
        .iter()
        .map(|&(u, v)| if u <= v { (u, v) } else { (v, u) })
        .collect();
    canonical.sort_unstable();
    canonical.dedup();
    if canonical.len() != ne_declared {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "declared {} unique edges but found {}",
                ne_declared,
                canonical.len()
            ),
        ));
    }
    SimpleGraph::try_from_edges(nv, &canonical)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleGraph;

    #[test]
    fn test_write_read_roundtrip() {
        let g = SimpleGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        let mut buf = Vec::new();
        write_edge_list(&g, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(text.lines().count(), 4);
        let g2 = read_edge_list(text.as_bytes()).unwrap();
        assert_eq!(g2.nv(), 4);
        assert_eq!(g2.ne(), 3);
        assert!(g2.has_edge(0, 1));
        assert!(g2.has_edge(2, 3));
    }

    #[test]
    fn test_read_with_comments() {
        let input = b"# comment\n3 1\n0 1\n";
        let g = read_edge_list(&input[..]).unwrap();
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_empty_graph_roundtrip() {
        let g = SimpleGraph::new(5);
        let mut buf = Vec::new();
        write_edge_list(&g, &mut buf).unwrap();
        let g2 = read_edge_list(&buf[..]).unwrap();
        assert_eq!(g2.nv(), 5);
        assert_eq!(g2.ne(), 0);
    }

    #[test]
    fn test_read_self_loop_error() {
        let input = b"3 1\n1 1\n";
        assert!(read_edge_list(&input[..]).is_err());
    }

    #[test]
    fn test_read_oob_error() {
        let input = b"3 1\n0 5\n";
        assert!(read_edge_list(&input[..]).is_err());
    }

    #[test]
    fn test_read_count_mismatch_error() {
        let input = b"3 2\n0 1\n";
        assert!(read_edge_list(&input[..]).is_err());
    }

    #[test]
    fn test_read_duplicate_edges() {
        // "0 1" and "1 0" are the same undirected edge — declares 1 unique edge
        let input = b"3 1\n0 1\n1 0\n";
        let g = read_edge_list(&input[..]).unwrap();
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_read_duplicate_edges_count_mismatch() {
        // Declares 2 edges but only 1 unique after canonicalization
        let input = b"3 2\n0 1\n1 0\n";
        assert!(read_edge_list(&input[..]).is_err());
    }
}
