# Lossless parser for Debian Control files

This crate provides a parser for Debian control files. It is lossless, meaning
that it will preserve the original formatting of the file. It also provides a
way to serialize the parsed data back to a string.

```rust
use debian_control::{Control, Priority};
use std::fs::File;

let mut control = Control::new();
let mut source = control.add_source("hello");
source.set_section("rust");

let mut binary = control.add_binary("hello");
binary.set_architecture("amd64");
binary.set_priority(Priority::Optional);
binary.set_description("Hello, world!");

assert_eq!(control.to_string(), r#"Source: hello
Section: rust

Package: hello
Architecture: amd64
Priority: optional
Description: Hello, world!
"#);
```

