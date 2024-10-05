//! Parser for relationship fields like `Depends`, `Recommends`, etc.
//!
//! # Example
//! ```
//! use debian_control::lossy::{Relations, Relation};
//! use debian_control::relations::VersionConstraint;
//!
//! let mut relations: Relations = r"python3-dulwich (>= 0.19.0), python3-requests, python3-urllib3 (<< 1.26.0)".parse().unwrap();
//! assert_eq!(relations.to_string(), "python3-dulwich (>= 0.19.0), python3-requests, python3-urllib3 (<< 1.26.0)");
//! assert!(relations.satisfied_by(|name: &str| -> Option<debversion::Version> {
//!    match name {
//!    "python3-dulwich" => Some("0.19.0".parse().unwrap()),
//!    "python3-requests" => Some("2.25.1".parse().unwrap()),
//!    "python3-urllib3" => Some("1.25.11".parse().unwrap()),
//!    _ => None
//!    }}));
//! relations.remove(1);
//! relations[0][0].archqual = Some("amd64".to_string());
//! assert_eq!(relations.to_string(), "python3-dulwich:amd64 (>= 0.19.0), python3-urllib3 (<< 1.26.0)");
//! ```

use std::iter::Peekable;

use crate::relations::SyntaxKind::*;
use crate::relations::{lex, BuildProfile, SyntaxKind, VersionConstraint};

/// A relation entry in a relationship field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    /// Package name.
    pub name: String,
    /// Architecture qualifier.
    pub archqual: Option<String>,
    /// Architectures that this relation is only valid for.
    pub architectures: Option<Vec<String>>,
    /// Version constraint and version.
    pub version: Option<(VersionConstraint, debversion::Version)>,
    /// Build profiles that this relation is only valid for.
    pub profiles: Vec<Vec<BuildProfile>>,
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
            archqual: None,
            architectures: None,
            version: None,
            profiles: Vec::new(),
        }
    }

    /// Check if this entry is satisfied by the given package versions.
    ///
    /// # Arguments
    /// * `package_version` - A function that returns the version of a package.
    ///
    /// # Example
    /// ```
    /// use debian_control::lossy::Relation;
    /// let entry: Relation = "samba (>= 2.0)".parse().unwrap();
    /// assert!(entry.satisfied_by(|name: &str| -> Option<debversion::Version> {
    ///    match name {
    ///    "samba" => Some("2.0".parse().unwrap()),
    ///    _ => None
    /// }}));
    /// ```
    pub fn satisfied_by(&self, package_version: impl crate::VersionLookup) -> bool {
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
        if let Some(archqual) = &self.archqual {
            write!(f, ":{}", archqual)?;
        }
        if let Some((constraint, version)) = &self.version {
            write!(f, " ({} {})", constraint, version)?;
        }
        if let Some(archs) = &self.architectures {
            write!(f, " [{}]", archs.join(" "))?;
        }
        for profile in &self.profiles {
            write!(f, " <")?;
            for (i, profile) in profile.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", profile)?;
            }
            write!(f, ">")?;
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
pub struct Relations(pub Vec<Vec<Relation>>);

impl std::ops::Index<usize> for Relations {
    type Output = Vec<Relation>;

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
        Self(vec![iter.into_iter().collect()])
    }
}

impl FromIterator<Vec<Relation>> for Relations {
    fn from_iter<I: IntoIterator<Item = Vec<Relation>>>(iter: I) -> Self {
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
    pub fn iter(&self) -> impl Iterator<Item = Vec<&Relation>> {
        self.0.iter().map(|entry| entry.iter().collect())
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
    pub fn satisfied_by(&self, package_version: impl crate::VersionLookup + Copy) -> bool {
        self.0
            .iter()
            .all(|e| e.iter().any(|r| r.satisfied_by(package_version)))
    }
}

impl std::fmt::Display for Relations {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, entry) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            for (j, relation) in entry.iter().enumerate() {
                if j > 0 {
                    f.write_str(" | ")?;
                }
                write!(f, "{}", relation)?;
            }
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
            while let Some((WHITESPACE, _)) = tokens.peek() {
                tokens.next();
            }
        }

