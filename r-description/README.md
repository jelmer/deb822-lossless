# R DESCRIPTION parser

This crate provides a parser and editor for the `DESCRIPTION` files used in R
packages.  Since the format is based on Debian control files, the parser
uses the ``deb822_lossless`` crate for the lower layer parsing.

See <https://r-pkgs.org/description.html> and
<https://cran.r-project.org/doc/manuals/R-exts.html> for more information on
the format.

Besides parsing the control files it also supports parsing and comparison
of version strings according to the R package versioning scheme.

## Example

```rust

use r_description::RDescription;

let desc = r_description::parse(r###"Package: foo
Version: 1.0
# Inline comment that will be preserved.
Depends: R (>= 3.0.0)
"###).unwrap();

assert_eq!(desc.get("Package"), Some("foo"));
assert_eq!(desc.get("Version"), Some("1.0"));
assert_eq!(desc.get("Depends"), Some("R (>= 3.0.0"));

desc.insert("License", "MIT");
```
