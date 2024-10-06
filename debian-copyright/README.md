# Lossless parser for Debian Copyright (DEP5) files

This crate contains a lossless parser for Debian Copyright files that use the
[DEP-5](https://dep-team.pages.debian.net/deps/dep5/) file format.

Once parsed, the files can be introspected as well as changed before
written back to disk.

Example:

```rust
let copyright: debian_copyright::Copyright = r#"\
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: lintian-brush
Upstream-Contact: Jelmer Vernooĳ <jelmer@debian.org>
Source: https://salsa.debian.org/jelmer/lintian-brush

Files: *
Copyright:
 2018-2019 Jelmer Vernooĳ <jelmer@debian.org>
License: GPL-2+

Files: lintian_brush/systemd.py
Copyright: 2001, 2002, 2003 Python Software Foundation
           2004-2008 Paramjit Oberoi <param.cs.wisc.edu>
           2007 Tim Lauridsen <tla@rasmil.dk>
           2019 Jelmer Vernooij <jelmer@jelmer.uk>
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a
 copy of this software and associated documentation files (the "Software"),
 to deal in the Software without restriction, including without limitation
 the rights to use, copy, modify, merge, publish, distribute, sublicense,
 and/or sell copies of the Software, and to permit persons to whom the
 Software is furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in
 all copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 DEALINGS IN THE SOFTWARE.

License: GPL-2+
 On Debian systems, the full text of the GNU General Public License is available
 in /usr/share/common-licenses/GPL-2.
"#.parse().unwrap();

let header = copyright.header().unwrap();

assert_eq!(
    "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/",
    header.format_string().unwrap());
assert_eq!("MIT", copyright.find_license_for_file("lintian_brush/systemd.py").unwrap().name());
assert_eq!("GPL-2+", copyright.find_license_for_file("lintian_brush/__init__.py").unwrap().name());
```
