//! Parser for Debian control files.
//!
//! This crate provides a parser for Debian control files.
//!
//! # Example
//!
//! ```rust
//! use debian_control::lossy::Control;
//! use debian_control::fields::Priority;
//! use std::fs::File;
//!
//! let mut control = Control::new();
//! let mut source = &mut control.source;
//! source.name = "hello".to_string();
//! source.section = Some("rust".to_string());
//!
//! let mut binary = control.add_binary("hello");
//! binary.architecture = Some("amd64".to_string());
//! binary.priority = Some(Priority::Optional);
//! binary.description = Some("Hello, world!".to_string());
//!
//! assert_eq!(control.to_string(), r#"Source: hello
//! Section: rust
//!
//! Package: hello
//! Architecture: amd64
//! Priority: optional
//! Description: Hello, world!
//! "#);
//! ```
//!
//! See the ``lossless`` module for a parser that preserves all comments and formatting, and
//! as well as allowing inline errors.
pub mod lossy;
pub use lossless::control::{Binary, Control, Source};
pub mod fields;
pub use fields::*;
pub mod lossless;
pub use lossless::apt;
pub use lossless::control;
pub use lossless::changes;
pub mod relations;
pub mod pgp;
pub mod vcs;

use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub enum ParseIdentityError {
    NoEmail,
}

impl std::fmt::Display for ParseIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseIdentityError::NoEmail => write!(f, "No email found"),
        }
    }
}

impl std::error::Error for ParseIdentityError {}

/// Parse an identity string into a name and an email address.
///
/// The input string should be in the format `Name <email>`. If the email is missing, an error is
/// returned.
///
/// # Example
/// ```
/// use debian_control::parse_identity;
/// assert_eq!(parse_identity("Joe Example <joe@example.com>"), Ok(("Joe Example", "joe@example.com")));
/// ```
///
/// # Arguments
/// * `s` - The input string.
///
/// # Returns
/// A tuple with the name and the email address.
pub fn parse_identity(s: &str) -> Result<(&str, &str), ParseIdentityError> {
    // Support Name <email> and email, but ensure email contains an "@".
    if let Some((name, email)) = s.split_once('<') {
        if let Some(email) = email.strip_suffix('>') {
            Ok((name.trim(), email.trim()))
        } else {
            Err(ParseIdentityError::NoEmail)
        }
    } else if s.contains('@') {
        Ok(("", s.trim()))
    } else {
        Err(ParseIdentityError::NoEmail)
    }
}

pub trait VersionLookup {
    fn lookup_version<'a>(&'a self, package: &'_ str) -> Option<std::borrow::Cow<'a, debversion::Version>>;
}

impl VersionLookup for std::collections::HashMap<String, debversion::Version> {
    fn lookup_version<'a>(&'a self, package: &str) -> Option<Cow<'a, debversion::Version>> {
        self.get(package).map(|v| Cow::Borrowed(v))
    }
}

impl<F> VersionLookup for F
where
    F: Fn(&str) -> Option<debversion::Version>,
{
    fn lookup_version<'a>(&'a self, name: &str) -> Option<Cow<'a, debversion::Version>> {
        self(name).map(|v| Cow::Owned(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity() {
        assert_eq!(
            parse_identity("Joe Example <joe@example.com>"),
            Ok(("Joe Example", "joe@example.com"))
        );
        assert_eq!(
            parse_identity("joe@example.com"),
            Ok(("", "joe@example.com"))
        );
        assert_eq!(parse_identity("somebody"), Err(ParseIdentityError::NoEmail));
    }
}
