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
//! relations.get_entry(0).unwrap().get_relation(0).unwrap().set_archqual("amd64");
//! assert_eq!(relations.to_string(), "python3-dulwich:amd64 (>= 0.19.0), python3-urllib3 (<< 1.26.0)");
//! ```
use debversion::Version;
use rowan::{Direction, NodeOrToken};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VersionConstraint {
    LessThan,         // <<
    LessThanEqual,    // <=
    Equal,            // =
    GreaterThan,      // >>
    GreaterThanEqual, // >=
}

impl std::str::FromStr for VersionConstraint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">=" => Ok(VersionConstraint::GreaterThanEqual),
            "<=" => Ok(VersionConstraint::LessThanEqual),
            "=" => Ok(VersionConstraint::Equal),
            ">>" => Ok(VersionConstraint::GreaterThan),
            "<<" => Ok(VersionConstraint::LessThan),
            _ => Err(ParseError(vec![format!(
                "Invalid version constraint: {}",
                s
            )])),
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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('!') {
            Ok(BuildProfile::Disabled(s.to_string()))
        } else {
            Ok(BuildProfile::Enabled(s.to_string()))
        }
    }
}

use SyntaxKind::*;

/// Convert our `SyntaxKind` into the rowan `SyntaxKind`.
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
pub struct ParseError(Vec<String>);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for err in &self.0 {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Second, implementing the `Language` trait teaches rowan to convert between
/// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
/// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Lang {}
impl rowan::Language for Lang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

/// GreenNode is an immutable tree, which is cheap to change,
/// but doesn't contain offsets and parent pointers.
use rowan::{GreenNode, GreenToken};

/// You can construct GreenNodes by hand, but a builder
/// is helpful for top-down parsers: it maintains a stack
/// of currently in-progress nodes
use rowan::GreenNodeBuilder;

/// The parse results are stored as a "green tree".
/// We'll discuss working with the results later
struct Parse {
    green_node: GreenNode,
    #[allow(unused)]
    errors: Vec<String>,
}

fn parse(text: &str, allow_substvar: bool) -> Parse {
    struct Parser {
        /// input tokens, including whitespace,
        /// in *reverse* order.
        tokens: Vec<(SyntaxKind, String)>,
        /// the in-progress tree.
        builder: GreenNodeBuilder<'static>,
        /// the list of syntax errors we've accumulated
        /// so far.
        errors: Vec<String>,
        /// whether to allow substvars
        allow_substvar: bool,
    }

    impl Parser {
        fn parse_substvar(&mut self) {
            self.builder.start_node(SyntaxKind::SUBSTVAR.into());
            self.bump();
            if self.current() != Some(L_CURLY) {
                self.error(format!("expected {{ but got {:?}", self.current()).to_string());
            } else {
                self.bump();
            }
            loop {
                match self.current() {
                    Some(IDENT) | Some(COLON) => {
                        self.bump();
                    }
                    Some(R_CURLY) => {
                        break;
                    }
                    e => {
                        self.error(format!("expected identifier or : but got {:?}", e).to_string());
                    }
                }
            }
            if self.current() != Some(R_CURLY) {
                self.error(format!("expected }} but got {:?}", self.current()).to_string());
            } else {
                self.bump();
            }
            self.builder.finish_node();
        }

        fn parse_entry(&mut self) {
            self.skip_ws();
            self.builder.start_node(SyntaxKind::ENTRY.into());
            loop {
                self.parse_relation();
                self.skip_ws();
                match self.current() {
                    Some(COMMA) => {
                        break;
                    }
                    Some(PIPE) => {
                        self.bump();
                        self.skip_ws();
                    }
                    None => {
                        break;
                    }
                    e => {
                        self.builder.start_node(SyntaxKind::ERROR.into());
                        match self.tokens.pop() {
                            Some((k, t)) => {
                                self.builder.token(k.into(), t.as_str());
                                self.errors
                                    .push(format!("Expected comma or pipe, not {:?}", (k, t)));
                            }
                            None => {
                                self.errors
                                    .push("Expected comma or pipe, got end of file".to_string());
                            }
                        }
                        self.builder.finish_node();
                    }
                }
            }
            self.builder.finish_node();
        }

        fn error(&mut self, error: String) {
            self.errors.push(error);
            self.builder.start_node(SyntaxKind::ERROR.into());
            if self.current().is_some() {
                self.bump();
            }
            self.builder.finish_node();
        }

        fn parse_relation(&mut self) {
            self.builder.start_node(SyntaxKind::RELATION.into());
            if self.current() == Some(IDENT) {
                self.bump();
            } else {
                self.error("Expected package name".to_string());
            }
            self.skip_ws();
            match self.current() {
                Some(COLON) => {
                    self.builder.start_node(ARCHQUAL.into());
                    self.bump();
                    self.skip_ws();
                    if self.current() == Some(IDENT) {
                        self.bump();
                    } else {
                        self.error("Expected architecture name".to_string());
                    }
                    self.builder.finish_node();
                    self.skip_ws();
                }
                None | Some(L_PARENS) | Some(L_BRACKET) | Some(PIPE) | Some(COMMA) => {}
                e => {
                    self.error(format!("Expected ':' or '|' or '[' or ',' but got {:?}", e));
                }
            }

            self.skip_ws();

            if self.current() == Some(L_PARENS) {
                self.builder.start_node(VERSION.into());
                self.bump();
                self.skip_ws();

                self.builder.start_node(CONSTRAINT.into());

                while self.current() == Some(L_ANGLE)
                    || self.current() == Some(R_ANGLE)
                    || self.current() == Some(EQUAL)
                {
                    self.bump();
                }

                self.builder.finish_node();

                self.skip_ws();

                if self.current() == Some(IDENT) {
                    self.bump();
                } else {
                    self.error("Expected version".to_string());
                }

                if self.current() == Some(R_PARENS) {
                    self.bump();
                } else {
                    self.error("Expected ')'".to_string());
                }

                self.builder.finish_node();
            }

            self.skip_ws();

            if self.current() == Some(L_BRACKET) {
                self.builder.start_node(ARCHITECTURES.into());
                self.bump();
                loop {
                    self.skip_ws();
                    match self.current() {
                        Some(NOT) => {
                            self.bump();
                        }
                        Some(IDENT) => {
                            self.bump();
                        }
                        Some(R_BRACKET) => {
                            self.bump();
                            break;
                        }
                        _ => {
                            self.error("Expected architecture name or '!' or ']'".to_string());
                        }
                    }
                }
                self.builder.finish_node();
            }
            self.skip_ws();

            while self.current() == Some(L_ANGLE) {
                self.builder.start_node(PROFILES.into());
                self.bump();

                loop {
                    self.skip_ws();
                    match self.current() {
                        Some(IDENT) => {
                            self.bump();
                        }
                        Some(NOT) => {
                            self.bump();
                            self.skip_ws();
                            if self.current() == Some(IDENT) {
                                self.bump();
                            } else {
                                self.error("Expected profile".to_string());
                            }
                        }
                        Some(R_ANGLE) => {
                            self.bump();
                            break;
                        }
                        None => {
                            self.error("Expected profile or '>'".to_string());
                            break;
                        }
                        _ => {
                            self.error("Expected profile or '!' or '>'".to_string());
                        }
                    }
                }

                self.builder.finish_node();

                self.skip_ws();
            }

            self.builder.finish_node();
        }

        fn parse(mut self) -> Parse {
            self.builder.start_node(SyntaxKind::ROOT.into());

            self.skip_ws();

            while self.current().is_some() {
                match self.current() {
                    Some(IDENT) => self.parse_entry(),
                    Some(DOLLAR) => {
                        if self.allow_substvar {
                            self.parse_substvar()
                        } else {
                            self.error("Substvars are not allowed".to_string());
                        }
                    }
                    Some(c) => {
                        self.error(format!("expected $ or identifier but got {:?}", c));
                    }
                    None => {
                        self.error("expected identifier but got end of file".to_string());
                    }
                }

                self.skip_ws();
                match self.current() {
                    Some(COMMA) => {
                        self.bump();
                    }
                    None => {
                        break;
                    }
                    c => {
                        self.error(format!("expected comma or end of file but got {:?}", c));
                    }
                }
                self.skip_ws();
            }

            self.builder.finish_node();
            // Turn the builder into a GreenNode
            Parse {
                green_node: self.builder.finish(),
                errors: self.errors,
            }
        }
        /// Advance one token, adding it to the current branch of the tree builder.
        fn bump(&mut self) {
            let (kind, text) = self.tokens.pop().unwrap();
            self.builder.token(kind.into(), text.as_str());
        }
        /// Peek at the first unprocessed token
        fn current(&self) -> Option<SyntaxKind> {
            self.tokens.last().map(|(kind, _)| *kind)
        }
        fn skip_ws(&mut self) {
            while self.current() == Some(WHITESPACE)
                || self.current() == Some(COMMENT)
                || self.current() == Some(NEWLINE)
            {
                self.bump()
            }
        }
    }

    let mut tokens = lex(text);
    tokens.reverse();
    Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
        allow_substvar,
    }
    .parse()
}

