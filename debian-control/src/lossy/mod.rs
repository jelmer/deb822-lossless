//! Lossy parsing of Debian control files

pub mod buildinfo;
mod control;
pub use control::*;
mod relations;
pub use relations::*;
