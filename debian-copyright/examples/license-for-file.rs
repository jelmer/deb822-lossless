use debian_copyright::Copyright;
use std::path::Path;

pub const TEXT: &str = r#"Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Author: John Doe <john@example>
Upstream-Name: example
Source: https://example.com/example

Files: *
License: GPL-3+
Copyright: 2019 John Doe

Files: debian/*
License: GPL-3+
Copyright: 2019 Jane Packager

License: GPL-3+
 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.
"#;

pub fn main() {
    let c = TEXT.parse::<Copyright>().unwrap();
    let license = c.find_license_for_file(Path::new("debian/foo")).unwrap();
    println!("{}", license.name().unwrap());
}