/// To work with the parse results we need a view into the
/// green tree - the Syntax tree.
/// It is also immutable, like a GreenNode,
/// but it contains parent pointers, offsets, and
/// has identity semantics.

type SyntaxNode = rowan::SyntaxNode<Lang>;
#[allow(unused)]
type SyntaxToken = rowan::SyntaxToken<Lang>;
#[allow(unused)]
type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

impl Parse {
    fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone()).clone_for_update()
    }

    fn root(&self) -> Relations {
        Relations::cast(self.syntax()).unwrap()
    }
}

macro_rules! ast_node {
    ($ast:ident, $kind:ident) => {
        #[repr(transparent)]
        pub struct $ast(SyntaxNode);
        impl $ast {
            #[allow(unused)]
            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == $kind {
                    Some(Self(node))
                } else {
                    None
                }
            }
        }

        impl std::fmt::Display for $ast {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0.text().to_string())
            }
        }
    };
}

ast_node!(Relations, ROOT);
ast_node!(Entry, ENTRY);
ast_node!(Relation, RELATION);
ast_node!(Substvar, SUBSTVAR);

impl PartialEq for Relations {
    fn eq(&self, other: &Self) -> bool {
        self.entries().collect::<Vec<_>>() == other.entries().collect::<Vec<_>>()
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.relations().collect::<Vec<_>>() == other.relations().collect::<Vec<_>>()
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.version() == other.version()
    }
}

