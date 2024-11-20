#![deny(missing_docs)]
//! A library for parsing and manipulating APT source files that
//! use the DEB822 format to hold package repositories specifications.
//!
//! # Examples
//!
//! ```rust
//!
//! use apt_source::Repositories;
//! use std::path::Path;
//!
//! let text = r#"Types: deb
//! URIs: http://ports.ubuntu.com/
//! Suites: noble
//! Components: stable
//! Architectures: arm64
//! Signed-By:
//!  -----BEGIN PGP PUBLIC KEY BLOCK-----
//!  .
//!  mDMEY865UxYJKwYBBAHaRw8BAQdAd7Z0srwuhlB6JKFkcf4HU4SSS/xcRfwEQWzr
//!  crf6AEq0SURlYmlhbiBTdGFibGUgUmVsZWFzZSBLZXkgKDEyL2Jvb2t3b3JtKSA8
//!  ZGViaWFuLXJlbGVhc2VAbGlzdHMuZGViaWFuLm9yZz6IlgQTFggAPhYhBE1k/sEZ
//!  wgKQZ9bnkfjSWFuHg9SBBQJjzrlTAhsDBQkPCZwABQsJCAcCBhUKCQgLAgQWAgMB
//!  Ah4BAheAAAoJEPjSWFuHg9SBSgwBAP9qpeO5z1s5m4D4z3TcqDo1wez6DNya27QW
//!  WoG/4oBsAQCEN8Z00DXagPHbwrvsY2t9BCsT+PgnSn9biobwX7bDDg==
//!  =5NZE
//!  -----END PGP PUBLIC KEY BLOCK-----"#;
//!
//! let r = text.parse::<Repositories>().unwrap();
//! let suites = r[0].suites();
//! assert_eq!(suites[0], "noble");
//! ```
//!
//! See the ``lossless`` module (behind the ``lossless`` feature) for a more forgiving parser that
//! allows partial parsing, parsing files with errors and unknown fields and editing while
//! preserving formatting.

use deb822_lossless::{FromDeb822, FromDeb822Paragraph, ToDeb822, ToDeb822Paragraph};
use signature::Signature;
use std::{collections::HashSet, ops::Deref, str::FromStr};
use url::Url;
use std::result::Result;
use error::RepositoryError;

pub mod error;
pub mod signature;

/// A representation of the repository type, by role of packages it can provide, either `Binary`
/// (indicated by `deb`) or `Source` (indicated by `deb-src`).
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum RepositoryType {
    /// Repository with binary packages, indicated as `deb`
    Binary,
    /// Repository with source packages, indicated as `deb-src`
    Source
}

impl FromStr for RepositoryType {
    type Err = RepositoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deb" => Ok(RepositoryType::Binary),
            "deb-src" => Ok(RepositoryType::Source),
            _ => Err(RepositoryError::InvalidType)
        }
    }
}

impl From<&RepositoryType> for String {
    fn from(value: &RepositoryType) -> Self {
        match value {
            RepositoryType::Binary => "deb".to_owned(),
            RepositoryType::Source => "deb-src".to_owned(),
        }
    }
}

impl ToString for RepositoryType {
    fn to_string(&self) -> String {
        self.into()
    }
}

fn deserialize_types(text: &str) -> Result<HashSet<RepositoryType>, RepositoryError> {
    text.split_whitespace()
        .map(|t| RepositoryType::from_str(t))
        .collect::<Result<HashSet<RepositoryType>, RepositoryError>>()
}

fn serialize_types(files: &HashSet<RepositoryType>) -> String {
    files.into_iter().map(|rt| rt.to_string()).collect::<Vec<String>>().join("\n")
}

fn deserialize_uris(text: &str) -> Result<Vec<Url>, String> { // TODO: bad error type
    text.split_whitespace()
        .map(|u| Url::from_str(u))
        .collect::<Result<Vec<Url>, _>>()
        .map_err(|e| e.to_string()) // TODO: bad error type
}

fn serialize_uris(uris: &[Url]) -> String {
    uris.into_iter().map(|u| u.as_str()).collect::<Vec<&str>>().join(" ")
}

fn deserialize_string_chain(text: &str) -> Result<Vec<String>, String> { // TODO: bad error type
    Ok(text.split_whitespace()
        .map(|x| x.to_string())
        .collect())
}

fn serialize_string_chain(chain: &[String]) -> String {
    chain.join(" ")
}

/// A structure representing APT repository as declared by DEB822 source file
#[derive(FromDeb822, ToDeb822, Clone, PartialEq, /*Eq,*/ Debug)]
pub struct Repository {
    #[deb822(field = "Enabled")] // TODO: support for `default` if omitted is missing
    enabled: Option<bool>,
    #[deb822(field = "Types", deserialize_with = deserialize_types, serialize_with = serialize_types)]
    types: HashSet<RepositoryType>, // consider alternative, closed set
    #[deb822(field = "Architectures", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    architectures: Vec<String>,
    #[deb822(field = "URIs", deserialize_with = deserialize_uris, serialize_with = serialize_uris)]
    uris: Vec<Url>, // according to Debian that's URI, but this type is more advanced than URI from `http` crate
    #[deb822(field = "Suites", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    suites: Vec<String>,
    #[deb822(field = "Components", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    components: Vec<String>,
    #[deb822(field = "Signed-By")]
    signature: Option<Signature>,
    #[deb822(field = "X-Repolib-Name")]
    x_repolib_name: Option<String>, // this supports RepoLib still used by PopOS, even if removed from Debian/Ubuntu
    //options: HashMap<String, String>
}