        let name = match tokens.next() {
            Some((IDENT, name)) => name,
            _ => return Err("Expected package name".to_string()),
        };

        eat_whitespace(&mut tokens);

        let archqual = if let Some((COLON, _)) = tokens.peek() {
            tokens.next();
            match tokens.next() {
                Some((IDENT, s)) => Some(s),
                _ => return Err("Expected architecture qualifier".to_string()),
            }
        } else {
            None
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
            let mut version_string = String::new();
            while let Some((kind, s)) = tokens.peek() {
                match kind {
                    R_PARENS => break,
                    IDENT | COLON => version_string.push_str(s),
                    n => return Err(format!("Unexpected token: {:?}", n)),
                }
                tokens.next();
            }
            let version = version_string
                .parse()
                .map_err(|e: debversion::ParseError| e.to_string())?;
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

        let architectures = if let Some((L_BRACKET, _)) = tokens.peek() {
            tokens.next();
            let mut archs = Vec::new();
            loop {
                match tokens.next() {
                    Some((IDENT, s)) => archs.push(s),
                    Some((WHITESPACE, _)) => {}
                    Some((R_BRACKET, _)) => break,
                    _ => return Err("Expected architecture name".to_string()),
                }
            }
            Some(archs)
        } else {
            None
        };

        eat_whitespace(&mut tokens);

        let mut profiles = Vec::new();
        while let Some((L_ANGLE, _)) = tokens.peek() {
            tokens.next();
            loop {
                let mut profile = Vec::new();
                loop {
                    match tokens.next() {
                        Some((NOT, _)) => {
                            let profile_name = match tokens.next() {
                                Some((IDENT, s)) => s,
                                _ => return Err("Expected profile name".to_string()),
                            };
                            profile.push(BuildProfile::Disabled(profile_name));
                        }
                        Some((IDENT, s)) => profile.push(BuildProfile::Enabled(s)),
                        Some((WHITESPACE, _)) => {}
                        _ => return Err("Expected profile name".to_string()),
                    }
                    if let Some((COMMA, _)) = tokens.peek() {
                        tokens.next();
                    } else {
                        break;
                    }
                }
                profiles.push(profile);
                if let Some((R_ANGLE, _)) = tokens.next() {
                    eat_whitespace(&mut tokens);
                    break;
                }
            }
        }

        eat_whitespace(&mut tokens);

        if let Some((kind, _)) = tokens.next() {
            return Err(format!("Unexpected token: {:?}", kind));
        }

        Ok(Relation {
            name,
            archqual,
            architectures,
            version,
            profiles,
        })
    }
}

