//! Parser for relationship fields like `Depends`, `Recommends`, etc.
//!
//! # Example
//! ```
//! use debian_control::relations::{Relations, Relation, VersionConstraint};
//!
//! let mut relations: Relations = r"python3-dulwich (>= 0.19.0), python3-requests, python3-urllib3 (<< 1.26.0)".parse().unwrap();
//! assert_eq!(relations.to_string(), "python3-dulwich (>= 0.19.0), python3-requests, python3-urllib3 (<< 1.26.0)");
//! assert!(relations.satisfied_by(&mut |name| {
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
use std::str::Chars;

/// Let's start with defining all kinds of tokens and
/// composite nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    IDENT = 0, // package name
    COLON,     // :
    PIPE,
    COMMA,      // ,
    L_PARENS,   // (
    R_PARENS,   // )
    L_BRACKET,  // [
    R_BRACKET,  // ]
    NOT,        // !
    L_ANGLE,    // <
    R_ANGLE,    // >
    EQUAL,      // =
    WHITESPACE, // whitespace
    NEWLINE,    // newline
    COMMENT,    // comments
    DOLLAR,     // $
    L_CURLY,
    R_CURLY,
    ERROR, // as well as errors

    // composite nodes
    ROOT,       // The entire file
    ENTRY,      // A single entry
    RELATION,   // An alternative in a dependency
    ARCHQUAL,   // An architecture qualifier
    VERSION,    // A version constraint
    CONSTRAINT, // (">=", "<=", "=", ">>", "<<")
    ARCHITECTURES,
    PROFILES,
    SUBSTVAR,
}

use SyntaxKind::*;

