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
    VERSION,    // A version constraint
    CONSTRAINT, // (">=", "<=", "=", ">>", "<<")
    ARCHITECTURES,
    PROFILES,
    SUBSTVAR,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VersionConstraint {
    GreaterThanEqual, // >=
    LessThanEqual,    // <=
    Equal,            // =
    GreaterThan,      // >>
    LessThan,         // <<
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

    fn is_valid_key_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '+' || c == '*'
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
                _ if Self::is_valid_key_char(c) => {
                    let key = self.read_while(Self::is_valid_key_char);
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

fn parse(text: &str) -> Parse {
    struct Parser {
        /// input tokens, including whitespace,
        /// in *reverse* order.
        tokens: Vec<(SyntaxKind, String)>,
        /// the in-progress tree.
        builder: GreenNodeBuilder<'static>,
        /// the list of syntax errors we've accumulated
        /// so far.
        errors: Vec<String>,
    }

    impl Parser {
        fn parse_substvar(&mut self) {
            self.builder.start_node(SyntaxKind::SUBSTVAR.into());
            self.bump();
            if self.current() != Some(L_CURLY) {
                self.error("expected {");
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
                        self.error(format!("unexpected identifier: {:?}", e).as_str());
                    }
                }
            }
            if self.current() != Some(R_CURLY) {
                self.error("expected }");
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
                        if self.current().is_some() {
                            self.bump();
                        }
                        self.errors
                            .push(format!("Expected comma or pipe, not {:?}", e));
                        self.builder.finish_node();
                    }
                }
            }
            self.builder.finish_node();
        }

        fn error(&mut self, error: &str) {
            self.errors.push(error.to_owned());
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
                self.error("Expected package name");
            }
            self.skip_ws();
            match self.current() {
                Some(COLON) => {
                    self.bump();
                    self.skip_ws();
                    if self.current() == Some(IDENT) {
                        self.bump();
                    } else {
                        self.error("Expected architecture name");
                    }
                    self.skip_ws();
                }
                None | Some(L_PARENS) | Some(L_BRACKET) | Some(PIPE) | Some(COMMA) => {}
                e => {
                    self.error(
                        format!("Expected ':' or '|' or '[' or ',' but got {:?}", e).as_str(),
                    );
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
                    self.error("Expected version");
                }

                if self.current() == Some(R_PARENS) {
                    self.bump();
                } else {
                    self.error("Expected ')'");
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
                            self.error("Expected architecture name or '!' or ']'");
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
                                self.error("Expected profile");
                            }
                        }
                        Some(R_ANGLE) => {
                            self.bump();
                            break;
                        }
                        None => {
                            self.error("Expected profile or '>'");
                            break;
                        }
                        _ => {
                            self.error("Expected profile or '!' or '>'");
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
                    Some(DOLLAR) => self.parse_substvar(),
                    _ => {
                        self.error("expected $ or identifier");
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
                    _ => {
                        self.error("Expected comma");
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
        let is_empty = !self.entries().any(|_| true);
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
            let child_count = self.0.children().count();
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
        );
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

impl Entry {
    pub fn new() -> Self {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::ENTRY.into());
        builder.finish_node();
        Entry(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    pub fn relations(&self) -> impl Iterator<Item = Relation> + '_ {
        self.0.children().filter_map(Relation::cast)
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
    for child in node.children_with_tokens() {
        match child {
            rowan::NodeOrToken::Node(child) => {
                builder.start_node(child.kind().into());
                inject(builder, child);
                builder.finish_node();
            }
            rowan::NodeOrToken::Token(token) => {
                builder.token(token.kind().into(), token.text());
            }
        }
    }
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
    pub fn architectures(&self) -> impl Iterator<Item = String> + '_ {
        let architectures = self.0.children().find(|n| n.kind() == ARCHITECTURES);

        let architectures = architectures.as_ref().unwrap();

        architectures.children_with_tokens().filter_map(|node| {
            let token = node.as_token()?;
            if token.kind() == IDENT {
                Some(token.text().to_string())
            } else {
                None
            }
        })
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

impl std::str::FromStr for Relations {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse = parse(s);
        if parse.errors.is_empty() {
            Ok(parse.root())
        } else {
            Err(parse.errors.join("\n"))
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
            relation.architectures().collect::<Vec<_>>(),
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
            relation.architectures().collect::<Vec<_>>(),
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

        let parsed: Relations = input.parse().unwrap();
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
        assert_eq!(rels.to_string(), "python3-dulwich, python3-breezy");
    }

    #[test]
    fn test_entry_from_relations() {
        let relations = vec![
            Relation::simple("python3-dulwich"),
            Relation::simple("python3-breezy"),
        ];
        let entry: Entry = relations.into();
        assert_eq!(entry.to_string(), "python3-dulwich | python3-breezy");
    }
}
