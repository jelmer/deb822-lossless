//! A library for parsing and manipulating debian/copyright files that
//! use the DEP-5 format.
//!
//! This library is intended to be used for manipulating debian/copyright
use deb822_lossless::{Deb822, Paragraph};
use std::path::Path;

pub const CURRENT_FORMAT: &str =
    "https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/";

pub const KNOWN_FORMATS: &[&str] = &[CURRENT_FORMAT];

pub struct Copyright(Deb822);

impl Copyright {
    pub fn new() -> Self {
        Copyright(Deb822::new())
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
}

impl Default for Copyright {
    fn default() -> Self {
        Copyright(Deb822::new())
    }
}

impl std::str::FromStr for Copyright {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Deb822::from_str(s)?))
    }
}

pub struct Header(Paragraph);

impl Header {
    pub fn format_string(&self) -> Option<String> {
        self.0
            .get("Format")
            .or_else(|| self.0.get("Format-Specification"))
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
}

pub struct LicenseParagraph(Paragraph);

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
