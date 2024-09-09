//! A library for parsing and generating Debian patch headers.
//!
//! # Examples
//!
//! ```rust
//! use dep3::PatchHeader;
//! use std::str::FromStr;
//! let text = r#"From: John Doe <john.doe@example>
//! Date: Mon, 1 Jan 2000 00:00:00 +0000
//! Subject: [PATCH] fix a bug
//! Bug-Debian: https://bugs.debian.org/123456
//! Bug: https://bugzilla.example.com/bug.cgi?id=123456
//! Forwarded: not-needed
//! "#;
//!
//! let patch_header = PatchHeader::from_str(text).unwrap();
//! assert_eq!(patch_header.description(), Some("[PATCH] fix a bug".to_string()));
//! assert_eq!(patch_header.vendor_bugs("Debian").collect::<Vec<_>>(), vec!["https://bugs.debian.org/123456".to_string()]);
//! ```
mod fields;
pub use fields::*;
pub mod lossless;
pub mod lossy;

pub use lossless::PatchHeader;
