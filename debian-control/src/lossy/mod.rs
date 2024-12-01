//! Lossy parsing of Debian control files

pub mod apt;
pub mod buildinfo;
mod control;
pub use control::*;
pub mod ftpmaster;
mod relations;
pub use relations::*;
