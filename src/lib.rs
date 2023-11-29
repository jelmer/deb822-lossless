//! Lossless parser for deb822 style files.
//!
//! This parser can be used to parse files in the deb822 format, while preserving
//! all whitespace and comments. It is based on the [rowan] library, which is a
//! lossless parser library for Rust.
//!
//! Once parsed, the file can be traversed or modified, and then written back to
//! a file.
//!
//! # Example
//!
//! ```rust
//! use deb822_lossless::Deb822;
//! use std::str::FromStr;
//!
//! let input = r#"Package: deb822-lossless
//! Maintainer: Jelmer Vernooĳ <jelmer@debian.org>
//! Homepage: https://github.com/jelmer/deb822-lossless
//! Section: rust
//!
//! Package: deb822-lossless
//! Architecture: any
//! Description: Lossless parser for deb822 style files.
//!   This parser can be used to parse files in the deb822 format, while preserving
//!   all whitespace and comments. It is based on the [rowan] library, which is a
//!   lossless parser library for Rust.
//! "#;
//!
//! let deb822 = Deb822::from_str(input).unwrap();
//! assert_eq!(deb822.paragraphs().count(), 2);
//! let homepage = deb822.paragraphs().nth(0).unwrap().get("Homepage");
//! assert_eq!(homepage.as_deref(), Some("https://github.com/jelmer/deb822-lossless"));
//! ```

mod lex;
use crate::lex::lex;
use rowan::ast::AstNode;
use std::str::FromStr;
use std::path::Path;

/// Let's start with defining all kinds of tokens and
/// composite nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    KEY = 0,
    VALUE,
    COLON,
    INDENT,
    NEWLINE,
    WHITESPACE, // whitespaces is explicit
    COMMENT,    // comments
    ERROR,      // as well as errors

    // composite nodes
    ROOT,      // The entire file
    PARAGRAPH, // A deb822 paragraph
    ENTRY,     // A single key-value pair
    EMPTY_LINE, // An empty line
}

use SyntaxKind::*;

