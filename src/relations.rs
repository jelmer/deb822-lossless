use debversion::Version;
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
    CONSTRAINT, // (">=", "<=", "=", ">>", "<<")
    COMMA,      // ,
    L_PARENS,   // (
    R_PARENS,   // )
    L_BRACKET,  // [
    R_BRACKET,  // ]
    NOT,        // !
    WHITESPACE, // whitespace
    COMMENT,    // comments
    ERROR,      // as well as errors

    // composite nodes
    ROOT,     // The entire file
    ENTRY,    // A single entry
    RELATION, // An alternative in a dependency
    VERSION,  // A version constraint
    ARCHITECTURES,
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

impl ToString for VersionConstraint {
    fn to_string(&self) -> String {
        match self {
            VersionConstraint::GreaterThanEqual => ">=".to_owned(),
            VersionConstraint::LessThanEqual => "<=".to_owned(),
            VersionConstraint::Equal => "=".to_owned(),
            VersionConstraint::GreaterThan => ">>".to_owned(),
            VersionConstraint::LessThan => "<<".to_owned(),
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
        c == ' ' || c == '\t' || c == '\r' || c == '\n'
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
                '<' | '>' | '=' => {
                    let constraint = self.read_while(|c| c == '<' || c == '>' || c == '=');
                    Some((SyntaxKind::CONSTRAINT, constraint))
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
use rowan::GreenNode;

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
        fn parse_entry(&mut self) {
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
                    _ => {
                        self.builder.start_node(SyntaxKind::ERROR.into());
                        if self.current().is_some() {
                            self.bump();
                        }
                        self.errors.push("Expected comma or pipe".to_owned());
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
                if self.current() == Some(CONSTRAINT) {
                    self.bump();
                } else {
                    self.error("Expected version constraint");
                }

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

            self.builder.finish_node();
        }

        fn parse(mut self) -> Parse {
            self.builder.start_node(SyntaxKind::ROOT.into());

            self.skip_ws();

            while self.current().is_some() {
                self.parse_entry();
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
            while self.current() == Some(WHITESPACE) || self.current() == Some(COMMENT) {
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
        SyntaxNode::new_root(self.green_node.clone())
    }

    fn root(&self) -> Relations {
        Relations::cast(self.syntax()).unwrap()
    }
}

macro_rules! ast_node {
    ($ast:ident, $kind:ident) => {
        #[derive(PartialEq, Eq, Hash)]
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

        impl ToString for $ast {
            fn to_string(&self) -> String {
                self.0.text().to_string()
            }
        }
    };
}

ast_node!(Relations, ROOT);
ast_node!(Entry, ENTRY);
ast_node!(Relation, RELATION);

impl std::fmt::Debug for Relations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Relations").finish()
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry").finish()
    }
}

impl std::fmt::Debug for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Relation").finish()
    }
}

impl Relations {
    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.0.children().filter_map(Entry::cast)
    }
}

impl Entry {
    pub fn relations(&self) -> impl Iterator<Item = Relation> + '_ {
        self.0.children().filter_map(Relation::cast)
    }
}

impl Relation {
    pub fn version(&self) -> Option<(VersionConstraint, Version)> {
        let vc = self.0.children().find(|n| n.kind() == VERSION);
        let vc = vc.as_ref()?;
        let constraint = vc.children_with_tokens().find_map(|it| match it {
            SyntaxElement::Token(token) if token.kind() == CONSTRAINT => Some(token),
            _ => None,
        });
        let version = vc.children_with_tokens().find_map(|it| match it {
            SyntaxElement::Token(token) if token.kind() == IDENT => Some(token),
            _ => None,
        });

        if let (Some(constraint), Some(version)) = (constraint, version) {
            let vc: VersionConstraint = constraint.text().to_string().parse().unwrap();
            return Some((vc, (version.text().to_string()).parse().unwrap()));
        } else {
            None
        }
    }

    pub fn arch_list(&self) -> impl Iterator<Item = String> + '_ {
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
fn test_arch_list() {
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
        relation.arch_list().collect::<Vec<_>>(),
        vec!["amd64", "arm64", "armhf", "i386", "mips", "mips64el", "mipsel", "ppc64el", "s390x"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    );
}