impl std::str::FromStr for Relations {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut relations = Vec::new();
        if s.is_empty() {
            return Ok(Relations(relations));
        }
        for entry in s.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                // Ignore empty entries.
                continue;
            }
            let entry_relations = entry.split('|').map(|relation| {
                let relation = relation.trim();
                if relation.is_empty() {
                    return Err("Empty relation".to_string());
                }
                relation.parse()
            });
            relations.push(entry_relations.collect::<Result<Vec<_>, _>>()?);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "python3-dulwich";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 1);
        let entry = &parsed[0];
        assert_eq!(entry.len(), 1);
        let relation = &entry[0];
        assert_eq!(relation.to_string(), "python3-dulwich");
        assert_eq!(relation.version, None);

        let input = "python3-dulwich (>= 0.20.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 1);
        let entry = &parsed[0];
        assert_eq!(entry.len(), 1);
        let relation = &entry[0];
        assert_eq!(relation.to_string(), "python3-dulwich (>= 0.20.21)");
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
        let input = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 2);
        let entry = &parsed[0];
        assert_eq!(entry.len(), 1);
        let relation = &entry[0];
        assert_eq!(relation.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(
            relation.version,
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.20.21".parse().unwrap()
            ))
        );
        let entry = &parsed[1];
        assert_eq!(entry.len(), 1);
        let relation = &entry[0];
        assert_eq!(relation.to_string(), "python3-dulwich (<< 0.21)");
        assert_eq!(
            relation.version,
            Some((VersionConstraint::LessThan, "0.21".parse().unwrap()))
        );
    }

    #[test]
    fn test_architectures() {
        let input = "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.len(), 1);
        let entry = &parsed[0];
        assert_eq!(
            entry[0].to_string(),
            "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]"
        );
        assert_eq!(entry.len(), 1);
        let relation = &entry[0];
        assert_eq!(
            relation.to_string(),
            "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]"
        );
        assert_eq!(relation.version, None);
        assert_eq!(
            relation.architectures.as_ref().unwrap(),
            &vec![
                "amd64", "arm64", "armhf", "i386", "mips", "mips64el", "mipsel", "ppc64el", "s390x"
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_profiles() {
        let input = "foo (>= 1.0) [i386 arm] <!nocheck> <!cross>, bar";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.iter().count(), 2);
        let entry = parsed.iter().next().unwrap();
        assert_eq!(
            entry[0].to_string(),
            "foo (>= 1.0) [i386 arm] <!nocheck> <!cross>"
        );
        assert_eq!(entry.len(), 1);
        let relation = entry[0];
        assert_eq!(
            relation.to_string(),
            "foo (>= 1.0) [i386 arm] <!nocheck> <!cross>"
        );
        assert_eq!(
            relation.version,
            Some((VersionConstraint::GreaterThanEqual, "1.0".parse().unwrap()))
        );
        assert_eq!(
            relation.architectures.as_ref().unwrap(),
            &["i386", "arm"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            relation.profiles,
            vec![
                vec![BuildProfile::Disabled("nocheck".to_string())],
                vec![BuildProfile::Disabled("cross".to_string())]
            ]
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_relations() {
        let input = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();
        assert_eq!(
            serialized,
            r#""python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)""#
        );
        let deserialized: Relations = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, parsed);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_relation() {
        let input = "python3-dulwich (>= 0.20.21)";
        let parsed: Relation = input.parse().unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();
        assert_eq!(serialized, r#""python3-dulwich (>= 0.20.21)""#);
        let deserialized: Relation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, parsed);
    }

    #[test]
    fn test_relations_is_empty() {
        let input = "python3-dulwich (>= 0.20.21)";
        let parsed: Relations = input.parse().unwrap();
        assert!(!parsed.is_empty());
        let input = "";
        let parsed: Relations = input.parse().unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_relations_len() {
        let input = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_relations_remove() {
        let input = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)";
        let mut parsed: Relations = input.parse().unwrap();
        parsed.remove(1);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.to_string(), "python3-dulwich (>= 0.20.21)");
    }

    #[test]
    fn test_relations_satisfied_by() {
        let input = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)";
        let parsed: Relations = input.parse().unwrap();
        assert!(
            parsed.satisfied_by(|name: &str| -> Option<debversion::Version> {
                match name {
                    "python3-dulwich" => Some("0.20.21".parse().unwrap()),
                    _ => None,
                }
            })
        );
        assert!(
            !parsed.satisfied_by(|name: &str| -> Option<debversion::Version> {
                match name {
                    "python3-dulwich" => Some("0.21".parse().unwrap()),
                    _ => None,
                }
            })
        );
    }

    #[test]
    fn test_relation_satisfied_by() {
        let input = "python3-dulwich (>= 0.20.21)";
        let parsed: Relation = input.parse().unwrap();
        assert!(
            parsed.satisfied_by(|name: &str| -> Option<debversion::Version> {
                match name {
                    "python3-dulwich" => Some("0.20.21".parse().unwrap()),
                    _ => None,
                }
            })
        );
        assert!(
            !parsed.satisfied_by(|name: &str| -> Option<debversion::Version> {
                match name {
                    "python3-dulwich" => Some("0.20.20".parse().unwrap()),
                    _ => None,
                }
            })
        );
    }
}
