/// A library for parsing and manipulating R DESCRIPTION files.
///
/// See https://r-pkgs.org/description.html and https://cran.r-project.org/doc/manuals/R-exts.html
/// for more information
///
/// See the ``lossless`` module for a lossless parser that is
/// forgiving in the face of errors and preserves formatting while editing
/// at the expense of a more complex API.
use deb822_lossless::{FromDeb822, FromDeb822Paragraph, ToDeb822, ToDeb822Paragraph};

use crate::RCode;
use std::iter::Peekable;

use crate::relations::SyntaxKind::*;
use crate::relations::{lex, SyntaxKind, VersionConstraint};
use crate::version::Version;

fn serialize_url_list(urls: &[url::Url]) -> String {
    let mut s = String::new();
    for (i, url) in urls.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(url.as_str());
    }
    s
}

fn deserialize_url_list(s: &str) -> Result<Vec<url::Url>, String> {
    s.split(',')
        .map(|s| url::Url::parse(s.trim()))
        .collect::<Result<Vec<_>, url::ParseError>>()
        .map_err(|e| e.to_string())
}

#[derive(FromDeb822, ToDeb822, Debug, PartialEq, Eq)]
pub struct RDescription {
    #[deb822(field = "Package")]
    pub name: String,

    #[deb822(field = "Description")]
    pub description: String,

    #[deb822(field = "Title")]
    pub title: String,

    #[deb822(field = "Maintainer")]
    pub maintainer: Option<String>,

    #[deb822(field = "Author")]
    /// Who wrote the the package
    pub author: Option<String>,

    // 'Authors@R' is a special field that can contain R code
    // that is evaluated to get the authors and maintainers.
    #[deb822(field = "Authors@R")]
    pub authors: Option<RCode>,

    #[deb822(field = "Version")]
    pub version: String,

    /// If the DESCRIPTION file is not written in pure ASCII, the encoding
    /// field must be used to specify the encoding.
    #[deb822(field = "Encoding")]
    pub encoding: Option<String>,

    #[deb822(field = "License")]
    pub license: String,

    #[deb822(field = "URL", serialize_with = serialize_url_list, deserialize_with = deserialize_url_list)]
    // TODO: parse this as a list of URLs, separated by commas
    pub url: Option<Vec<url::Url>>,

    #[deb822(field = "BugReports")]
    pub bug_reports: Option<String>,

    #[deb822(field = "Imports")]
    pub imports: Option<Relations>,

    #[deb822(field = "Suggests")]
    pub suggests: Option<Relations>,

    #[deb822(field = "Depends")]
    pub depends: Option<Relations>,

    #[deb822(field = "LinkingTo")]
    pub linking_to: Option<Relations>,

    #[deb822(field = "LazyData")]
    pub lazy_data: Option<String>,

    #[deb822(field = "Collate")]
    pub collate: Option<String>,

    #[deb822(field = "VignetteBuilder")]
    pub vignette_builder: Option<String>,

    #[deb822(field = "SystemRequirements")]
    pub system_requirements: Option<String>,

    #[deb822(field = "Date")]
    /// The release date of the current version of the package.
    /// Strongly recommended to use the ISO 8601 format: YYYY-MM-DD
    pub date: Option<String>,

    #[deb822(field = "Language")]
    /// Indicates the package documentation is not in English.
    /// This should be a comma-separated list of IETF language
    /// tags as defined by RFC5646
    pub language: Option<String>,
}

/// A relation entry in a relationship field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    /// Package name.
    pub name: String,
    /// Version constraint and version.
    pub version: Option<(VersionConstraint, Version)>,
}

impl Default for Relation {
    fn default() -> Self {
        Self::new()
    }
}