impl Repository {
    /// Returns slice of strings containing suites for which this repository provides
    pub fn suites(&self) -> &[String] {
        self.suites.as_slice()
    }
    
}

/// Container for multiple `Repository` specifications as single `.sources` file may contain as per specification
#[derive(Debug)]
pub struct Repositories(Vec<Repository>);

impl Repositories {
    /// Creates empty container of repositories
    pub fn empty() -> Self {
        Repositories(Vec::new())
    }
    
    /// Creates repositories from container consisting `Repository` instances
    pub fn new<Container>(container: Container) -> Self
    where
        Container: Into<Vec<Repository>>
    {
        Repositories(container.into())
    }
}

impl std::str::FromStr for Repositories {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deb822: deb822_lossless::Deb822 = s
            .parse()
            .map_err(|e: deb822_lossless::ParseError| e.to_string())?;

        let repos = deb822.paragraphs().map(|p| Repository::from_paragraph(&p)).collect::<Result<Vec<Repository>, Self::Err>>()?;
        Ok(Repositories(repos))
    }
}

impl ToString for Repositories {
    fn to_string(&self) -> String {
        self.0.iter()
            .map(|r| { let p: deb822_lossless::lossy::Paragraph = r.to_paragraph(); p.to_string() })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Deref for Repositories {
    type Target = Vec<Repository>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, str::FromStr};

    use indoc::indoc;
    use url::Url;

    use crate::{signature::Signature, Repositories, Repository, RepositoryType};

    #[test]
    fn test_not_machine_readable() {
        let s = indoc!(r#"
            deb [arch=arm64 signed-by=/usr/share/keyrings/docker.gpg] http://ports.ubuntu.com/ noble stable
        "#);
        let ret = s.parse::<Repositories>();
        assert!(ret.is_err());
        //assert_eq!(ret.unwrap_err(), "Not machine readable".to_string());
        assert_eq!(ret.unwrap_err(), "expected ':', got Some(NEWLINE)\n".to_owned());
    }

    #[test]
    fn test_parse_w_keyblock() {
        let s = indoc!(r#"
            Types: deb
            URIs: http://ports.ubuntu.com/
            Suites: noble
            Components: stable
            Architectures: arm64
            Signed-By:
             -----BEGIN PGP PUBLIC KEY BLOCK-----
             .
             mDMEY865UxYJKwYBBAHaRw8BAQdAd7Z0srwuhlB6JKFkcf4HU4SSS/xcRfwEQWzr
             crf6AEq0SURlYmlhbiBTdGFibGUgUmVsZWFzZSBLZXkgKDEyL2Jvb2t3b3JtKSA8
             ZGViaWFuLXJlbGVhc2VAbGlzdHMuZGViaWFuLm9yZz6IlgQTFggAPhYhBE1k/sEZ
             wgKQZ9bnkfjSWFuHg9SBBQJjzrlTAhsDBQkPCZwABQsJCAcCBhUKCQgLAgQWAgMB
             Ah4BAheAAAoJEPjSWFuHg9SBSgwBAP9qpeO5z1s5m4D4z3TcqDo1wez6DNya27QW
             WoG/4oBsAQCEN8Z00DXagPHbwrvsY2t9BCsT+PgnSn9biobwX7bDDg==
             =5NZE
             -----END PGP PUBLIC KEY BLOCK-----
        "#);

        let repos = s.parse::<Repositories>().expect("Shall be parsed flawlessly");
        assert!(repos[0].types.contains(&super::RepositoryType::Binary));
        assert!(matches!(repos[0].signature, Some(Signature::KeyBlock(_))));
    }

    #[test]
    fn test_parse_w_keypath() {
        let s = indoc!(r#"
            Types: deb
            URIs: http://ports.ubuntu.com/
            Suites: noble
            Components: stable
            Architectures: arm64
            Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
        "#);

        let reps = s.parse::<Repositories>().expect("Shall be parsed flawlessly");
        assert!(reps[0].types.contains(&super::RepositoryType::Binary));
        assert!(matches!(reps[0].signature, Some(Signature::KeyPath(_))));
    }

    #[test]
    fn test_serialize() {
        //let repos = Repositories::empty();
        let repos = Repositories::new([
            Repository {
                enabled: Some(true), // TODO: looks odd, as only `Enabled: no` in meaningful
                types: HashSet::from([RepositoryType::Binary]),
                architectures: vec!["arm64".to_owned()],
                uris: vec![Url::from_str("https://deb.debian.org/debian").unwrap()],
                suites: vec!["jammy".to_owned()],
                components: vec!["main". to_owned()],
                signature: None,
                x_repolib_name: None,
            }
        ]);
        let text = repos.to_string();
        assert_eq!(text, indoc! {r#"
            Enabled: yes
            Types: deb
            Architectures: arm64
            URIs: https://deb/debian.org/debian
            Suites: jammy
            Components: main
        "#});
    }
}
