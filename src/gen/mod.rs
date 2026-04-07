mod basic;
pub use basic::{complete, complete_csr, cycle, grid_2d, grid_2d_csr, path};

#[cfg(feature = "rand")]
mod random;

#[cfg(feature = "rand")]
pub use random::erdos_renyi;