impl Relation {
    /// Create an empty relation.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            version: None,
        }
    }

    /// Check if this entry is satisfied by the given package versions.
    ///
    /// # Arguments
    /// * `package_version` - A function that returns the version of a package.
    ///
    /// # Example
    /// ```
    /// use r_description::lossy::Relation;
    /// use r_description::version::Version;
    /// let entry: Relation = "cli (>= 2.0)".parse().unwrap();
    /// assert!(entry.satisfied_by(|name: &str| -> Option<Version> {
    ///    match name {
    ///    "cli" => Some("2.0".parse().unwrap()),
    ///    _ => None
    /// }}));
    /// ```
    pub fn satisfied_by(&self, package_version: impl crate::relations::VersionLookup) -> bool {
        let actual = package_version.lookup_version(self.name.as_str());
        if let Some((vc, version)) = &self.version {
            if let Some(actual) = actual {
                match vc {
                    VersionConstraint::GreaterThanEqual => actual.as_ref() >= version,
                    VersionConstraint::LessThanEqual => actual.as_ref() <= version,
                    VersionConstraint::Equal => actual.as_ref() == version,
                    VersionConstraint::GreaterThan => actual.as_ref() > version,
                    VersionConstraint::LessThan => actual.as_ref() < version,
                }
            } else {
                false
            }
        } else {
            actual.is_some()
        }
    }
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some((constraint, version)) = &self.version {
            write!(f, " ({} {})", constraint, version)?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Relation, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Relation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

/// A collection of relation entries in a relationship field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relations(pub Vec<Relation>);

impl std::ops::Index<usize> for Relations {
    type Output = Relation;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Relations {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl FromIterator<Relation> for Relations {
    fn from_iter<I: IntoIterator<Item = Relation>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Default for Relations {
    fn default() -> Self {
        Self::new()
    }
}

impl Relations {
    /// Create an empty relations.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Remove an entry from the relations.
    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    /// Iterate over the entries in the relations.
    pub fn iter(&self) -> impl Iterator<Item = &Relation> {
        self.0.iter()
    }

    /// Number of entries in the relations.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the relations are empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if the relations are satisfied by the given package versions.
    pub fn satisfied_by(
        &self,
        package_version: impl crate::relations::VersionLookup + Copy,
    ) -> bool {
        self.0.iter().all(|r| r.satisfied_by(package_version))
    }
}

impl std::fmt::Display for Relations {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, relation) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", relation)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Relation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = lex(s);
        let mut tokens = tokens.into_iter().peekable();

        fn eat_whitespace(tokens: &mut Peekable<impl Iterator<Item = (SyntaxKind, String)>>) {
            while let Some((k, _)) = tokens.peek() {
                match k {
                    WHITESPACE | NEWLINE => {
                        tokens.next();
                    }
                    _ => break,
                }
            }
        }

        let name = match tokens.next() {
            Some((IDENT, name)) => name,
            _ => return Err("Expected package name".to_string()),
        };

        eat_whitespace(&mut tokens);

        let version = if let Some((L_PARENS, _)) = tokens.peek() {
            tokens.next();
            eat_whitespace(&mut tokens);
            let mut constraint = String::new();
            while let Some((kind, t)) = tokens.peek() {
                match kind {
                    EQUAL | L_ANGLE | R_ANGLE => {
                        constraint.push_str(t);
                        tokens.next();
                    }
                    _ => break,
                }
            }
            let constraint = constraint.parse()?;
            eat_whitespace(&mut tokens);
            // Read IDENT and COLON tokens until we see R_PARENS
            let version_string = match tokens.next() {
                Some((IDENT, s)) => s,
                _ => return Err("Expected version string".to_string()),
            };
            let version: Version = version_string.parse().map_err(|e: String| e.to_string())?;
            eat_whitespace(&mut tokens);
            if let Some((R_PARENS, _)) = tokens.next() {
            } else {
                return Err(format!("Expected ')', found {:?}", tokens.next()));
            }
            Some((constraint, version))
        } else {
            None
        };

        eat_whitespace(&mut tokens);

        if let Some((kind, _)) = tokens.next() {
            return Err(format!("Unexpected token: {:?}", kind));
        }

        Ok(Relation { name, version })
    }
}

impl std::str::FromStr for Relations {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut relations = Vec::new();
        if s.is_empty() {
            return Ok(Relations(relations));
        }
        for relation in s.split(',') {
            let relation = relation.trim();
            if relation.is_empty() {
                // Ignore empty entries.
                continue;
            }
            relations.push(relation.parse()?);
        }
        Ok(Relations(relations))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Relations {
    fn deserialize<D>(deserializer: D) -> Result<Relations, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Relations {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl std::str::FromStr for RDescription {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let para: deb822_lossless::Paragraph = s
            .parse()
            .map_err(|e: deb822_lossless::ParseError| e.to_string())?;
        Self::from_paragraph(&para)
    }
}

impl std::fmt::Display for RDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.to_paragraph().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r###"Package: mypackage
Title: What the Package Does (One Line, Title Case)
Version: 0.0.0.9000
Authors@R: 
    person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
           comment = c(ORCID = "YOUR-ORCID-ID"))
Description: What the package does (one paragraph).
License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a
    license
Encoding: UTF-8
Roxygen: list(markdown = TRUE)
RoxygenNote: 7.3.2
"###;
        let desc: RDescription = s.parse().unwrap();

        assert_eq!(desc.name, "mypackage".to_string());
        assert_eq!(
            desc.title,
            "What the Package Does (One Line, Title Case)".to_string()
        );
        assert_eq!(desc.version, "0.0.0.9000".to_string());
        assert_eq!(
            desc.authors,
            Some(RCode(
                r#"person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
comment = c(ORCID = "YOUR-ORCID-ID"))"#
                    .to_string()
            ))
        );
        assert_eq!(
            desc.description,
            "What the package does (one paragraph).".to_string()
        );
        assert_eq!(
            desc.license,
            "`use_mit_license()`, `use_gpl3_license()` or friends to pick a\nlicense".to_string()
        );
        assert_eq!(desc.encoding, Some("UTF-8".to_string()));

        assert_eq!(
            desc.to_string(),
            r###"Package: mypackage
Description: What the package does (one paragraph).
Title: What the Package Does (One Line, Title Case)
Authors@R: person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
 comment = c(ORCID = "YOUR-ORCID-ID"))
Version: 0.0.0.9000
Encoding: UTF-8
License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a
 license
"###
        );
    }

    #[test]
    fn test_parse_dplyr() {
        let s = include_str!("../testdata/dplyr.desc");
        let desc: RDescription = s.parse().unwrap();

        assert_eq!(desc.name, "dplyr".to_string());
    }

    #[test]
    fn test_parse_relations() {
        let input = "cli";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 1);
        let relation = &parsed[0];
        assert_eq!(relation.to_string(), "cli");
        assert_eq!(relation.version, None);

        let input = "cli (>= 0.20.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 1);
        let relation = &parsed[0];
        assert_eq!(relation.to_string(), "cli (>= 0.20.21)");
        assert_eq!(
            relation.version,
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.20.21".parse().unwrap()
            ))
        );
    }

    #[test]
    fn test_multiple() {
        let input = "cli (>= 0.20.21), cli (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 2);
        let relation = &parsed[0];
        assert_eq!(relation.to_string(), "cli (>= 0.20.21)");
        assert_eq!(
            relation.version,
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.20.21".parse().unwrap()
            ))
        );
        let relation = &parsed[1];
        assert_eq!(relation.to_string(), "cli (<< 0.21)");
        assert_eq!(
            relation.version,
            Some((VersionConstraint::LessThan, "0.21".parse().unwrap()))
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_relations() {
        let input = "cli (>= 0.20.21), cli (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();
        assert_eq!(serialized, r#""cli (>= 0.20.21), cli (<< 0.21)""#);
        let deserialized: Relations = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, parsed);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_relation() {
        let input = "cli (>= 0.20.21)";
        let parsed: Relation = input.parse().unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();
        assert_eq!(serialized, r#""cli (>= 0.20.21)""#);
        let deserialized: Relation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, parsed);
    }

    #[test]
    fn test_relations_is_empty() {
        let input = "cli (>= 0.20.21)";
        let parsed: Relations = input.parse().unwrap();
        assert!(!parsed.is_empty());
        let input = "";
        let parsed: Relations = input.parse().unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_relations_len() {
        let input = "cli (>= 0.20.21), cli (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_relations_remove() {
        let input = "cli (>= 0.20.21), cli (<< 0.21)";
        let mut parsed: Relations = input.parse().unwrap();
        parsed.remove(1);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.to_string(), "cli (>= 0.20.21)");
    }

    #[test]
    fn test_relations_satisfied_by() {
        let input = "cli (>= 0.20.21), cli (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert!(parsed.satisfied_by(|name: &str| -> Option<Version> {
            match name {
                "cli" => Some("0.20.21".parse().unwrap()),
                _ => None,
            }
        }));
        assert!(!parsed.satisfied_by(|name: &str| -> Option<Version> {
            match name {
                "cli" => Some("0.21".parse().unwrap()),
                _ => None,
            }
        }));
    }

    #[test]
    fn test_relation_satisfied_by() {
        let input = "cli (>= 0.20.21)";
        let parsed: Relation = input.parse().unwrap();
        assert!(parsed.satisfied_by(|name: &str| -> Option<Version> {
            match name {
                "cli" => Some("0.20.21".parse().unwrap()),
                _ => None,
            }
        }));
        assert!(!parsed.satisfied_by(|name: &str| -> Option<Version> {
            match name {
                "cli" => Some("0.20.20".parse().unwrap()),
                _ => None,
            }
        }));
    }
}