impl std::fmt::Debug for Relations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Relations");

        for entry in self.entries() {
            s.field("entry", &entry);
        }

        s.finish()
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Entry");

        for relation in self.relations() {
            s.field("relation", &relation);
        }

        s.finish()
    }
}

impl std::fmt::Debug for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Relation");

        s.field("name", &self.name());

        if let Some((vc, version)) = self.version() {
            s.field("version", &vc);
            s.field("version", &version);
        }

        s.finish()
    }
}

impl Relations {
    pub fn new() -> Self {
        Self::from(vec![])
    }

    #[must_use]
    pub fn wrap_and_sort(self) -> Self {
        let mut entries = self.entries().map(|e| e.wrap_and_sort()).collect::<Vec<_>>();
        entries.sort();
        // TODO: preserve comments
        Self::from(entries)
    }

    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.0.children().filter_map(Entry::cast)
    }

    /// Remove the entry at the given index
    pub fn get_entry(&self, idx: usize) -> Option<Entry> {
        self.entries().nth(idx)
    }

    /// Remove the entry at the given index
    pub fn remove(&mut self, idx: usize) {
        self.get_entry(idx).unwrap().remove();
    }

    /// Insert a new entry at the given index
    pub fn insert(&mut self, idx: usize, entry: Entry) {
        let is_empty = !self.0.children_with_tokens().any(|n| n.kind() == COMMA);
        let (position, new_children) = if let Some(current_entry) = self.entries().nth(idx) {
            let to_insert: Vec<NodeOrToken<GreenNode, GreenToken>> = if idx == 0 && is_empty {
                vec![entry.0.green().into()]
            } else {
                vec![
                    entry.0.green().into(),
                    NodeOrToken::Token(GreenToken::new(COMMA.into(), ",")),
                    NodeOrToken::Token(GreenToken::new(WHITESPACE.into(), " ")),
                ]
            };

            (current_entry.0.index(), to_insert)
        } else {
            let child_count = self.0.children_with_tokens().count();
            (
                child_count,
                if idx == 0 {
                    vec![entry.0.green().into()]
                } else {
                    vec![
                        NodeOrToken::Token(GreenToken::new(COMMA.into(), ",")),
                        NodeOrToken::Token(GreenToken::new(WHITESPACE.into(), " ")),
                        entry.0.green().into(),
                    ]
                },
            )
        };
        self.0 = SyntaxNode::new_root(
            self.0.replace_with(
                self.0
                    .green()
                    .splice_children(position..position, new_children),
            ),
        )
        .clone_for_update();
    }

    pub fn replace(&mut self, idx: usize, entry: Entry) {
        let current_entry = self.get_entry(idx).unwrap();
        self.0.splice_children(
            current_entry.0.index()..current_entry.0.index() + 1,
            vec![entry.0.into()],
        );
    }

    pub fn push(&mut self, entry: Entry) {
        let pos = self.entries().count();
        self.insert(pos, entry);
    }

    pub fn substvars(&self) -> impl Iterator<Item = String> + '_ {
        self.0
            .children()
            .filter_map(Substvar::cast)
            .map(|s| s.to_string())
    }

    pub fn parse_relaxed(s: &str, allow_substvar: bool) -> (Relations, Vec<String>) {
        let parse = parse(s, allow_substvar);
        (parse.root(), parse.errors)
    }

    pub fn satisfied_by(&self, package_version: &mut dyn FnMut(&str) -> Option<debversion::Version>) -> bool {
        self.entries().all(|e| e.satisfied_by(package_version))
    }
}

impl From<Vec<Entry>> for Relations {
    fn from(entries: Vec<Entry>) -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(ROOT.into());
        for (i, entry) in entries.into_iter().enumerate() {
            if i > 0 {
                builder.token(COMMA.into(), ",");
                builder.token(WHITESPACE.into(), " ");
            }
            inject(&mut builder, entry.0);
        }
        builder.finish_node();
        Relations(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }
}

