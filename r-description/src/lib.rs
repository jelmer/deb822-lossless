pub mod lossless;
/// A library for parsing and manipulating R DESCRIPTION files.
///
/// See https://r-pkgs.org/description.html for more information.
///
/// See the ``lossless`` module for a lossless parser that is
/// forgiving in the face of errors and preserves formatting while editing
/// at the expense of a more complex API.
///
/// # Example
///
/// ```rust
/// use r_description::RDescription;
/// use std::str::FromStr;
///
/// let desc = RDescription::from_str(
///    r#"Package: foo
/// Title: A Foo Package
/// Version: 0.1.0
/// Authors@R: person("First", "Last", email = "email@example.com", role = c("aut", "cre"))
/// Description: A longer description of the package.
/// License: MIT + file LICENSE
/// URL: https://example.com
/// "#).unwrap();
/// assert_eq!(desc.name, "foo");
/// assert_eq!(desc.title, "A Foo Package");
/// assert_eq!(desc.version, "0.1.0".to_string());
/// ```
pub mod lossy;

pub use lossy::RDescription;

#[derive(Debug, PartialEq, Eq)]
pub struct RCode(String);

impl std::str::FromStr for RCode {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl std::fmt::Display for RCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