/// Convert our `SyntaxKind` into the rowan `SyntaxKind`.
#[cfg(feature = "lossless")]
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t' || c == '\r'
    }

    fn is_valid_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '+' || c == '~'
    }

    fn read_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(&c) = self.input.peek() {
            if predicate(c) {
                result.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        result
    }

    fn next_token(&mut self) -> Option<(SyntaxKind, String)> {
        if let Some(&c) = self.input.peek() {
            match c {
                ':' => {
                    self.input.next();
                    Some((SyntaxKind::COLON, ":".to_owned()))
                }
                '|' => {
                    self.input.next();
                    Some((SyntaxKind::PIPE, "|".to_owned()))
                }
                ',' => {
                    self.input.next();
                    Some((SyntaxKind::COMMA, ",".to_owned()))
                }
                '(' => {
                    self.input.next();
                    Some((SyntaxKind::L_PARENS, "(".to_owned()))
                }
                ')' => {
                    self.input.next();
                    Some((SyntaxKind::R_PARENS, ")".to_owned()))
                }
                '[' => {
                    self.input.next();
                    Some((SyntaxKind::L_BRACKET, "[".to_owned()))
                }
                ']' => {
                    self.input.next();
                    Some((SyntaxKind::R_BRACKET, "]".to_owned()))
                }
                '!' => {
                    self.input.next();
                    Some((SyntaxKind::NOT, "!".to_owned()))
                }
                '$' => {
                    self.input.next();
                    Some((SyntaxKind::DOLLAR, "$".to_owned()))
                }
                '{' => {
                    self.input.next();
                    Some((SyntaxKind::L_CURLY, "{".to_owned()))
                }
                '}' => {
                    self.input.next();
                    Some((SyntaxKind::R_CURLY, "}".to_owned()))
                }
                '<' => {
                    self.input.next();
                    Some((SyntaxKind::L_ANGLE, "<".to_owned()))
                }
                '>' => {
                    self.input.next();
                    Some((SyntaxKind::R_ANGLE, ">".to_owned()))
                }
                '=' => {
                    self.input.next();
                    Some((SyntaxKind::EQUAL, "=".to_owned()))
                }
                '\n' => {
                    self.input.next();
                    Some((SyntaxKind::NEWLINE, "\n".to_owned()))
                }
                _ if Self::is_whitespace(c) => {
                    let whitespace = self.read_while(Self::is_whitespace);
                    Some((SyntaxKind::WHITESPACE, whitespace))
                }
                '#' => {
                    self.input.next();
                    let comment = self.read_while(|c| c != '\n' && c != '\r');
                    Some((SyntaxKind::COMMENT, format!("#{}", comment)))
                }
                // TODO: separate handling for package names and versions?
                _ if Self::is_valid_ident_char(c) => {
                    let key = self.read_while(Self::is_valid_ident_char);
                    Some((SyntaxKind::IDENT, key))
                }
                _ => {
                    self.input.next();
                    Some((SyntaxKind::ERROR, c.to_string()))
                }
            }
        } else {
            None
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = (SyntaxKind, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub(crate) fn lex(input: &str) -> Vec<(SyntaxKind, String)> {
    let mut lexer = Lexer::new(input);
    lexer.by_ref().collect::<Vec<_>>()
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    pub name: String,
    pub archqual: Option<String>,
    pub architectures: Option<Vec<String>>,
    pub version: Option<(VersionConstraint, debversion::Version)>,
    pub profiles: Vec<Vec<BuildProfile>>,
}

impl Relation {
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
    /// use debian_control::relations::{Relation};
    /// let entry: Relation = "samba (>= 2.0)".parse().unwrap();
    /// assert!(entry.satisfied_by(&mut |name| {
    ///    match name {
    ///    "samba" => Some("2.0".parse().unwrap()),
    ///    _ => None
    /// }}));
    /// ```
    pub fn satisfied_by(&self, package_version: &mut dyn FnMut(&str) -> Option<debversion::Version>) -> bool {
        let actual = package_version(self.name.as_str());
        if let Some((vc, version)) = &self.version {
            if let Some(actual) = actual {
                match vc {
                    VersionConstraint::GreaterThanEqual => actual >= *version,
                    VersionConstraint::LessThanEqual => actual <= *version,
                    VersionConstraint::Equal => actual == *version,
                    VersionConstraint::GreaterThan => actual > *version,
                    VersionConstraint::LessThan => actual < *version,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relations(pub Vec<Vec<Relation>>);

impl std::ops::Index <usize> for Relations {
    type Output = Vec<Relation>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut <usize> for Relations {
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

impl Relations {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec<&Relation>> {
        self.0.iter().map(|entry| entry.iter().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn satisfied_by(&self, package_version: &mut dyn FnMut(&str) -> Option<debversion::Version>) -> bool {
        self.0.iter().all(|e| e.iter().any(|r| r.satisfied_by(package_version)))
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
                    EQUAL | L_ANGLE | R_ANGLE => { constraint.push_str(&t); tokens.next(); }
                    _ => break,
                }
            };
            let constraint = constraint.parse()?;
            eat_whitespace(&mut tokens);
            let version_str = match tokens.next() {
                Some((IDENT, s)) => s,
                _ => return Err("Expected version".to_string()),
            };
            let version = version_str.parse().map_err(|e: debversion::ParseError| e.to_string())?;
            eat_whitespace(&mut tokens);
            if let Some((R_PARENS, _)) = tokens.next() {
            } else {
                return Err("Expected ')'".to_string());
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
        };

        eat_whitespace(&mut tokens);

        if let Some((kind, _)) = tokens.next() {
            return Err(format!("Unexpected token: {:?}", kind));
        }

        Ok(Relation {
            name,
            archqual,
            architectures,
            version,
            profiles
        })
    }
}

impl std::str::FromStr for Relations {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut relations = Vec::new();
        for entry in s.split(",") {
            let entry = entry.trim();
            if entry.is_empty() {
                return Err("Empty entry".to_string());
            }
            let entry_relations = entry.split("|").map(|relation| {
                let relation = relation.trim();
                if relation.is_empty() {
                    return Err("Empty relation".to_string());
                }
                Ok(relation.parse()?)
            });
            relations.push(entry_relations.collect::<Result<Vec<_>, _>>()?);
        }
        Ok(Relations(relations))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VersionConstraint {
    LessThan,         // <<
    LessThanEqual,    // <=
    Equal,            // =
    GreaterThan,      // >>
    GreaterThanEqual, // >=
}

impl std::str::FromStr for VersionConstraint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">=" => Ok(VersionConstraint::GreaterThanEqual),
            "<=" => Ok(VersionConstraint::LessThanEqual),
            "=" => Ok(VersionConstraint::Equal),
            ">>" => Ok(VersionConstraint::GreaterThan),
            "<<" => Ok(VersionConstraint::LessThan),
            _ => Err(format!("Invalid version constraint: {}", s)),
        }
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VersionConstraint::GreaterThanEqual => f.write_str(">="),
            VersionConstraint::LessThanEqual => f.write_str("<="),
            VersionConstraint::Equal => f.write_str("="),
            VersionConstraint::GreaterThan => f.write_str(">>"),
            VersionConstraint::LessThan => f.write_str("<<"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildProfile {
    Enabled(String),
    Disabled(String),
}

impl std::fmt::Display for BuildProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildProfile::Enabled(s) => f.write_str(s),
            BuildProfile::Disabled(s) => write!(f, "!{}", s),
        }
    }
}

impl std::str::FromStr for BuildProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('!') {
            Ok(BuildProfile::Disabled(s.to_string()))
        } else {
            Ok(BuildProfile::Enabled(s.to_string()))
        }
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
}
