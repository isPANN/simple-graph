use crate::{graph::Graph, SimpleGraph};
use std::io::{self, BufRead, Write};

/// Write a graph in Matrix Market symmetric coordinate pattern format.
/// Uses 1-based indexing. Writes the lower triangle (row > col).
///
/// # Examples
///
/// ```
/// use simple_graph::{SimpleGraph, io};
///
/// let g = SimpleGraph::from_edges(3, &[(0, 1)]);
/// let mut buf = Vec::new();
/// io::write_matrix_market(&g, &mut buf).unwrap();
/// let text = String::from_utf8(buf).unwrap();
/// assert!(text.starts_with("%%MatrixMarket"));
/// ```
pub fn write_matrix_market<G: Graph>(graph: &G, mut w: impl Write) -> io::Result<()> {
    writeln!(w, "%%MatrixMarket matrix coordinate pattern symmetric")?;
    let n = graph.nv();
    writeln!(w, "{} {} {}", n, n, graph.ne())?;
    for v in 0..n as u32 {
        for &u in graph.neighbors(v) {
            if u > v {
                writeln!(w, "{} {}", u + 1, v + 1)?;
            }
        }
    }
    Ok(())
}

/// Read a graph from Matrix Market symmetric coordinate pattern format.
/// Validates banner for "coordinate pattern symmetric". Converts 1-based to 0-based.
/// Errors on invalid format, count mismatches, self-loops, OOB vertices.
///
/// # Examples
///
/// ```
/// use simple_graph::io;
///
/// let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n1 2\n";
/// let g = io::read_matrix_market(&input[..]).unwrap();
/// assert_eq!(g.nv(), 3);
/// assert!(g.has_edge(0, 1));
/// ```
pub fn read_matrix_market(r: impl BufRead) -> io::Result<SimpleGraph> {
    let mut lines = r.lines();

    // Parse and validate banner
    let banner = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty input"))??;
    let banner_lower = banner.trim().to_lowercase();
    if !banner_lower.starts_with("%%matrixmarket") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing %%MatrixMarket banner",
        ));
    }
    if !banner_lower.contains("coordinate") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only coordinate format supported",
        ));
    }
    if !banner_lower.contains("pattern") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only pattern format supported",
        ));
    }
    if !banner_lower.contains("symmetric") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "only symmetric format supported",
        ));
    }

    // Parse size line
    let (nv, nnz_declared) = loop {
        let line = lines
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing size line"))??;
        let trimmed = line.trim();
        if trimmed.starts_with('%') || trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let nrow: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nrow"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let ncol: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing ncol"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let nnz: usize = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing nnz"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if nrow != ncol {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-square matrix",
            ));
        }
        break (nrow, nnz);
    };

    // Parse entries
    let mut edges = Vec::new();
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let row: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing row"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let col: u32 = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing col"))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if parts.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "extra fields in entry line (expected pattern format)",
            ));
        }
        if row == 0 || col == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Matrix Market uses 1-based indices",
            ));
        }
        edges.push((row - 1, col - 1));
    }

    // Canonicalize to (min, max) and dedup to get unique undirected edges
    let mut canonical: Vec<(u32, u32)> = edges
        .iter()
        .map(|&(u, v)| if u <= v { (u, v) } else { (v, u) })
        .collect();
    canonical.sort_unstable();
    canonical.dedup();
    if canonical.len() != nnz_declared {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "declared {} unique entries but found {}",
                nnz_declared,
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
        write_matrix_market(&g, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("%%MatrixMarket"));
        let g2 = read_matrix_market(text.as_bytes()).unwrap();
        assert_eq!(g2.nv(), 4);
        assert_eq!(g2.ne(), 3);
        assert!(g2.has_edge(0, 1));
        assert!(g2.has_edge(2, 3));
    }

    #[test]
    fn test_read_1indexed() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 2\n1 2\n2 3\n";
        let g = read_matrix_market(&input[..]).unwrap();
        assert_eq!(g.nv(), 3);
        assert_eq!(g.ne(), 2);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 2));
    }

    #[test]
    fn test_empty_graph() {
        let g = SimpleGraph::new(3);
        let mut buf = Vec::new();
        write_matrix_market(&g, &mut buf).unwrap();
        let g2 = read_matrix_market(&buf[..]).unwrap();
        assert_eq!(g2.nv(), 3);
        assert_eq!(g2.ne(), 0);
    }

    #[test]
    fn test_reject_non_symmetric() {
        let input = b"%%MatrixMarket matrix coordinate pattern general\n3 3 1\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_non_square() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 4 1\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_zero_index() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n0 1\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_count_mismatch() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 2\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_real_format() {
        let input = b"%%MatrixMarket matrix coordinate real symmetric\n3 3 1\n1 2 3.5\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_integer_format() {
        let input = b"%%MatrixMarket matrix coordinate integer symmetric\n3 3 1\n1 2 7\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_reject_extra_fields_in_entry() {
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n1 2 extra\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }

    #[test]
    fn test_duplicate_entries_accepted() {
        // "2 1" and "1 2" are the same undirected edge — declares 1 unique
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 1\n2 1\n1 2\n";
        let g = read_matrix_market(&input[..]).unwrap();
        assert_eq!(g.ne(), 1);
    }

    #[test]
    fn test_duplicate_entries_count_mismatch() {
        // Declares 2 but only 1 unique after dedup
        let input = b"%%MatrixMarket matrix coordinate pattern symmetric\n3 3 2\n2 1\n1 2\n";
        assert!(read_matrix_market(&input[..]).is_err());
    }
}
