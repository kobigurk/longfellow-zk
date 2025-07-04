pub mod affine;
pub mod dense;
pub mod sparse;
pub mod eq;
pub mod eqs;

pub use affine::*;
pub use dense::*;
pub use sparse::*;
pub use eq::*;
pub use eqs::*;

pub type CornerIndex = usize;