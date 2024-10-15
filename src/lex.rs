use crate::common;

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

fn lex_(mut input: &str, mut start_of_line: bool) -> impl Iterator<Item = (SyntaxKind, &str)> {
    let mut colon_count = if start_of_line { 0 } else { 1 };
    let mut indent = 0;

    std::iter::from_fn(move || {
        if let Some(c) = input.chars().next() {
            match c {
                ':' if colon_count == 0 => {
                    colon_count += 1;
                    input = &input[1..];
                    Some((SyntaxKind::COLON, ":"))
                }
                _ if common::is_newline(c) => {
                    let (nl, remaining) = input.split_at(1);
                    input = remaining;
                    start_of_line = true;
                    colon_count = 0;
                    indent = 0;
                    Some((SyntaxKind::NEWLINE, nl))
                }
                _ if common::is_indent(c) => {
                    let (whitespace, remaining) = input
                        .split_at(input.find(|c| !common::is_indent(c)).unwrap_or(input.len()));
                    input = remaining;
                    if start_of_line {
                        indent = whitespace.len();
                        Some((SyntaxKind::INDENT, whitespace))
                    } else {
                        Some((SyntaxKind::WHITESPACE, whitespace))
                    }
                }
                '#' if start_of_line => {
                    let (comment, remaining) =
                        input.split_at(input.find(common::is_newline).unwrap_or(input.len()));
                    input = remaining;
                    start_of_line = true;
                    colon_count = 0;
                    Some((SyntaxKind::COMMENT, comment))
                }
                _ if common::is_valid_initial_key_char(c) && start_of_line && indent == 0 => {
                    let (key, remaining) = input.split_at(
                        input
                            .find(|c| !common::is_valid_key_char(c))
                            .unwrap_or(input.len()),
                    );
                    input = remaining;
                    start_of_line = false;
                    Some((SyntaxKind::KEY, key))
                }
                _ if !start_of_line || indent > 0 => {
                    let (value, remaining) =
                        input.split_at(input.find(common::is_newline).unwrap_or(input.len()));
                    input = remaining;
                    Some((SyntaxKind::VALUE, value))
                }
                _ => {
                    let (text, remaining) = input.split_at(1);
                    input = remaining;
                    Some((SyntaxKind::ERROR, text))
                }
            }
        } else {
            None
        }
    })
}

pub(crate) fn lex(input: &str) -> impl Iterator<Item = (SyntaxKind, &str)> {
    lex_(input, true)
}

pub(crate) fn lex_inline(input: &str) -> impl Iterator<Item = (SyntaxKind, &str)> {
    lex_(input, false)
}

#[cfg(test)]
mod tests {
    use super::SyntaxKind::*;
    #[test]
    fn test_empty() {
        assert_eq!(super::lex("").collect::<Vec<_>>(), vec![]);
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
        assert_eq!(tokens.collect::<Vec<_>>(), vec![(VALUE, "syncthing-gtk")]);
    }

    #[test]
    fn test_lex_odd_key_characters() {
        let text = "foo-bar: baz\n";

        let tokens = super::lex(text);

        assert_eq!(
            tokens.collect::<Vec<_>>(),
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
