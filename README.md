Parsers and editors for deb822 style files
==========================================

This crate contains parsers and editors for RFC822 style file as used in
Debian.

Four related crates that build on this one are:

* ``debian-control``: A parser and editor for Debian control files, apt lists.
* ``debian-copyright``: A parser and editor for Debian copyright files.
* ``dep3``: A parser and editor for Debian DEP-3 headers.
* [r-description](https://github.com/jelmer/r-description-rs): A parser and
editor for R DESCRIPTION files.
* ``apt-sources``: A parser and editor for APT source files (package repositories specification).

# Example

```rust
use deb822_lossless::Deb822;
use std::str::FromStr;

let input = r#"Package: deb822-lossless
Maintainer: Jelmer Vernooĳ <jelmer@debian.org>
Section: rust

Package: deb822-lossless
Architecture: any
Description: Lossless parser for deb822 style files.
  This parser can be used to parse files in the deb822 format, while preserving
  all whitespace and comments. It is based on the [rowan] library, which is a
  lossless parser library for Rust.
"#;

let deb822 = Deb822::from_str(input).unwrap();
assert_eq!(deb822.paragraphs().count(), 2);
```

A derive-macro is also provided for easily defining more Deb822-derived types:

```rust
#[cfg(feature = "derive")]
{
use deb822_lossless::{FromDeb822, ToDeb822};

#[derive(FromDeb822, ToDeb822, Debug, PartialEq)]
struct Test {
    #[deb822(field = "Package")]
    package: String,
    architecture: Option<String>,

    #[deb822(field = "Description")]
    description: String,
}

let input = r#"Package: deb822-lossless
Architecture: any
Description: Lossless parser for deb822 style files.
  This parser can be used to parse files in the deb822 format, while preserving
  all whitespace and comments. It is based on the [rowan] library, which is a
  lossless parser library for Rust.
"#;

use deb822_lossless::Paragraph;
use deb822_lossless::convert::FromDeb822Paragraph;
let parser: Paragraph = input.parse().unwrap();
let test: Test = FromDeb822Paragraph::from_paragraph(&parser).unwrap();

assert_eq!(test.package, "deb822-lossless");
}
```
