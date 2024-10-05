//! Parser for Debian buildinfo files
//!
//! The buildinfo file format is a Debian-specific format that is used to store
//! information about the build environment of a package. See https://wiki.debian.org/Buildinfo for
//! more information.

use crate::fields::{Md5Checksum, Sha1Checksum, Sha256Checksum};
use crate::lossless::relations::Relations;

/// A buildinfo file
pub struct Buildinfo(deb822_lossless::Paragraph);

impl From<deb822_lossless::Paragraph> for Buildinfo {
    fn from(paragraph: deb822_lossless::Paragraph) -> Self {
        Self(paragraph)
    }
}

impl Default for Buildinfo {
    fn default() -> Self {
        let mut para = deb822_lossless::Paragraph::new();
        para.set("Format", "1.0");
        Self(para)
    }
}

impl Buildinfo {
    /// Create a new source package
    pub fn new() -> Self {
        Self(deb822_lossless::Paragraph::new())
    }

    /// Get the source name
    pub fn source(&self) -> Option<String> {
        self.0.get("Source").map(|s| s.to_string())
    }

    /// Set the package name
    pub fn set_source(&mut self, package: &str) {
        self.0.set("Source", package);
    }

    /// Get the binary package names
    pub fn binaries(&self) -> Option<Vec<String>> {
        self.0.get("Binary").map(|s| {
            s.split(' ')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
    }

    /// Set the binary package names
    pub fn set_binaries(&mut self, binaries: Vec<String>) {
        self.0.set("Binary", &binaries.join(" "));
    }

    /// Get the version of the package
    pub fn version(&self) -> Option<debversion::Version> {
        self.0.get("Version").map(|s| s.parse().unwrap())
    }

    /// Set the version of the package
    pub fn set_version(&mut self, version: debversion::Version) {
        self.0.set("Version", &version.to_string());
    }

    /// Get the build architecture
    pub fn build_architecture(&self) -> Option<String> {
        self.0.get("Build-Architecture").map(|s| s.to_string())
    }

    /// Set the build architecture
    pub fn set_build_architecture(&mut self, arch: &str) {
        self.0.set("Build-Architecture", arch);
    }

    /// Get the architecture
    pub fn architecture(&self) -> Option<String> {
        self.0.get("Architecture").map(|s| s.to_string())
    }

    /// Set the architecture
    pub fn set_architecture(&mut self, arch: &str) {
        self.0.set("Architecture", arch);
    }

    /// Get Sha256 checksums
    pub fn checksums_sha256(&self) -> Vec<Sha256Checksum> {
        self.0
            .get("Checksums-Sha256")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha256Checksum>>()
            })
            .unwrap_or_default()
    }

    /// Set Sha256 checksums
    pub fn set_checksums_sha256(&mut self, checksums: Vec<Sha256Checksum>) {
        self.0.set(
            "Checksums-Sha256",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    /// Get SHA1 checksums
    pub fn checksums_sha1(&self) -> Vec<Sha1Checksum> {
        self.0
            .get("Checksums-Sha1")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha1Checksum>>()
            })
            .unwrap_or_default()
    }

    /// Set SHA1 checksums
    pub fn set_checksums_sha1(&mut self, checksums: Vec<Sha1Checksum>) {
        self.0.set(
            "Checksums-Sha1",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    /// Get MD5 checksums
    pub fn checksums_md5(&self) -> Vec<Md5Checksum> {
        self.0
            .get("Checksums-Md5")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Md5Checksum>>()
            })
            .unwrap_or_default()
    }

    /// Set MD5 checksums
    pub fn set_checksums_md5(&mut self, checksums: Vec<Md5Checksum>) {
        self.0.set(
            "Checksums-Md5",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    /// Get the build origin
    pub fn build_origin(&self) -> Option<String> {
        self.0.get("Build-Origin").map(|s| s.to_string())
    }

    /// Set the build origin
    pub fn set_build_origin(&mut self, origin: &str) {
        self.0.set("Build-Origin", origin);
    }

    /// Date on which the package was built
    pub fn build_date(&self) -> Option<String> {
        self.0.get("Build-Date").map(|s| s.to_string())
    }

    /// Set the build date
    pub fn set_build_date(&mut self, date: &str) {
        self.0.set("Build-Date", date);
    }

    /// Get the build tainted by field list
    pub fn build_tainted_by(&self) -> Option<Vec<String>> {
        self.0
            .get("Build-Tainted-By")
            .map(|s| s.split(' ').map(|s| s.to_string()).collect())
    }

    /// Set the build tainted by field list
    pub fn set_build_tainted_by(&mut self, tainted_by: Vec<String>) {
        self.0.set("Build-Tainted-By", &tainted_by.join(" "));
    }

    /// Get the source format of the package
    pub fn format(&self) -> Option<String> {
        self.0.get("Format").map(|s| s.to_string())
    }

    /// Set the format of the package
    pub fn set_format(&mut self, format: &str) {
        self.0.set("Format", format);
    }

    /// Get the build path
    pub fn build_path(&self) -> Option<String> {
        self.0.get("Build-Path").map(|s| s.to_string())
    }

    /// Set the build path
    pub fn set_build_path(&mut self, path: &str) {
        self.0.set("Build-Path", path);
    }

    /// Get the build environment
    pub fn environment(&self) -> Option<std::collections::HashMap<String, String>> {
        self.0.get("Environment").map(|s| {
            s.lines()
                .map(|line| {
                    let (key, value) = line.split_once('=').unwrap();
                    (key.to_string(), value.to_string())
                })
                .collect()
        })
    }

    /// Set the build environment
    pub fn set_environment(&mut self, env: std::collections::HashMap<String, String>) {
        let mut s = String::new();
        for (key, value) in env {
            s.push_str(&format!("{}={}\n", key, value));
        }
        self.0.set("Environment", &s);
    }

    /// Get the list of installed build depends
    pub fn installed_build_depends(&self) -> Option<Relations> {
        self.0
            .get("Installed-Build-Depends")
            .map(|s| s.parse().unwrap())
    }

    /// Set the list of installed build depends
    pub fn set_installed_build_depends(&mut self, depends: Relations) {
        self.0.set("Installed-Build-Depends", &depends.to_string());
    }
}

impl std::str::FromStr for Buildinfo {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = include_str!("../../testdata/ruff.buildinfo");
        let buildinfo: Buildinfo = s.parse().unwrap();
        assert_eq!(buildinfo.format(), Some("1.0".to_string()));
    }
}