impl From<Entry> for Relations {
    fn from(entry: Entry) -> Self {
        Self::from(vec![entry])
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut rels_a = self.relations();
        let mut rels_b = other.relations();
        while let (Some(a), Some(b)) = (rels_a.next(), rels_b.next()) {
            match a.cmp(&b) {
                std::cmp::Ordering::Equal => continue,
                x => return Some(x),
            }
        }

        if rels_a.next().is_some() {
            return Some(std::cmp::Ordering::Greater);
        }

        if rels_b.next().is_some() {
            return Some(std::cmp::Ordering::Less);
        }

        Some(std::cmp::Ordering::Equal)
    }
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Entry {
    pub fn new() -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::ENTRY.into());
        builder.finish_node();
        Entry(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    #[must_use]
    pub fn wrap_and_sort(&self) -> Self {
        let mut relations = self.relations().map(|r| r.wrap_and_sort()).collect::<Vec<_>>();
        // TODO: preserve comments
        relations.sort();
        Self::from(relations)
    }

    pub fn relations(&self) -> impl Iterator<Item = Relation> + '_ {
        self.0.children().filter_map(Relation::cast)
    }

    pub fn get_relation(&self, idx: usize) -> Option<Relation> {
        self.relations().nth(idx)
    }

    /// Check if this entry is satisfied by the given package versions.
    ///
    /// # Arguments
    /// * `package_version` - A function that returns the version of a package.
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::Entry;
    /// let entry = Entry::from(vec!["samba (>= 2.0)".parse().unwrap()]);
    /// assert!(entry.satisfied_by(&mut |name| {
    ///    match name {
    ///    "samba" => Some("2.0".parse().unwrap()),
    ///    _ => None
    /// }}));
    /// ```
    pub fn satisfied_by(&self, package_version: &mut dyn FnMut(&str) -> Option<debversion::Version>) -> bool {
        self.relations().any(|r| {
            let actual = package_version(r.name().as_str());
            if let Some((vc, version)) = r.version() {
                if let Some(actual) = actual {
                    match vc {
                        VersionConstraint::GreaterThanEqual => actual >= version,
                        VersionConstraint::LessThanEqual => actual <= version,
                        VersionConstraint::Equal => actual == version,
                        VersionConstraint::GreaterThan => actual > version,
                        VersionConstraint::LessThan => actual < version,
                    }
                } else {
                    false
                }
            } else {
                actual.is_some()
            }
        })
    }

    /// Remove this entry
    pub fn remove(&mut self) {
        let mut removed_comma = false;
        let is_first = !self
            .0
            .siblings(Direction::Prev)
            .skip(1)
            .any(|n| n.kind() == ENTRY);
        while let Some(n) = self.0.next_sibling_or_token() {
            if n.kind() == WHITESPACE || n.kind() == COMMENT || n.kind() == NEWLINE {
                n.detach();
            } else if n.kind() == COMMA {
                n.detach();
                removed_comma = true;
                break;
            } else {
                panic!("Unexpected node: {:?}", n);
            }
        }
        if !is_first {
            while let Some(n) = self.0.prev_sibling_or_token() {
                if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                    n.detach();
                } else if !removed_comma && n.kind() == COMMA {
                    n.detach();
                    break;
                } else {
                    break;
                }
            }
        } else {
            while let Some(n) = self.0.next_sibling_or_token() {
                if n.kind() == WHITESPACE || n.kind() == NEWLINE {
                    n.detach();
                } else {
                    break;
                }
            }
        }
        self.0.detach();
    }
}

fn inject(builder: &mut GreenNodeBuilder, node: SyntaxNode) {
    builder.start_node(node.kind().into());
    for child in node.children_with_tokens() {
        match child {
            rowan::NodeOrToken::Node(child) => {
                inject(builder, child);
            }
            rowan::NodeOrToken::Token(token) => {
                builder.token(token.kind().into(), token.text());
            }
        }
    }
    builder.finish_node();
}

impl From<Vec<Relation>> for Entry {
    fn from(relations: Vec<Relation>) -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::ENTRY.into());
        for (i, relation) in relations.into_iter().enumerate() {
            if i > 0 {
                builder.token(WHITESPACE.into(), " ");
                builder.token(COMMA.into(), "|");
                builder.token(WHITESPACE.into(), " ");
            }
            inject(&mut builder, relation.0);
        }
        builder.finish_node();
        Entry(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }
}

impl From<Relation> for Entry {
    fn from(relation: Relation) -> Self {
        Self::from(vec![relation])
    }
}

