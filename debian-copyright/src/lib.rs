//! A library for parsing and manipulating debian/copyright files that
//! use the DEP-5 format.
//!
//! This library is intended to be used for manipulating debian/copyright
//!
//! # Examples
//!
//! ```rust
//!
//! use debian_copyright::Copyright;
//! use std::path::Path;
//!
//! let text = r#"Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
//! Upstream-Author: John Doe <john@example>
//! Upstream-Name: example
//! Source: https://example.com/example
//!
//! Files: *
//! License: GPL-3+
//! Copyright: 2019 John Doe
//!
//! Files: debian/*
//! License: GPL-3+
//! Copyright: 2019 Jane Packager
//!
//! License: GPL-3+
//!  This program is free software: you can redistribute it and/or modify
//!  it under the terms of the GNU General Public License as published by
//!  the Free Software Foundation, either version 3 of the License, or
//!  (at your option) any later version.
//! "#;
//!
//! let c = text.parse::<Copyright>().unwrap();
//! let license = c.find_license_for_file(Path::new("debian/foo")).unwrap();
//! assert_eq!(license.name(), Some("GPL-3+"));
//! ```

use deb822_lossless::{Deb822, Paragraph};
use std::path::Path;

pub const CURRENT_FORMAT: &str =
    "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/";

pub const KNOWN_FORMATS: &[&str] = &[CURRENT_FORMAT];

pub enum License {
    Name(String),
    Text(String),
    Named(String, String),
}

impl License {
    pub fn name(&self) -> Option<&str> {
        match self {
            License::Name(name) => Some(name),
            License::Text(_) => None,
            License::Named(name, _) => Some(name),
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            License::Name(_) => None,
            License::Text(text) => Some(text),
            License::Named(_, text) => Some(text),
        }
    }
}

#[derive(Debug)]
pub struct Copyright(Deb822);

impl Copyright {
    pub fn new() -> Self {
        let mut deb822 = Deb822::new();
        let mut header = deb822.add_paragraph();
        header.insert("Format", CURRENT_FORMAT);
        Copyright(deb822)
    }

    pub fn empty() -> Self {
        Self(Deb822::new())
    }

    pub fn header(&self) -> Option<Header> {
        self.0.paragraphs().next().map(Header)
    }

    pub fn iter_files(&self) -> impl Iterator<Item = FilesParagraph> {
        self.0
            .paragraphs()
            .filter(|x| x.contains_key("Files"))
            .map(FilesParagraph)
    }

    pub fn iter_licenses(&self) -> impl Iterator<Item = LicenseParagraph> {
        self.0
            .paragraphs()
            .filter(|x| !x.contains_key("Files") && x.contains_key("License"))
            .map(LicenseParagraph)
    }

    /// Returns the Files paragraph for the given filename.
    ///
    /// Consistent with the specification, this returns the last paragraph
    /// that matches (which should be the most specific)
    pub fn find_files(&self, filename: &Path) -> Option<FilesParagraph> {
        self.iter_files().filter(|p| p.matches(filename)).last()
    }

    pub fn find_license_by_name(&self, name: &str) -> Option<License> {
        self.iter_licenses().find(|p| p.name().as_deref() == Some(name)).map(|x| x.into())
    }

    /// Returns the license for the given file.
    pub fn find_license_for_file(&self, filename: &Path) -> Option<License> {
        let files = self.find_files(filename)?;
        let license = files.license()?;
        if license.text().is_some() {
            return Some(license);
        }
        self.find_license_by_name(license.name()?)
    }

    pub fn from_str_relaxed(s: &str) -> Result<(Self, Vec<String>), Error> {
        if !s.starts_with("Format:") {
            return Err(Error::NotMachineReadable);
        }

        let (deb822, errors) = Deb822::from_str_relaxed(s);
        Ok((Self(deb822), errors))
    }

    pub fn from_file_relaxed<P: AsRef<Path>>(path: P) -> Result<(Self, Vec<String>), Error> {
        let text = std::fs::read_to_string(path)?;
        Self::from_str_relaxed(&text)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        use std::str::FromStr;
        Self::from_str(&text)
    }
}

