Parsers and editors for deb822 style files
==========================================

This repository contains parsers and editors for RFC822 style file as used in
Debian. There are two core crates:

* ``deb822-fast``: fast parser (with derive support) that does not preserve formatting or comments
* ``deb822-lossless``: parser that preserves formatting, whitespace and comments as well as allowing syntax errors.

Related crates that build on these two are:

* ``debian-control``: A parser and editor for Debian control files, apt lists.
* ``debian-copyright``: A parser and editor for Debian copyright files.
* ``dep3``: A parser and editor for Debian DEP-3 headers.
* [r-description](https://github.com/jelmer/r-description-rs): A parser and
editor for R DESCRIPTION files.
* ``apt-sources``: A parser and editor for APT source files (package repositories specification).