impl Relation {
    pub fn new(name: &str, version_constraint: Option<(VersionConstraint, Version)>) -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::RELATION.into());
        builder.token(IDENT.into(), name);
        if let Some((vc, version)) = version_constraint {
            builder.token(WHITESPACE.into(), " ");
            builder.start_node(SyntaxKind::VERSION.into());
            builder.token(L_PARENS.into(), "(");
            builder.start_node(SyntaxKind::CONSTRAINT.into());
            for c in vc.to_string().chars() {
                builder.token(
                    match c {
                        '>' => R_ANGLE.into(),
                        '<' => L_ANGLE.into(),
                        '=' => EQUAL.into(),
                        _ => unreachable!(),
                    },
                    c.to_string().as_str(),
                );
            }
            builder.finish_node();

            builder.token(WHITESPACE.into(), " ");

            builder.token(IDENT.into(), version.to_string().as_str());

            builder.token(R_PARENS.into(), ")");

            builder.finish_node();
        }

        builder.finish_node();
        Relation(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    #[must_use]
    pub fn wrap_and_sort(&self) -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::RELATION.into());
        builder.token(IDENT.into(), self.name().as_str());
        if let Some(archqual) = self.archqual() {
            builder.token(COLON.into(), ":");
            builder.token(IDENT.into(), archqual.as_str());
        }
        if let Some((vc, version)) = self.version() {
            builder.token(WHITESPACE.into(), " ");
            builder.start_node(SyntaxKind::VERSION.into());
            builder.token(L_PARENS.into(), "(");
            builder.start_node(SyntaxKind::CONSTRAINT.into());
            builder.token(
                match vc {
                    VersionConstraint::GreaterThanEqual => R_ANGLE.into(),
                    VersionConstraint::LessThanEqual => L_ANGLE.into(),
                    VersionConstraint::Equal => EQUAL.into(),
                    VersionConstraint::GreaterThan => R_ANGLE.into(),
                    VersionConstraint::LessThan => L_ANGLE.into(),
                },
                vc.to_string().as_str(),
            );
            builder.finish_node();
            builder.token(WHITESPACE.into(), " ");
            builder.token(IDENT.into(), version.to_string().as_str());
            builder.token(R_PARENS.into(), ")");
            builder.finish_node();
        }
        if let Some(architectures) = self.architectures() {
            builder.token(WHITESPACE.into(), " ");
            builder.start_node(ARCHITECTURES.into());
            builder.token(L_BRACKET.into(), "[");
            for (i, arch) in architectures.enumerate() {
                if i > 0 {
                    builder.token(WHITESPACE.into(), " ");
                }
                builder.token(IDENT.into(), arch.as_str());
            }
            builder.token(R_BRACKET.into(), "]");
            builder.finish_node();
        }
        for profiles in self.profiles() {
            builder.token(WHITESPACE.into(), " ");
            builder.start_node(PROFILES.into());
            builder.token(L_ANGLE.into(), "<");
            for (i, profile) in profiles.into_iter().enumerate() {
                if i > 0 {
                    builder.token(WHITESPACE.into(), " ");
                }
                match profile {
                    BuildProfile::Disabled(name) => {
                        builder.token(NOT.into(), "!");
                        builder.token(IDENT.into(), name.as_str());
                    }
                    BuildProfile::Enabled(name) => {
                        builder.token(IDENT.into(), name.as_str());
                    }
                }
            }
            builder.token(R_ANGLE.into(), ">");
            builder.finish_node();
        }
        builder.finish_node();
        Relation(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    /// Create a new simple relation, without any version constraints.
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::Relation;
    /// let relation = Relation::simple("samba");
    /// assert_eq!(relation.to_string(), "samba");
    /// ```
    pub fn simple(name: &str) -> Self {
        Self::new(name, None)
    }

    /// Remove the version constraint from the relation.
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::{Relation, VersionConstraint};
    /// let mut relation = Relation::new("samba", Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())));
    /// relation.drop_constraint();
    /// assert_eq!(relation.to_string(), "samba");
    /// ```
    pub fn drop_constraint(&mut self) -> bool {
        let version_token = self.0.children().find(|n| n.kind() == VERSION);
        if let Some(version_token) = version_token {
            // Remove any whitespace before the version token
            while let Some(prev) = version_token.prev_sibling_or_token() {
                if prev.kind() == WHITESPACE || prev.kind() == NEWLINE {
                    prev.detach();
                } else {
                    break;
                }
            }
            version_token.detach();
            return true;
        }

        false
    }

    /// Return the name of the package in the relation.
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::Relation;
    /// let relation = Relation::simple("samba");
    /// assert_eq!(relation.name(), "samba");
    /// ```
    pub fn name(&self) -> String {
        self.0
            .children_with_tokens()
            .find_map(|it| match it {
                SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
                _ => None,
            })
            .unwrap()
            .text()
            .to_string()
    }

    /// Return the archqual
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::Relation;
    /// let relation: Relation = "samba:any".parse().unwrap();
    /// assert_eq!(relation.archqual(), Some("any".to_string()));
    /// ```
    pub fn archqual(&self) -> Option<String> {
        let archqual = self.0.children().find(|n| n.kind() == ARCHQUAL);
        let node = if let Some(archqual) = archqual {
            archqual.children_with_tokens().find_map(|it| match it {
                SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
                _ => None,
            })
        } else {
            None
        };
        node.map(|n| n.text().to_string())
    }

    /// Set the architecture qualifier for this relation.
    ///
    /// # Example
    /// ```
    /// use debian_control::relations::Relation;
    /// let mut relation = Relation::simple("samba");
    /// relation.set_archqual("any");
    /// assert_eq!(relation.to_string(), "samba:any");
    /// ```
    pub fn set_archqual(&mut self, archqual: &str) {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(ARCHQUAL.into());
        builder.token(COLON.into(), ":");
        builder.token(IDENT.into(), archqual);
        builder.finish_node();

        let node_archqual = self.0.children().find(|n| n.kind() == ARCHQUAL);
        if let Some(node_archqual) = node_archqual {
            self.0 = SyntaxNode::new_root(node_archqual.replace_with(builder.finish()))
                .clone_for_update();
        } else {
            let name_node = self.0.children_with_tokens().find(|n| n.kind() == IDENT);
            let idx = if let Some(name_node) = name_node {
                name_node.index() + 1
            } else {
                0
            };
            self.0.splice_children(
                idx..idx,
                vec![SyntaxNode::new_root(builder.finish())
                    .clone_for_update()
                    .into()],
            );
        }
    }

    /// Return the version constraint and the version it is constrained to.
    pub fn version(&self) -> Option<(VersionConstraint, Version)> {
        let vc = self.0.children().find(|n| n.kind() == VERSION);
        let vc = vc.as_ref()?;
        let constraint = vc.children().find(|n| n.kind() == CONSTRAINT);

        let version = vc.children_with_tokens().find_map(|it| match it {
            SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
            _ => None,
        });

        if let (Some(constraint), Some(version)) = (constraint, version) {
            let vc: VersionConstraint = constraint.to_string().parse().unwrap();
            return Some((vc, (version.text().to_string()).parse().unwrap()));
        } else {
            None
        }
    }

    /// Return an iterator over the architectures for this relation
    pub fn architectures(&self) -> Option<impl Iterator<Item = String> + '_> {
        let architectures = self.0.children().find(|n| n.kind() == ARCHITECTURES)?;

        Some(architectures.children_with_tokens().filter_map(|node| {
            let token = node.as_token()?;
            if token.kind() == IDENT {
                Some(token.text().to_string())
            } else {
                None
            }
        }))
    }

    /// Returns an iterator over the build profiles for this relation<up><up>
    pub fn profiles(&self) -> impl Iterator<Item = Vec<BuildProfile>> + '_ {
        let profiles = self.0.children().filter(|n| n.kind() == PROFILES);

        profiles.map(|profile| {
            // iterate over nodes separated by whitespace tokens
            let mut ret = vec![];
            let mut current = vec![];
            for token in profile.children_with_tokens() {
                match token.kind() {
                    WHITESPACE | NEWLINE => {
                        if !current.is_empty() {
                            ret.push(current.join("").parse::<BuildProfile>().unwrap());
                            current = vec![];
                        }
                    }
                    L_ANGLE | R_ANGLE => {}
                    _ => {
                        current.push(token.to_string());
                    }
                }
            }
            if !current.is_empty() {
                ret.push(current.concat().parse().unwrap());
            }
            ret
        })
    }
}

