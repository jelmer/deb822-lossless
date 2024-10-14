use crate::common;
use std::iter::Peekable;
use std::str::Chars;

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
    ROOT,       // The entire file
    PARAGRAPH,  // A deb822 paragraph
    ENTRY,      // A single key-value pair
    EMPTY_LINE, // An empty line
}

/// Convert our `SyntaxKind` into the rowan `SyntaxKind`.
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    start_of_line: bool,
    indent: usize,
    colon_count: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            start_of_line: true,
            colon_count: 0,
            indent: 0,
        }
    }

    pub fn new_inline(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            start_of_line: false,
            colon_count: 1,
            indent: 0,
        }
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
                ':' if self.colon_count == 0 => {
                    self.colon_count += 1;
                    self.input.next();
                    Some((SyntaxKind::COLON, ":".to_owned()))
                }
                _ if common::is_newline(c) => {
                    self.input.next();
                    self.start_of_line = true;
                    self.colon_count = 0;
                    self.indent = 0;
                    Some((SyntaxKind::NEWLINE, c.to_string()))
                }
                _ if common::is_indent(c) => {
                    let whitespace = self.read_while(common::is_indent);
                    if self.start_of_line {
                        self.indent = whitespace.len();
                        Some((SyntaxKind::INDENT, whitespace))
                    } else {
                        Some((SyntaxKind::WHITESPACE, whitespace))
                    }
                }
                '#' if self.start_of_line => {
                    self.input.next();
                    let comment = self.read_while(|c| !common::is_newline(c));
                    self.start_of_line = true;
                    self.colon_count = 0;
                    Some((SyntaxKind::COMMENT, format!("#{}", comment)))
                }
                _ if common::is_valid_initial_key_char(c)
                    && self.start_of_line
                    && self.indent == 0 =>
                {
                    let key = self.read_while(common::is_valid_key_char);
                    self.start_of_line = false;
                    Some((SyntaxKind::KEY, key))
                }
                _ if !self.start_of_line || self.indent > 0 => {
                    let value = self.read_while(|c| !common::is_newline(c));
                    Some((SyntaxKind::VALUE, value))
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

pub(crate) fn lex_inline(input: &str) -> Vec<(SyntaxKind, String)> {
    let mut lexer = Lexer::new_inline(input);
    lexer.by_ref().collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::SyntaxKind::*;
    #[test]
    fn test_empty() {
        assert_eq!(super::lex(""), vec![]);
    }

    #[test]
    fn test_simple() {
        assert_eq!(
            super::lex(
                r#"Source: syncthing-gtk
Maintainer: Jelmer Vernooĳ <jelmer@jelmer.uk>
Section:    net   

# This is the first binary package:

Package: syncthing-gtk
Architecture: all
Depends: 
  foo,
  bar,
  blah (= 1.0)
Description: a package
 with a loooong
 .
 long
 .
 description
"#
            )
            .iter()
            .map(|(kind, text)| (*kind, text.as_str()))
            .collect::<Vec<_>>(),
            vec![
                (KEY, "Source"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "syncthing-gtk"),
                (NEWLINE, "\n"),
                (KEY, "Maintainer"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "Jelmer Vernooĳ <jelmer@jelmer.uk>"),
                (NEWLINE, "\n"),
                (KEY, "Section"),
                (COLON, ":"),
                (WHITESPACE, "    "),
                (VALUE, "net   "),
                (NEWLINE, "\n"),
                (NEWLINE, "\n"),
                (COMMENT, "# This is the first binary package:"),
                (NEWLINE, "\n"),
                (NEWLINE, "\n"),
                (KEY, "Package"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "syncthing-gtk"),
                (NEWLINE, "\n"),
                (KEY, "Architecture"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "all"),
                (NEWLINE, "\n"),
                (KEY, "Depends"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (NEWLINE, "\n"),
                (INDENT, "  "),
                (VALUE, "foo,"),
                (NEWLINE, "\n"),
                (INDENT, "  "),
                (VALUE, "bar,"),
                (NEWLINE, "\n"),
                (INDENT, "  "),
                (VALUE, "blah (= 1.0)"),
                (NEWLINE, "\n"),
                (KEY, "Description"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "a package"),
                (NEWLINE, "\n"),
                (INDENT, " "),
                (VALUE, "with a loooong"),
                (NEWLINE, "\n"),
                (INDENT, " "),
                (VALUE, "."),
                (NEWLINE, "\n"),
                (INDENT, " "),
                (VALUE, "long"),
                (NEWLINE, "\n"),
                (INDENT, " "),
                (VALUE, "."),
                (NEWLINE, "\n"),
                (INDENT, " "),
                (VALUE, "description"),
                (NEWLINE, "\n")
            ]
        );
    }

    #[test]
    fn test_apt() {
        let text = r#"Package: cvsd
Binary: cvsd
Version: 1.0.24
Maintainer: Arthur de Jong <adejong@debian.org>
Build-Depends: debhelper (>= 9), po-debconf
Architecture: any
Standards-Version: 3.9.3
Format: 3.0 (native)
Files:
 b7a7d67a02974c52c408fdb5e118406d 890 cvsd_1.0.24.dsc
 b73ee40774c3086cb8490cdbb96ac883 258139 cvsd_1.0.24.tar.gz
Vcs-Browser: http://arthurdejong.org/viewvc/cvsd/
# A comment
Vcs-Cvs: :pserver:anonymous@arthurdejong.org:/arthur/
Checksums-Sha256:
 a7bb7a3aacee19cd14ce5c26cb86e348b1608e6f1f6e97c6ea7c58efa440ac43 890 cvsd_1.0.24.dsc
 46bc517760c1070ae408693b89603986b53e6f068ae6bdc744e2e830e46b8cba 258139 cvsd_1.0.24.tar.gz
Homepage: http://arthurdejong.org/cvsd/
Package-List:
 cvsd deb vcs optional
Directory: pool/main/c/cvsd
Priority: source
Section: vcs

"#;
        let tokens = super::lex(text);
        assert_eq!(
            tokens
                .iter()
                .map(|(kind, text)| (*kind, text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (KEY, "Package"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "cvsd"), (NEWLINE, "\n"),
                (KEY, "Binary"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "cvsd"), (NEWLINE, "\n"),
                (KEY, "Version"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "1.0.24"), (NEWLINE, "\n"),
                (KEY, "Maintainer"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "Arthur de Jong <adejong@debian.org>"), (NEWLINE, "\n"),
                (KEY, "Build-Depends"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "debhelper (>= 9), po-debconf"), (NEWLINE, "\n"),
                (KEY, "Architecture"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "any"), (NEWLINE, "\n"),
                (KEY, "Standards-Version"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "3.9.3"), (NEWLINE, "\n"),
                (KEY, "Format"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "3.0 (native)"), (NEWLINE, "\n"),
                (KEY, "Files"), (COLON, ":"), (NEWLINE, "\n"), (INDENT, " "), (VALUE, "b7a7d67a02974c52c408fdb5e118406d 890 cvsd_1.0.24.dsc"), (NEWLINE, "\n"), (INDENT, " "), (VALUE, "b73ee40774c3086cb8490cdbb96ac883 258139 cvsd_1.0.24.tar.gz"), (NEWLINE, "\n"),
                (KEY, "Vcs-Browser"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "http://arthurdejong.org/viewvc/cvsd/"), (NEWLINE, "\n"),
                (COMMENT, "# A comment"), (NEWLINE, "\n"),
                (KEY, "Vcs-Cvs"), (COLON, ":"), (WHITESPACE, " "), (VALUE, ":pserver:anonymous@arthurdejong.org:/arthur/"), (NEWLINE, "\n"),
                (KEY, "Checksums-Sha256"), (COLON, ":"), (NEWLINE, "\n"), (INDENT, " "), (VALUE, "a7bb7a3aacee19cd14ce5c26cb86e348b1608e6f1f6e97c6ea7c58efa440ac43 890 cvsd_1.0.24.dsc"), (NEWLINE, "\n"), (INDENT, " "), (VALUE, "46bc517760c1070ae408693b89603986b53e6f068ae6bdc744e2e830e46b8cba 258139 cvsd_1.0.24.tar.gz"), (NEWLINE, "\n"),
                (KEY, "Homepage"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "http://arthurdejong.org/cvsd/"), (NEWLINE, "\n"),
                (KEY, "Package-List"), (COLON, ":"), (NEWLINE, "\n"), (INDENT, " "), (VALUE, "cvsd deb vcs optional"), (NEWLINE, "\n"),
                (KEY, "Directory"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "pool/main/c/cvsd"), (NEWLINE, "\n"),
                (KEY, "Priority"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "source"), (NEWLINE, "\n"),
                (KEY, "Section"), (COLON, ":"), (WHITESPACE, " "), (VALUE, "vcs"), (NEWLINE, "\n"), (NEWLINE, "\n")
            ]
        );
    }

    #[test]
    fn test_lex_inline() {
        let text = r"syncthing-gtk";
        let tokens = super::lex_inline(text);
        assert_eq!(
            tokens
                .iter()
                .map(|(kind, text)| (*kind, text.as_str()))
                .collect::<Vec<_>>(),
            vec![(VALUE, "syncthing-gtk")]
        );
    }

    #[test]
    fn test_lex_odd_key_characters() {
        let text = "foo-bar: baz\n";

        let tokens = super::lex(text);

        assert_eq!(
            tokens
                .iter()
                .map(|(kind, text)| (*kind, text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (KEY, "foo-bar"),
                (COLON, ":"),
                (WHITESPACE, " "),
                (VALUE, "baz"),
                (NEWLINE, "\n")
            ]
        );
    }
}