#[derive(Debug)]
pub enum Error {
    ParseError(deb822_lossless::ParseError),
    IoError(std::io::Error),
    NotMachineReadable,
}

impl From<deb822_lossless::Error> for Error {
    fn from(e: deb822_lossless::Error) -> Self {
        match e {
            deb822_lossless::Error::ParseError(e) => Error::ParseError(e),
            deb822_lossless::Error::IoError(e) => Error::IoError(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<deb822_lossless::ParseError> for Error {
    fn from(e: deb822_lossless::ParseError) -> Self {
        Error::ParseError(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Error::ParseError(e) => write!(f, "parse error: {}", e),
            Error::NotMachineReadable => write!(f, "not machine readable"),
            Error::IoError(e) => write!(f, "io error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl Default for Copyright {
    fn default() -> Self {
        Copyright(Deb822::new())
    }
}

impl std::str::FromStr for Copyright {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("Format:") {
            return Err(Error::NotMachineReadable);
        }
        Ok(Self(Deb822::from_str(s)?))
    }
}

impl ToString for Copyright {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct Header(Paragraph);

impl Header {
    /// Returns the format string for this file.
    pub fn format_string(&self) -> Option<String> {
        self.0
            .get("Format")
            .or_else(|| self.0.get("Format-Specification"))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.0.get(key)
    }

    pub fn upstream_name(&self) -> Option<String> {
        self.0.get("Upstream-Name")
    }

    pub fn upstream_contact(&self) -> Option<String> {
        self.0.get("Upstream-Contact")
    }

    pub fn source(&self) -> Option<String> {
        self.0.get("Source")
    }

    pub fn files_excluded(&self) -> Option<Vec<String>> {
        self.0
            .get("Files-Excluded")
            .map(|x| x.split('\n').map(|x| x.to_string()).collect::<Vec<_>>())
    }

    pub fn fix(&mut self) {
        if self.0.contains_key("Format-Specification") {
            self.0.rename("Format-Specification", "Format");
        }

        if let Some(mut format) = self.0.get("Format") {
            if !format.ends_with('/') {
                format.push('/');
            }

            if let Some(rest) = format.strip_prefix("http:") {
                format = format!("https:{}", rest);
            }

            if KNOWN_FORMATS.contains(&format.as_str()) {
                format = CURRENT_FORMAT.to_string();
            }

            self.0.insert("Format", format.as_str());
        }
    }
}

pub struct FilesParagraph(Paragraph);

impl FilesParagraph {
    pub fn files(&self) -> Vec<String> {
        self.0
            .get("Files")
            .unwrap()
            .split_whitespace()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
    }

    pub fn matches(&self, filename: &std::path::Path) -> bool {
        self.files()
            .iter()
            .any(|f| glob_to_regex(f).is_match(filename.to_str().unwrap()))
    }

    pub fn copyright(&self) -> Vec<String> {
        self.0
            .get("Copyright")
            .unwrap_or_default()
            .split('\n')
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    }

    pub fn comment(&self) -> Option<String> {
        self.0.get("Comment")
    }

    pub fn license(&self) -> Option<License> {
        self.0.get("License").map(|x| {
            x.split_once('\n').map_or_else(
                || License::Name(x.to_string()),
                |(name, text)| {
                    if name.is_empty() {
                        License::Text(text.to_string())
                    } else {
                        License::Named(name.to_string(), text.to_string())
                    }
                }
            )
        })
    }
}

pub struct LicenseParagraph(Paragraph);

impl From<LicenseParagraph> for License {
    fn from(p: LicenseParagraph) -> Self {
        let x = p.0.get("License").unwrap();
        x.split_once('\n').map_or_else(
            || License::Name(x.to_string()),
            |(name, text)| {
                if name.is_empty() {
                    License::Text(text.to_string())
                } else {
                    License::Named(name.to_string(), text.to_string())
                }
            }
        )
    }
}

impl LicenseParagraph {
    pub fn comment(&self) -> Option<String> {
        self.0.get("Comment")
    }

    pub fn name(&self) -> Option<String> {
        self.0.get("License").and_then(|x| x.split_once('\n').map(|(name, _)| name.to_string()))
    }

    pub fn text(&self) -> Option<String> {
        self.0.get("License").and_then(|x| x.split_once('\n').map(|(_, text)| text.to_string()))
    }
}

fn glob_to_regex(glob: &str) -> regex::Regex {
    let mut it = glob.chars();
    let mut r = String::new();

    while let Some(c) = it.next() {
        r.push_str(
            match c {
                '*' => ".*".to_string(),
                '?' => ".".to_string(),
                '\\' => match it.next().unwrap() {
                    '?' | '*' | '\\' => regex::escape(c.to_string().as_str()),
                    x => {
                        panic!("invalid escape sequence: \\{}", x);
                    }
                },
                c => regex::escape(c.to_string().as_str()),
            }
            .as_str(),
        )
    }

    regex::Regex::new(r.as_str()).unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_not_machine_readable() {
        let s = r#"
This copyright file is not machine readable.
"#;
        let ret = s.parse::<super::Copyright>();
        assert!(ret.is_err());
        assert!(matches!(ret.unwrap_err(), super::Error::NotMachineReadable));
    }

    #[test]
    fn test_new() {
        let n = super::Copyright::new();
        assert_eq!(n.to_string().as_str(), "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n");
    }

    #[test]
    fn test_parse() {
        let s = r#"Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: foo
Upstream-Contact: Joe Bloggs <joe@example.com>
Source: https://example.com/foo

Files: *
Copyright:
  2020 Joe Bloggs <joe@example.com>
License: GPL-3+

Files: debian/*
Comment: Debian packaging is licensed under the GPL-3+.
Copyright: 2023 Jelmer Vernooij
License: GPL-3+

License: GPL-3+
 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.
"#;
        let copyright = s
            .parse::<super::Copyright>()
            .expect("failed to parse");

        assert_eq!(
            "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/",
            copyright.header().unwrap().format_string().unwrap()
        );
        assert_eq!("foo", copyright.header().unwrap().upstream_name().unwrap());
        assert_eq!(
            "Joe Bloggs <joe@example.com>", copyright.header().unwrap()
                .upstream_contact().unwrap());
        assert_eq!("https://example.com/foo", copyright.header().unwrap()
                .source().unwrap());

        let files = copyright.iter_files().collect::<Vec<_>>();
        assert_eq!(2, files.len());
        assert_eq!("*", files[0].files().join(" "));
        assert_eq!("debian/*", files[1].files().join(" "));
        assert_eq!(
            "Debian packaging is licensed under the GPL-3+.",
            files[1].comment().unwrap()
        );
        assert_eq!(
            vec!["2023 Jelmer Vernooij".to_string()],
            files[1].copyright()
        );
        assert_eq!("GPL-3+", files[1].license().unwrap().name().unwrap());
        assert_eq!(files[1].license().unwrap().text(), None);

        let licenses = copyright.iter_licenses().collect::<Vec<_>>();
        assert_eq!(1, licenses.len());
        assert_eq!("GPL-3+", licenses[0].name().unwrap());
        assert_eq!(
            "This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.",
            licenses[0].text().unwrap()
        );

        let upstream_files = copyright.find_files(std::path::Path::new("foo.c")).unwrap();
        assert_eq!(vec!["*"], upstream_files.files());

        let debian_files = copyright.find_files(std::path::Path::new("debian/foo.c")).unwrap();
        assert_eq!(vec!["debian/*"], debian_files.files());

        let gpl = copyright.find_license_by_name("GPL-3+");
        assert!(gpl.is_some());

        let gpl = copyright.find_license_for_file(
            std::path::Path::new("debian/foo.c"),
        );
        assert_eq!(gpl.unwrap().name().unwrap(), "GPL-3+");
    }
}