impl PartialOrd for Relation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Compare by name first, then by version
        let name_cmp = self.name().cmp(&other.name());
        if name_cmp != std::cmp::Ordering::Equal {
            return Some(name_cmp);
        }

        let self_version = self.version();
        let other_version = other.version();

        match (self_version, other_version) {
            (Some((self_vc, self_version)), Some((other_vc, other_version))) => {
                let vc_cmp = self_vc.cmp(&other_vc);
                if vc_cmp != std::cmp::Ordering::Equal {
                    return Some(vc_cmp);
                }

                Some(self_version.cmp(&other_version))
            }
            (Some(_), None) => Some(std::cmp::Ordering::Greater),
            (None, Some(_)) => Some(std::cmp::Ordering::Less),
            (None, None) => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl Eq for Relation {}

impl Ord for Relation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::str::FromStr for Relations {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse = parse(s, false);
        if parse.errors.is_empty() {
            Ok(parse.root())
        } else {
            Err(parse.errors.join("\n"))
        }
    }
}

impl std::str::FromStr for Entry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let root: Relations = s.parse()?;

        let mut entries = root.entries();
        let entry = if let Some(entry) = entries.next() {
            entry
        } else {
            return Err("No entry found".to_string());
        };

        if entries.next().is_some() {
            return Err("Multiple entries found".to_string());
        }

        Ok(entry)
    }
}

