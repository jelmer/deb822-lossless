use crate::deb822::SyntaxKind;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    start_of_line: bool,
    indent: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            start_of_line: true,
            indent: 0,
        }
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t'
    }

    fn is_newline(c: char) -> bool {
        c == '\n' || c == '\r'
    }

    fn is_valid_key_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
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
                _ if Self::is_newline(c) => {
                    self.input.next();
                    self.start_of_line = true;
                    self.indent = 0;
                    Some((SyntaxKind::NEWLINE, c.to_string()))
                }
                _ if Self::is_whitespace(c) => {
                    let whitespace = self.read_while(Self::is_whitespace);
                    if self.start_of_line {
                        self.indent = whitespace.len();
                        Some((SyntaxKind::INDENT, whitespace))
                    } else {
                        Some((SyntaxKind::WHITESPACE, whitespace))
                    }
                }
                '#' if self.start_of_line => {
                    self.input.next();
                    let comment = self.read_while(|c| c != '\n' && c != '\r');
                    self.start_of_line = true;
                    Some((SyntaxKind::COMMENT, format!("#{}", comment)))
                }
                _ if Self::is_valid_key_char(c) && self.start_of_line && self.indent == 0 => {
                    let key = self.read_while(Self::is_valid_key_char);
                    self.start_of_line = false;
                    Some((SyntaxKind::KEY, key))
                }
                _ if !self.start_of_line || self.indent > 0 => {
                    let value = self.read_while(|c| !Self::is_newline(c));
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
    type Item = (crate::deb822::SyntaxKind, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub(crate) fn lex(input: &str) -> Vec<(SyntaxKind, String)> {
    let mut lexer = Lexer::new(input);
    lexer.by_ref().collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::deb822::SyntaxKind::*;
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
}
