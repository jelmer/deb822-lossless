//! Parsing of Debian relations strings.
use crate::version::Version;
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

/// Constraint on a Debian package version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VersionConstraint {
    /// <<
    LessThan, // <<
    /// <=
    LessThanEqual, // <=
    /// =
    Equal, // =
    /// >>
    GreaterThan, // >>
    /// >=
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

/// Let's start with defining all kinds of tokens and
/// composite nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum SyntaxKind {
    IDENT = 0,  // package name
    COMMA,      // ,
    L_PARENS,   // (
    R_PARENS,   // )
    L_ANGLE,    // <
    R_ANGLE,    // >
    EQUAL,      // =
    WHITESPACE, // whitespace
    NEWLINE,    // newline
    ERROR,      // as well as errors

    // composite nodes
    ROOT,       // The entire file
    RELATION,   // An alternative in a dependency
    VERSION,    // A version constraint
    CONSTRAINT, // (">=", "<=", "=", ">>", "<<")
}

/// Convert our `SyntaxKind` into the rowan `SyntaxKind`.
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// A lexer for relations strings.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t' || c == '\r'
    }

    fn is_valid_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '.'
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

/// A trait for looking up versions of packages.
pub trait VersionLookup {
    /// Look up the version of a package.
    fn lookup_version<'a>(&'a self, package: &'_ str) -> Option<std::borrow::Cow<'a, Version>>;
}

impl VersionLookup for std::collections::HashMap<String, Version> {
    fn lookup_version<'a>(&'a self, package: &str) -> Option<Cow<'a, Version>> {
        self.get(package).map(Cow::Borrowed)
    }
}

impl<F> VersionLookup for F
where
    F: Fn(&str) -> Option<Version>,
{
    fn lookup_version<'a>(&'a self, name: &str) -> Option<Cow<'a, Version>> {
        self(name).map(Cow::Owned)
    }
}

impl VersionLookup for (String, Version) {
    fn lookup_version<'a>(&'a self, name: &str) -> Option<Cow<'a, Version>> {
        if name == self.0 {
            Some(Cow::Borrowed(&self.1))
        } else {
            None
        }
    }
}