impl std::str::FromStr for Relation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entry: Entry = s.parse()?;

        let mut relations = entry.relations();
        let relation = if let Some(relation) = relations.next() {
            relation
        } else {
            return Err("No relation found".to_string());
        };

        if relations.next().is_some() {
            return Err("Multiple relations found".to_string());
        }

        Ok(relation)
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
        assert_eq!(parsed.entries().count(), 1);
        let entry = parsed.entries().next().unwrap();
        assert_eq!(entry.to_string(), "python3-dulwich");
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(relation.to_string(), "python3-dulwich");
        assert_eq!(relation.version(), None);

        let input = "python3-dulwich (>= 0.20.21)";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.entries().count(), 1);
        let entry = parsed.entries().next().unwrap();
        assert_eq!(entry.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(relation.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(
            relation.version(),
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
        assert_eq!(parsed.entries().count(), 2);
        let entry = parsed.entries().next().unwrap();
        assert_eq!(entry.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(relation.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(
            relation.version(),
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.20.21".parse().unwrap()
            ))
        );
        let entry = parsed.entries().nth(1).unwrap();
        assert_eq!(entry.to_string(), "python3-dulwich (<< 0.21)");
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(relation.to_string(), "python3-dulwich (<< 0.21)");
        assert_eq!(
            relation.version(),
            Some((VersionConstraint::LessThan, "0.21".parse().unwrap()))
        );
    }

    #[test]
    fn test_architectures() {
        let input = "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]";
        let parsed: Relations = input.parse().unwrap();
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.entries().count(), 1);
        let entry = parsed.entries().next().unwrap();
        assert_eq!(
            entry.to_string(),
            "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]"
        );
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(
            relation.to_string(),
            "python3-dulwich [amd64 arm64 armhf i386 mips mips64el mipsel ppc64el s390x]"
        );
        assert_eq!(relation.version(), None);
        assert_eq!(
            relation.architectures().unwrap().collect::<Vec<_>>(),
            vec![
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
        assert_eq!(parsed.entries().count(), 2);
        let entry = parsed.entries().next().unwrap();
        assert_eq!(
            entry.to_string(),
            "foo (>= 1.0) [i386 arm] <!nocheck> <!cross>"
        );
        assert_eq!(entry.relations().count(), 1);
        let relation = entry.relations().next().unwrap();
        assert_eq!(
            relation.to_string(),
            "foo (>= 1.0) [i386 arm] <!nocheck> <!cross>"
        );
        assert_eq!(
            relation.version(),
            Some((VersionConstraint::GreaterThanEqual, "1.0".parse().unwrap()))
        );
        assert_eq!(
            relation.architectures().unwrap().collect::<Vec<_>>(),
            vec!["i386", "arm"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            relation.profiles().collect::<Vec<_>>(),
            vec![
                vec![BuildProfile::Disabled("nocheck".to_string())],
                vec![BuildProfile::Disabled("cross".to_string())]
            ]
        );
    }

    #[test]
    fn test_substvar() {
        let input = "${shlibs:Depends}";

        let (parsed, errors) = Relations::parse_relaxed(input, true);
        assert_eq!(errors, Vec::<String>::new());
        assert_eq!(parsed.to_string(), input);
        assert_eq!(parsed.entries().count(), 0);

        assert_eq!(
            parsed.substvars().collect::<Vec<_>>(),
            vec!["${shlibs:Depends}"]
        );
    }

    #[test]
    fn test_new() {
        let r = Relation::new(
            "samba",
            Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())),
        );

        assert_eq!(r.to_string(), "samba (>= 2.0)");
    }

    #[test]
    fn test_drop_constraint() {
        let mut r = Relation::new(
            "samba",
            Some((VersionConstraint::GreaterThanEqual, "2.0".parse().unwrap())),
        );

        r.drop_constraint();

        assert_eq!(r.to_string(), "samba");
    }

    #[test]
    fn test_simple() {
        let r = Relation::simple("samba");

        assert_eq!(r.to_string(), "samba");
    }

    #[test]
    fn test_remove_first_entry() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"#
            .parse()
            .unwrap();
        rels.remove(0);
        assert_eq!(rels.to_string(), "python3-dulwich (<< 0.21)");
    }

    #[test]
    fn test_remove_last_entry() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"#
            .parse()
            .unwrap();
        rels.remove(1);
        assert_eq!(rels.to_string(), "python3-dulwich (>= 0.20.21)");
    }

    #[test]
    fn test_remove_middle() {
        let mut rels: Relations =
            r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21), python3-dulwich (<< 0.22)"#
                .parse()
                .unwrap();
        rels.remove(1);
        assert_eq!(
            rels.to_string(),
            "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.22)"
        );
    }

    #[test]
    fn test_remove_added() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21)"#.parse().unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.push(entry);
        rels.remove(1);
        assert_eq!(rels.to_string(), "python3-dulwich (>= 0.20.21)");
    }

    #[test]
    fn test_push() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21)"#.parse().unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.push(entry);
        assert_eq!(
            rels.to_string(),
            "python3-dulwich (>= 0.20.21), python3-dulwich"
        );
    }

    #[test]
    fn test_push_from_empty() {
        let mut rels: Relations = "".parse().unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.push(entry);
        assert_eq!(rels.to_string(), "python3-dulwich");
    }

    #[test]
    fn test_insert() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"#
            .parse()
            .unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.insert(1, entry);
        assert_eq!(
            rels.to_string(),
            "python3-dulwich (>= 0.20.21), python3-dulwich, python3-dulwich (<< 0.21)"
        );
    }

    #[test]
    fn test_insert_at_start() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"#
            .parse()
            .unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.insert(0, entry);
        assert_eq!(
            rels.to_string(),
            "python3-dulwich, python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"
        );
    }

    #[test]
    fn test_insert_after_error() {
        let (mut rels, errors) = Relations::parse_relaxed("@foo@, debhelper (>= 1.0)", false);
        assert_eq!(
            errors,
            vec![
                "expected $ or identifier but got ERROR",
                "expected comma or end of file but got Some(IDENT)",
                "expected $ or identifier but got ERROR"
            ]
        );
        let entry = Entry::from(vec![Relation::simple("bar")]);
        rels.push(entry);
        assert_eq!(rels.to_string(), "@foo@, debhelper (>= 1.0), bar");
    }

    #[test]
    fn test_insert_before_error() {
        let (mut rels, errors) = Relations::parse_relaxed("debhelper (>= 1.0), @foo@, bla", false);
        assert_eq!(
            errors,
            vec![
                "expected $ or identifier but got ERROR",
                "expected comma or end of file but got Some(IDENT)",
                "expected $ or identifier but got ERROR"
            ]
        );
        let entry = Entry::from(vec![Relation::simple("bar")]);
        rels.insert(0, entry);
        assert_eq!(rels.to_string(), "bar, debhelper (>= 1.0), @foo@, bla");
    }

    #[test]
    fn test_replace() {
        let mut rels: Relations = r#"python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"#
            .parse()
            .unwrap();
        let entry = Entry::from(vec![Relation::simple("python3-dulwich")]);
        rels.replace(1, entry);
        assert_eq!(
            rels.to_string(),
            "python3-dulwich (>= 0.20.21), python3-dulwich"
        );
    }

    #[test]
    fn test_relation_from_entries() {
        let entries = vec![
            Entry::from(vec![Relation::simple("python3-dulwich")]),
            Entry::from(vec![Relation::simple("python3-breezy")]),
        ];
        let rels: Relations = entries.into();
        assert_eq!(rels.entries().count(), 2);
        assert_eq!(rels.to_string(), "python3-dulwich, python3-breezy");
    }

    #[test]
    fn test_entry_from_relations() {
        let relations = vec![
            Relation::simple("python3-dulwich"),
            Relation::simple("python3-breezy"),
        ];
        let entry: Entry = relations.into();
        assert_eq!(entry.relations().count(), 2);
        assert_eq!(entry.to_string(), "python3-dulwich | python3-breezy");
    }

    #[test]
    fn test_parse_entry() {
        let parsed: Entry = "python3-dulwich (>= 0.20.21) | bar".parse().unwrap();
        assert_eq!(parsed.to_string(), "python3-dulwich (>= 0.20.21) | bar");
        assert_eq!(parsed.relations().count(), 2);

        assert_eq!(
            "foo, bar".parse::<Entry>().unwrap_err(),
            "Multiple entries found"
        );
        assert_eq!("".parse::<Entry>().unwrap_err(), "No entry found");
    }

    #[test]
    fn test_parse_relation() {
        let parsed: Relation = "python3-dulwich (>= 0.20.21)".parse().unwrap();
        assert_eq!(parsed.to_string(), "python3-dulwich (>= 0.20.21)");
        assert_eq!(
            parsed.version(),
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.20.21".parse().unwrap()
            ))
        );
        assert_eq!(
            "foo | bar".parse::<Relation>().unwrap_err(),
            "Multiple relations found"
        );
        assert_eq!("".parse::<Relation>().unwrap_err(), "No entry found");
    }

    #[test]
    fn test_special() {
        let parsed: Relation = "librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-~~)"
            .parse()
            .unwrap();
        assert_eq!(
            parsed.to_string(),
            "librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-~~)"
        );
        assert_eq!(
            parsed.version(),
            Some((
                VersionConstraint::GreaterThanEqual,
                "0.1.138-~~".parse().unwrap()
            ))
        );
        assert_eq!(parsed.archqual(), Some("amd64".to_string()));
        assert_eq!(parsed.name(), "librust-breezyshim+dirty-tracker-dev");
    }

    #[test]
    fn test_satisfied_by() {
        let rels: Relations = "python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)"
            .parse()
            .unwrap();
        let mut satisfied = |name: &str| match name {
            "python3-dulwich" => Some("0.20.21".parse().unwrap()),
            _ => None,
        };
        assert!(rels.satisfied_by(&mut satisfied));

        let mut satisfied = |name: &str| match name {
            "python3-dulwich" => Some("0.21".parse().unwrap()),
            _ => None,
        };
        assert!(!rels.satisfied_by(&mut satisfied));

        let mut satisfied = |name: &str| match name {
            "python3-dulwich" => Some("0.20.20".parse().unwrap()),
            _ => None,
        };
        assert!(!rels.satisfied_by(&mut satisfied));
    }

    #[test]
    fn test_wrap_and_sort_relation() {
        let relation: Relation = "   python3-dulwich   (>= 11) [  amd64 ] <  lala>".parse().unwrap();

        let wrapped = relation.wrap_and_sort();

        assert_eq!(wrapped.to_string(), "python3-dulwich (>= 11) [amd64] <lala>");
    }

    #[test]
    fn test_wrap_and_sort_relations() {
        let entry: Relations = "python3-dulwich (>= 0.20.21)   | bar, \n\n\n\npython3-dulwich (<< 0.21)".parse().unwrap();

        let wrapped = entry.wrap_and_sort();

        assert_eq!(wrapped.to_string(), "bar | python3-dulwich (>= 0.20.21), python3-dulwich (<< 0.21)");
    }
}
