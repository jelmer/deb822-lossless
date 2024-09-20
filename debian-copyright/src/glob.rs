pub fn glob_to_regex(glob: &str) -> regex::Regex {
    let mut it = glob.chars();
    let mut r = "^".to_string();

    while let Some(c) = it.next() {
        r.push_str(
            match c {
                '*' => ".*".to_string(),
                '?' => ".".to_string(),
                '\\' => {
                    let c = it.next();
                    match c {
                        Some('?') | Some('*') | Some('\\') => regex::escape(c.unwrap().to_string().as_str()),
                        Some(x) => {
                            panic!("invalid escape sequence: \\{}", x);
                        }
                        None => {
                            panic!("invalid escape sequence: \\");
                        }
                    }
                },
                c => regex::escape(c.to_string().as_str()),
            }
            .as_str(),
        )
    }

    r.push_str("$");

    regex::Regex::new(r.as_str()).unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple() {
        let r = super::glob_to_regex("*.rs");
        assert!(r.is_match("foo.rs"));
        assert!(r.is_match("bar.rs"));
        assert!(!r.is_match("foo.rs.bak"));
        assert!(!r.is_match("foo"));
    }

    #[test]
    fn test_single_char() {
        let r = super::glob_to_regex("?.rs");
        assert!(r.is_match("a.rs"));
        assert!(r.is_match("b.rs"));
        assert!(!r.is_match("foo.rs"));
        assert!(!r.is_match("foo"));
    }

    #[test]
    fn test_escape() {
        let r = super::glob_to_regex(r"\?.rs");
        assert!(r.is_match("?.rs"));
        assert!(!r.is_match("a.rs"));
        assert!(!r.is_match("b.rs"));

        let r = super::glob_to_regex(r"\*.rs");
        assert!(r.is_match("*.rs"));
        assert!(!r.is_match("a.rs"));
        assert!(!r.is_match("b.rs"));

        let r = super::glob_to_regex(r"\\?.rs");
        assert!(r.is_match("\\a.rs"));
        assert!(r.is_match("\\b.rs"));
        assert!(!r.is_match("a.rs"));
    }

    #[should_panic]
    #[test]
    fn test_invalid_escape() {
        super::glob_to_regex(r"\x.rs");
    }

    #[should_panic]
    #[test]
    fn test_invalid_escape2() {
        super::glob_to_regex(r"\");
    }
}
