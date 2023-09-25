use crate::SyntaxKind;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    start_of_line: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            start_of_line: true,
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
                    Some((SyntaxKind::NEWLINE, c.to_string()))
                }
                _ if Self::is_whitespace(c) => {
                    let whitespace = self.read_while(Self::is_whitespace);
                    if self.start_of_line {
                        Some((SyntaxKind::INDENT, whitespace))
                    } else {
                        Some((SyntaxKind::WHITESPACE, whitespace))
                    }
                }
                '#' => {
                    self.input.next();
                    let comment = self.read_while(|c| c != '\n' && c != '\r');
                    Some((SyntaxKind::COMMENT, format!("#{}", comment)))
                }
                _ if Self::is_valid_key_char(c) && self.start_of_line => {
                    let key = self.read_while(Self::is_valid_key_char);
                    self.start_of_line = false;
                    Some((SyntaxKind::KEY, key))
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
