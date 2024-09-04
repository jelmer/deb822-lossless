pub fn glob_to_regex(glob: &str) -> regex::Regex {
    let mut it = glob.chars();
    let mut r = String::new();

    while let Some(c) = it.next() {
        r.push_str(
            match c {
                '*' => ".*".to_string(),
                '?' => ".".to_string(),
                '\\' => match it.next().unwrap() {
                    '?' | '*' | '\\' => regex::escape(c.to_string().as_str()),
                    x => {
                        panic!("invalid escape sequence: \\{}", x);
                    }
                },
                c => regex::escape(c.to_string().as_str()),
            }
            .as_str(),
        )
    }

    regex::Regex::new(r.as_str()).unwrap()
}


