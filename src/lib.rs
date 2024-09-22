#![deny(missing_docs)]
#![allow(clippy::type_complexity)]
//! Parser for deb822 style files.
//!
//! This parser can be used to parse files in the deb822 format, while preserving
//! all whitespace and comments. It is based on the [rowan] library, which is a
//! lossless parser library for Rust.
//!
//! Once parsed, the file can be traversed or modified, and then written back to
//! a file.
//!
//! # Example
//!
//! ```rust
//! use deb822_lossless::Deb822;
//! use std::str::FromStr;
//!
//! let input = r#"Package: deb822-lossless
//! Maintainer: Jelmer Vernooĳ <jelmer@debian.org>
//! Homepage: https://github.com/jelmer/deb822-lossless
//! Section: rust
//!
//! Package: deb822-lossless
//! Architecture: any
//! Description: Lossless parser for deb822 style files.
//!   This parser can be used to parse files in the deb822 format, while preserving
//!   all whitespace and comments. It is based on the [rowan] library, which is a
//!   lossless parser library for Rust.
//! "#;
//!
//! let deb822 = Deb822::from_str(input).unwrap();
//! assert_eq!(deb822.paragraphs().count(), 2);
//! let homepage = deb822.paragraphs().nth(0).unwrap().get("Homepage");
//! assert_eq!(homepage.as_deref(), Some("https://github.com/jelmer/deb822-lossless"));
//! ```

mod convert;
mod lex;
pub mod lossless;
pub mod lossy;
pub use convert::{FromDeb822Paragraph, ToDeb822Paragraph};
#[cfg(feature = "derive")]
pub use deb822_derive::{FromDeb822, ToDeb822};
pub use lossless::{Deb822, Error, Paragraph, ParseError};

/// The indentation to use when writing a deb822 file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indentation {
    /// Use the same indentation as the original line for the value.
    FieldNameLength,

    /// The number of spaces to use for indentation.
    Spaces(u32),
}
