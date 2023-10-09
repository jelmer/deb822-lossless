Lossless parser for deb822 style files
======================================

This crate contains lossless parsers and editors for RFC822 style file as used
in Debian.

In addition, it has wrappers for a couple of common formats that are based
on RFC822 style files:

 * `control` - ``debian/control``
 * `copyright` - ``debian/copyright``
 * `relations`` - relationship fields
    (``Build-Depends``, ``Depends``, ``Provides``, etc) as used in e.g.  debian
    control files