/// Convert our `SyntaxKind` into the rowan `SyntaxKind`.
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
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

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    IoError(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Error::ParseError(err) => write!(f, "{}", err),
            Error::IoError(err) => write!(f, "{}", err),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl std::error::Error for Error {}

/// Second, implementing the `Language` trait teaches rowan to convert between
/// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
/// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}
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
            self.builder.start_node(ENTRY.into());
            if self.current() == Some(KEY) {
                self.bump();
                self.skip_ws();
            } else {
                self.builder.start_node(ERROR.into());
                self.bump();
                self.errors.push("expected key".to_string());
                self.builder.finish_node();
            }
            if self.current() == Some(COLON) {
                self.bump();
                self.skip_ws();
            } else {
                self.builder.start_node(ERROR.into());
                self.bump();
                self.errors.push("expected ':'".to_string());
                self.builder.finish_node();
            }
            loop {
                while self.current() == Some(WHITESPACE) || self.current() == Some(VALUE) {
                    self.bump();
                }

                match self.current() {
                    None => {
                        break;
                    }
                    Some(NEWLINE) => {
                        self.bump();
                    }
                    Some(_) => {
                        self.builder.start_node(ERROR.into());
                        self.bump();
                        self.errors.push("expected newline".to_string());
                        self.builder.finish_node();
                    }
                }
                if self.current() == Some(INDENT) {
                    self.bump();
                    self.skip_ws();
                } else {
                    break;
                }
            }
            self.builder.finish_node();
        }

        fn parse_paragraph(&mut self) {
            self.builder.start_node(PARAGRAPH.into());
            while self.current() != Some(NEWLINE) && self.current().is_some() {
                self.parse_entry();
            }
            self.builder.finish_node();
        }

        fn parse(mut self) -> Parse {
            // Make sure that the root node covers all source
            self.builder.start_node(ROOT.into());
            while self.current().is_some() {
                self.skip_ws_and_newlines();
                self.parse_paragraph();
            }
            // Don't forget to eat *trailing* whitespace
            self.skip_ws_and_newlines();
            // Close the root node.
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
        fn skip_ws_and_newlines(&mut self) {
            while self.current() == Some(WHITESPACE)
                || self.current() == Some(COMMENT)
                || self.current() == Some(NEWLINE)
            {
                self.builder.start_node(EMPTY_LINE.into());
                while self.current() != Some(NEWLINE) && self.current().is_some() {
                    self.bump();
                }
                if self.current() == Some(NEWLINE) {
                    self.bump();
                }
                self.builder.finish_node();
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

    fn root(&self) -> Deb822 {
        Deb822::cast(self.syntax()).unwrap()
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

        impl AstNode for $ast {
            type Language = Lang;

            fn can_cast(kind: SyntaxKind) -> bool {
                kind == $kind
            }

            fn cast(syntax: SyntaxNode) -> Option<Self> {
                Self::cast(syntax)
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }

        impl ToString for $ast {
            fn to_string(&self) -> String {
                self.0.text().to_string()
            }
        }
    };
}

impl std::fmt::Debug for Deb822 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deb822").finish()
    }
}

ast_node!(Deb822, ROOT);
ast_node!(Paragraph, PARAGRAPH);
ast_node!(Entry, ENTRY);

impl Default for Deb822 {
    fn default() -> Self {
        Self::new()
    }
}

impl Deb822 {
    pub fn new() -> Deb822 {
        let mut builder = GreenNodeBuilder::new();

        builder.start_node(ROOT.into());
        builder.finish_node();
        Deb822(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    /// Returns an iterator over all paragraphs in the file.
    pub fn paragraphs(&self) -> impl Iterator<Item = Paragraph> {
        self.0.children().filter_map(Paragraph::cast)
    }

    /// Add a new empty paragraph to the end of the file.
    pub fn add_paragraph(&mut self) -> Paragraph {
        let paragraph = Paragraph::new();
        let mut to_insert = vec![];
        if self.0.children().count() > 0 {
            let mut builder = GreenNodeBuilder::new();
            builder.start_node(EMPTY_LINE.into());
            builder.token(NEWLINE.into(), "\n");
            builder.finish_node();
            to_insert.push(SyntaxNode::new_root(builder.finish()).clone_for_update().into());
        }
        to_insert.push(paragraph.0.clone().into());
        self.0.splice_children(
            self.0.children().count()..self.0.children().count(),
            to_insert
        );
        paragraph
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&text)?)
    }

    pub fn from_file_relaxed(path: impl AsRef<Path>) -> Result<(Self, Vec<String>), std::io::Error> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_str_relaxed(&text))
    }

    pub fn from_str_relaxed(s: &str) -> (Self, Vec<String>) {
        let parsed = parse(s);
        (parsed.root(), parsed.errors)
    }
}

impl Paragraph {
    pub fn new() -> Paragraph {
        let mut builder = GreenNodeBuilder::new();

        builder.start_node(PARAGRAPH.into());
        builder.finish_node();
        Paragraph(SyntaxNode::new_root(builder.finish()).clone_for_update())
    }

    /// Returns the value of the given key in the paragraph.
    pub fn get(&self, key: &str) -> Option<String> {
        self.entries()
            .find(|e| e.key().as_deref() == Some(key))
            .map(|e| e.value())
    }

    /// Returns whether the paragraph contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Returns an iterator over all entries in the paragraph.
    fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.0.children().filter_map(Entry::cast)
    }

    /// Returns an iterator over all items in the paragraph.
    pub fn items(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.entries()
            .filter_map(|e| e.key().map(|k| (k, e.value())))
    }

    /// Returns an iterator over all values for the given key in the paragraph.
    pub fn get_all<'a>(&'a self, key: &'a str) -> impl Iterator<Item = String> + '_ {
        self.items().filter_map(move |(k, v)| if k.as_str() == key { Some(v) } else { None })
    }

    #[deprecated(note = "use `contains_key` instead")]
    /// Returns true if the paragraph contains the given key.
    pub fn contains(&self, key: &str) -> bool {
        self.get_all(key).any(|_| true)
    }

    /// Returns an iterator over all keys in the paragraph.
    pub fn keys(&self) -> impl Iterator<Item = String> + '_ {
        self.entries().filter_map(|e| e.key())
    }

    /// Remove the given field from the paragraph.
    pub fn remove(&mut self, key: &str) {
        for mut entry in self.entries() {
            if entry.key().as_deref() == Some(key) {
                entry.detach();
            }
        }
    }

    /// Add a new field to the paragraph.
    pub fn insert(&mut self, key: &str, value: &str) {
        for mut entry in self.entries() {
            if entry.key().as_deref() == Some(key) {
                entry.set_value(value);
                return;
            }
        }
        let entry = Entry::new(key, value);
        self.0.splice_children(
            self.0.children().count()..self.0.children().count(),
            vec![entry.0.clone_for_update().into()],
        );
    }

    /// Rename the given field in the paragraph.
    pub fn rename(&mut self, old_key: &str, new_key: &str) {
        for mut entry in self.entries() {
            if entry.key().as_deref() == Some(old_key) {
                entry.set_key(new_key);
            }
        }
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for Paragraph {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let deb822 = Deb822::from_str(text)?;

        let mut paragraphs = deb822.paragraphs();

        paragraphs.next().ok_or_else(||ParseError(vec!["no paragraphs".to_string()]))
    }
}

