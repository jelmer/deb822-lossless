use dep3::PatchHeader;
use std::str::FromStr;

pub const TEXT: &str = r#"From: John Doe <john.doe@example>
Date: Mon, 1 Jan 2000 00:00:00 +0000
Subject: [PATCH] fix a bug
Bug-Debian: https://bugs.debian.org/123456
Bug: https://bugzilla.example.com/bug.cgi?id=123456
Forwarded: not-needed
"#;

pub fn main() {
    let patch_header = PatchHeader::from_str(TEXT).unwrap();

    println!("Description: {}", patch_header.description().unwrap());
    println!("Debian Bugs: {}", patch_header.vendor_bugs("Debian").collect::<Vec<_>>().join(", "));
}
