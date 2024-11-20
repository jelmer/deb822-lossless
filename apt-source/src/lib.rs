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

#[derive(Debug, Clone, PartialEq)]
/// Enumeration for fields like `By-Hash` which have third value of `force`
pub enum YesNoForce {
    /// True
    Yes,
    /// False
    No,
    /// Forced
    Force
}


impl FromStr for YesNoForce {
    type Err = RepositoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yes" => Ok(Self::Yes),
            "no" => Ok(Self::No),
            "force" => Ok(Self::Force),
            _ => Err(RepositoryError::InvalidType)
        }
    }
}

impl From<&YesNoForce> for String {
    fn from(value: &YesNoForce) -> Self {
        match value {
            YesNoForce::Yes => "yes".to_owned(),
            YesNoForce::No => "no".to_owned(),
            YesNoForce::Force => "force".to_owned()
        }
    }
}

impl ToString for &YesNoForce {
    fn to_string(&self) -> String {
        self.to_owned().into()
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

fn deserialize_yesno(text: &str) -> Result<bool, String> { // TODO: bad error type
    match text {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err("Invalid value for yes/no field".to_owned())
    }
}

fn serializer_yesno(value: &bool) -> String {
    if *value {
        "yes".to_owned()
    } else {
        "no".to_owned()
    }
}

fn serialize_string_chain(chain: &[String]) -> String {
    chain.join(" ")
}

/// A structure representing APT repository as declared by DEB822 source file
/// 
/// According to `sources.list(5)` man pages, only four fields are mandatory:
/// * `Types` either `deb` or/and `deb-src`
/// * `URIs` to repositories holding valid APT structure (unclear if multiple are allowed)
/// * `Suites` usually being distribution codenames
/// * `Component` most of the time `main`, but it's a section of the repository
/// 
/// The manpage specifies following optional fields
/// * `Enabled`        is a yes/no field, default yes
/// * `Architectures`
/// * `Languages`
/// * `Targets`
/// * `PDiffs`         is a yes/no field
/// * `By-Hash`        is a yes/no/force field
/// * `Allow-Insecure` is a yes/no field, default no
/// * `Allow-Weak`     is a yes/no field, default no
/// * `Allow-Downgrade-To-Insecure` is a yes/no field, default no
/// * `Trusted`        us a yes/no field
/// * `Signed-By`      is either path to the key or PGP key block
/// * `Check-Valid-Until` is a yes/no field
/// * `Valid-Until-Min`
/// * `Valid-Until-Max`
/// * `Check-Date`     is a yes/no field
/// * `Date-Max-Future`
/// * `InRelease-Path` relative path
/// * `Snapshot`       either `enable` or a snapshot ID
/// 
/// The unit tests of APT use:
/// * `Description`
/// 
/// The RepoLib tool uses:
/// * `X-Repolib-Name` identifier for own reference, meaningless for APT
/// 
/// Note: Multivalues `*-Add` & `*-Remove` semantics aren't supported.
#[derive(FromDeb822, ToDeb822, Clone, PartialEq, /*Eq,*/ Debug, Default)]
pub struct Repository {
    /// If `no` (false) the repository is ignored by APT
    #[deb822(field = "Enabled", deserialize_with = deserialize_yesno, serialize_with = serializer_yesno)] // TODO: support for `default` if omitted is missing
    enabled: Option<bool>,

    /// The value `RepositoryType::Binary` (`deb`) or/and `RepositoryType::Source` (`deb-src`)
    #[deb822(field = "Types", deserialize_with = deserialize_types, serialize_with = serialize_types)]
    types: HashSet<RepositoryType>, // consider alternative, closed set
    /// The address of the repository
    #[deb822(field = "URIs", deserialize_with = deserialize_uris, serialize_with = serialize_uris)]
    uris: Vec<Url>, // according to Debian that's URI, but this type is more advanced than URI from `http` crate
    /// The distribution name as codename or suite type (like `stable` or `testing`)
    #[deb822(field = "Suites", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    suites: Vec<String>,
    /// Section of the repository, usually `main`, `contrib` or `non-free`
    #[deb822(field = "Components", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    components: Vec<String>,

    /// (Optional) Architectures binaries from this repository run on
    #[deb822(field = "Architectures", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    architectures: Vec<String>,
    /// (Optional) Translations support to download
    #[deb822(field = "Languages", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    languages: Option<Vec<String>>, // TODO: Option is redundant to empty vectors
    /// (Optional) Download targets to acquire from this source
    #[deb822(field = "Targets", deserialize_with = deserialize_string_chain, serialize_with = serialize_string_chain)]
    targets: Option<Vec<String>>,
    /// (Optional) Controls if APT should try PDiffs instead of downloading indexes entirely; if not set defaults to configuration option `Acquire::PDiffs`
    #[deb822(field = "PDiffs", deserialize_with = deserialize_yesno)]
    pdiffs: Option<bool>,
    /// (Optional) Controls if APT should try to acquire indexes via a URI constructed from a hashsum of the expected file
    #[deb822(field = "By-Hash")]
    by_hash: Option<YesNoForce>,
    /// (Optional) If yes circumvents parts of `apt-secure`, don't thread lightly
    #[deb822(field = "Allow-Insecure")]
    allow_insecure: Option<bool>, // TODO: redundant option, not present = default no
    /// (Optional) If yes circumvents parts of `apt-secure`, don't thread lightly
    #[deb822(field = "Allow-Weak")]
    allow_weak: Option<bool>, // TODO: redundant option, not present = default no
    /// (Optional) If yes circumvents parts of `apt-secure`, don't thread lightly
    #[deb822(field = "Allow-Downgrade-To-Insecure")]
    allow_downgrade_to_insecure: Option<bool>, // TODO: redundant option, not present = default no
    /// (Optional) If set forces whether APT considers source as rusted or no (default not present is a third state)
    #[deb822(field = "Trusted")]
    trusted: Option<bool>,
    /// (Optional) Contains either absolute path to GPG keyring or embedded GPG public key block, if not set APT uses all trusted keys;
    /// I can't find example of using with fingerprints
    #[deb822(field = "Signed-By")]
    signature: Option<Signature>,

    /// (Optional) Field ignored by APT but used by RepoLib to identify repositories, Ubuntu sources contain them
    #[deb822(field = "X-Repolib-Name")]
    x_repolib_name: Option<String>, // this supports RepoLib still used by PopOS, even if removed from Debian/Ubuntu

    /// (Optional) Field not present in the man page, but used in APT unit tests, potentially to hold the repository description
    #[deb822(field = "Description")]
    description: Option<String>

    // options: HashMap<String, String> // My original parser kept remaining optional fields in the hash map, is this right approach?
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
                languages: None,
                targets: None,
                pdiffs: None,
                ..Default::default()
            }
        ]);
        let text = repos.to_string();
        assert_eq!(text, indoc! {r#"
            Enabled: yes
            Types: deb
            URIs: https://deb.debian.org/debian
            Suites: jammy
            Components: main
            Architectures: arm64
        "#});
    }
}