impl Entry {
    pub fn new(key: &str, value: &str) -> Entry {
        let mut builder = GreenNodeBuilder::new();

        builder.start_node(ENTRY.into());
        builder.token(KEY.into(), key);
        builder.token(COLON.into(), ":");
        builder.token(WHITESPACE.into(), " ");
        builder.token(VALUE.into(), value);
        builder.token(NEWLINE.into(), "\n");
        builder.finish_node();
        Entry(SyntaxNode::new_root(builder.finish()))
    }

    pub fn key(&self) -> Option<String> {
        self.0
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind() == KEY)
            .map(|it| it.text().to_string())
    }

    pub fn value(&self) -> String {
        self.0
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .filter(|it| it.kind() == VALUE)
            .map(|it| it.text().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn set_key(&mut self, _key: &str) {
        todo!();
    }

    pub fn set_value(&mut self, _value: &str) {
        todo!();
    }

    pub fn detach(&mut self) {
        self.0.detach();
    }
}

impl FromStr for Deb822 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = parse(s);
        if parsed.errors.is_empty() {
            Ok(parsed.root().clone_for_update())
        } else {
            Err(ParseError(parsed.errors))
        }
    }
}

#[test]
fn test_parse_simple() {
    const CONTROLV1: &str = r#"Source: foo
Maintainer: Foo Bar <foo@example.com>
Section: net

# This is a comment

Package: foo
Architecture: all
Depends:
 bar,
 blah
Description: This is a description
 And it is
 .
 multiple
 lines
"#;
    let parsed = parse(CONTROLV1);
    let node = parsed.syntax();
    assert_eq!(
        format!("{:#?}", node),
        r###"ROOT@0..203
  PARAGRAPH@0..63
    ENTRY@0..12
      KEY@0..6 "Source"
      COLON@6..7 ":"
      WHITESPACE@7..8 " "
      VALUE@8..11 "foo"
      NEWLINE@11..12 "\n"
    ENTRY@12..50
      KEY@12..22 "Maintainer"
      COLON@22..23 ":"
      WHITESPACE@23..24 " "
      VALUE@24..49 "Foo Bar <foo@example. ..."
      NEWLINE@49..50 "\n"
    ENTRY@50..63
      KEY@50..57 "Section"
      COLON@57..58 ":"
      WHITESPACE@58..59 " "
      VALUE@59..62 "net"
      NEWLINE@62..63 "\n"
  EMPTY_LINE@63..64
    NEWLINE@63..64 "\n"
  EMPTY_LINE@64..84
    COMMENT@64..83 "# This is a comment"
    NEWLINE@83..84 "\n"
  EMPTY_LINE@84..85
    NEWLINE@84..85 "\n"
  PARAGRAPH@85..203
    ENTRY@85..98
      KEY@85..92 "Package"
      COLON@92..93 ":"
      WHITESPACE@93..94 " "
      VALUE@94..97 "foo"
      NEWLINE@97..98 "\n"
    ENTRY@98..116
      KEY@98..110 "Architecture"
      COLON@110..111 ":"
      WHITESPACE@111..112 " "
      VALUE@112..115 "all"
      NEWLINE@115..116 "\n"
    ENTRY@116..137
      KEY@116..123 "Depends"
      COLON@123..124 ":"
      NEWLINE@124..125 "\n"
      INDENT@125..126 " "
      VALUE@126..130 "bar,"
      NEWLINE@130..131 "\n"
      INDENT@131..132 " "
      VALUE@132..136 "blah"
      NEWLINE@136..137 "\n"
    ENTRY@137..203
      KEY@137..148 "Description"
      COLON@148..149 ":"
      WHITESPACE@149..150 " "
      VALUE@150..171 "This is a description"
      NEWLINE@171..172 "\n"
      INDENT@172..173 " "
      VALUE@173..182 "And it is"
      NEWLINE@182..183 "\n"
      INDENT@183..184 " "
      VALUE@184..185 "."
      NEWLINE@185..186 "\n"
      INDENT@186..187 " "
      VALUE@187..195 "multiple"
      NEWLINE@195..196 "\n"
      INDENT@196..197 " "
      VALUE@197..202 "lines"
      NEWLINE@202..203 "\n"
"###
    );
    assert_eq!(parsed.errors, Vec::<String>::new());

    let root = parsed.root();
    assert_eq!(root.paragraphs().count(), 2);
    let source = root.paragraphs().next().unwrap();
    assert_eq!(
        source.keys().collect::<Vec<_>>(),
        vec!["Source", "Maintainer", "Section"]
    );
    assert_eq!(source.get("Source").as_deref(), Some("foo"));
    assert_eq!(
        source.get("Maintainer").as_deref(),
        Some("Foo Bar <foo@example.com>")
    );
    assert_eq!(source.get("Section").as_deref(), Some("net"));
    assert_eq!(
        source.items().collect::<Vec<_>>(),
        vec![
            ("Source".into(), "foo".into()),
            ("Maintainer".into(), "Foo Bar <foo@example.com>".into()),
            ("Section".into(), "net".into()),
        ]
    );

    let binary = root.paragraphs().nth(1).unwrap();
    assert_eq!(
        binary.keys().collect::<Vec<_>>(),
        vec!["Package", "Architecture", "Depends", "Description"]
    );
    assert_eq!(binary.get("Package").as_deref(), Some("foo"));
    assert_eq!(binary.get("Architecture").as_deref(), Some("all"));
    assert_eq!(binary.get("Depends").as_deref(), Some("bar,\nblah"));
    assert_eq!(
        binary.get("Description").as_deref(),
        Some("This is a description\nAnd it is\n.\nmultiple\nlines")
    );

    assert_eq!(node.text(), CONTROLV1);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        let d: super::Deb822 = r#"Source: foo
Maintainer: Foo Bar <jelmer@jelmer.uk>
Section: net

Package: foo
Architecture: all
Depends: libc6
Description: This is a description
 With details
 "#
        .parse()
        .unwrap();
        let mut ps = d.paragraphs();
        let p = ps.next().unwrap();

        assert_eq!(p.get("Source").as_deref(), Some("foo"));
        assert_eq!(
            p.get("Maintainer").as_deref(),
            Some("Foo Bar <jelmer@jelmer.uk>")
        );
        assert_eq!(p.get("Section").as_deref(), Some("net"));

        let b = ps.next().unwrap();
        assert_eq!(b.get("Package").as_deref(), Some("foo"));
    }

    #[test]
    fn test_modify() {
        let d: super::Deb822 = r#"Source: foo
Maintainer: Foo Bar <jelmer@jelmer.uk>
Section: net

Package: foo
Architecture: all
Depends: libc6
Description: This is a description
 With details
 "#
        .parse()
        .unwrap();
        let mut ps = d.paragraphs();
        let mut p = ps.next().unwrap();
        p.insert("Foo", "Bar");
        p.remove("Section");
        p.remove("Nonexistant");
        assert_eq!(p.get("Foo").as_deref(), Some("Bar"));
        assert_eq!(
            p.to_string(),
            r#"Source: foo
Maintainer: Foo Bar <jelmer@jelmer.uk>
Foo: Bar
"#
        );
    }

    #[test]
    fn test_add_paragraph() {
        let mut d = super::Deb822::new();
        let mut p = d.add_paragraph();
        p.insert("Foo", "Bar");
        assert_eq!(p.get("Foo").as_deref(), Some("Bar"));
        assert_eq!(p.to_string(), r#"Foo: Bar
"#);
        assert_eq!(d.to_string(), r#"Foo: Bar
"#);

        let mut p = d.add_paragraph();
        p.insert("Foo", "Blah");
        assert_eq!(p.get("Foo").as_deref(), Some("Blah"));
        assert_eq!(d.to_string(), r#"Foo: Bar

Foo: Blah
"#);
    }

}
