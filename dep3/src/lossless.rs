//! A library for parsing and generating Debian patch headers.
//!
//! # Examples
//!
//! ```rust
//! use dep3::lossless::PatchHeader;
//! use std::str::FromStr;
//! let text = r#"From: John Doe <john.doe@example>
//! Date: Mon, 1 Jan 2000 00:00:00 +0000
//! Subject: [PATCH] fix a bug
//! Bug-Debian: https://bugs.debian.org/123456
//! Bug: https://bugzilla.example.com/bug.cgi?id=123456
//! Forwarded: not-needed
//! "#;
//!
//! let patch_header = PatchHeader::from_str(text).unwrap();
//! assert_eq!(patch_header.description(), Some("[PATCH] fix a bug".to_string()));
//! assert_eq!(patch_header.vendor_bugs("Debian").collect::<Vec<_>>(), vec!["https://bugs.debian.org/123456".to_string()]);
//! ```
use deb822_lossless::Paragraph;

use crate::fields::*;

/// A Debian patch header.
pub struct PatchHeader(Paragraph);

impl PatchHeader {
    /// Create a new, empty patch header.
    pub fn new() -> Self {
        PatchHeader(Paragraph::new())
    }

    /// Get a reference to the underlying `Paragraph`.
    pub fn as_deb822(&self) -> &Paragraph {
        &self.0
    }

    /// Get a mutable reference to the underlying `Paragraph`, mutably.
    pub fn as_deb822_mut(&mut self) -> &mut Paragraph {
        &mut self.0
    }

    /// The origin of the patch.
    pub fn origin(&self) -> Option<(Option<OriginCategory>, Origin)> {
        self.0
            .get("Origin")
            .as_deref()
            .map(crate::fields::parse_origin)
    }

    /// Set the origin of the patch.
    pub fn set_origin(&mut self, category: Option<OriginCategory>, origin: Origin) {
        self.0.insert(
            "Origin",
            crate::fields::format_origin(&category, &origin).as_str(),
        );
    }

    /// The `Forwarded` field.
    pub fn forwarded(&self) -> Option<Forwarded> {
        self.0
            .get("Forwarded")
            .as_deref()
            .map(|s| s.parse().unwrap())
    }

    /// Set the `Forwarded` field.
    pub fn set_forwarded(&mut self, forwarded: Forwarded) {
        self.0.insert("Forwarded", forwarded.to_string().as_str());
    }

    /// The author of the patch.
    pub fn author(&self) -> Option<String> {
        self.0.get("Author").or_else(|| self.0.get("From"))
    }

    /// Set the author of the patch.
    pub fn set_author(&mut self, author: &str) {
        if self.0.contains_key("From") {
            self.0.insert("From", author);
        } else {
            self.0.insert("Author", author);
        }
    }

    /// The `Reviewed-By` field.
    pub fn reviewed_by(&self) -> Vec<String> {
        self.0.get_all("Reviewed-By").collect()
    }

    /// Get the last update date of the patch.
    pub fn last_update(&self) -> Option<chrono::NaiveDate> {
        self.0
            .get("Last-Update")
            .as_deref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Set the date of the last update
    pub fn set_last_update(&mut self, date: chrono::NaiveDate) {
        self.0
            .insert("Last-Update", date.format("%Y-%m-%d").to_string().as_str());
    }

    /// The `Applied-Upstream` field.
    pub fn applied_upstream(&self) -> Option<AppliedUpstream> {
        self.0
            .get("Applied-Upstream")
            .as_deref()
            .map(|s| s.parse().unwrap())
    }

    /// Set the `Applied-Upstream` field.
    pub fn set_applied_upstream(&mut self, applied_upstream: AppliedUpstream) {
        self.0
            .insert("Applied-Upstream", applied_upstream.to_string().as_str());
    }

    /// Get the bugs associated with the patch.
    pub fn bugs(&self) -> impl Iterator<Item = (Option<String>, String)> + '_ {
        self.0.items().filter_map(|(k, v)| {
            if k.starts_with("Bug-") {
                Some((Some(k.strip_prefix("Bug-").unwrap().to_string()), v))
            } else if k == "Bug" {
                Some((None, v))
            } else {
                None
            }
        })
    }

    /// Get the bugs associated with a specific vendor.
    pub fn vendor_bugs<'a>(&'a self, vendor: &'a str) -> impl Iterator<Item = String> + '_ {
        self.bugs().filter_map(|(k, v)| {
            if k == Some(vendor.to_string()) {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Set the upstream bug associated with the patch.
    pub fn set_upstream_bug(&mut self, bug: &str) {
        self.0.insert("Bug", bug);
    }

    /// Set the bug associated with a specific vendor.
    pub fn set_vendor_bug(&mut self, vendor: &str, bug: &str) {
        self.0.insert(format!("Bug-{}", vendor).as_str(), bug);
    }

    /// Get the description or subject field.
    fn description_field(&self) -> Option<String> {
        self.0.get("Description").or_else(|| self.0.get("Subject"))
    }

    /// Get the description of the patch.
    pub fn description(&self) -> Option<String> {
        self.description_field()
            .as_deref()
            .map(|s| s.split('\n').next().unwrap_or(s).to_string())
    }

    /// Set the description of the patch.
    pub fn set_description(&mut self, description: &str) {
        if let Some(subject) = self.0.get("Subject") {
            // Replace the first line with ours
            let new = format!(
                "{}\n{}",
                description,
                subject.split_once('\n').map(|x| x.1).unwrap_or("")
            );
            self.0.insert("Subject", new.as_str());
        } else if let Some(description) = self.0.get("Description") {
            // Replace the first line with ours
            let new = format!(
                "{}\n{}",
                description.split_once('\n').map(|x| x.1).unwrap_or(""),
                description
            );
            self.0.insert("Description", new.as_str());
        } else {
            self.0.insert("Description", description);
        }
    }

    /// Get the long description of the patch.
    pub fn long_description(&self) -> Option<String> {
        self.description_field()
            .as_deref()
            .map(|s| s.split_once('\n').map(|x| x.1).unwrap_or("").to_string())
    }

    /// Set the long description of the patch.
    pub fn set_long_description(&mut self, long_description: &str) {
        if let Some(subject) = self.0.get("Subject") {
            // Keep the first line, but replace the rest with our text
            let first_line = subject
                .split_once('\n')
                .map(|x| x.0)
                .unwrap_or(subject.as_str());
            let new = format!("{}\n{}", first_line, long_description);
            self.0.insert("Subject", new.as_str());
        } else if let Some(description) = self.0.get("Description") {
            // Keep the first line, but replace the rest with our text
            let first_line = description
                .split_once('\n')
                .map(|x| x.0)
                .unwrap_or(description.as_str());
            let new = format!("{}\n{}", first_line, long_description);
            self.0.insert("Description", new.as_str());
        } else {
            self.0.insert("Description", long_description);
        }
    }

    /// Write the patch header
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.to_string().as_bytes())
    }
}

impl std::fmt::Display for PatchHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl Default for PatchHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for PatchHeader {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PatchHeader(Paragraph::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::PatchHeader;
    use std::str::FromStr;

    #[test]
    fn test_upstream() {
        let text = r#"From: Ulrich Drepper <drepper@redhat.com>
Subject: Fix regex problems with some multi-bytes characters
 
 * posix/bug-regex17.c: Add testcases.
 * posix/regcomp.c (re_compile_fastmap_iter): Rewrite COMPLEX_BRACKET
   handling.
 
Origin: upstream, http://sourceware.org/git/?p=glibc.git;a=commitdiff;h=bdb56bac
Bug: http://sourceware.org/bugzilla/show_bug.cgi?id=9697
Bug-Debian: http://bugs.debian.org/510219
"#;

        let header = PatchHeader::from_str(text).unwrap();

        assert_eq!(
            header.origin(),
            Some((
                Some(super::OriginCategory::Upstream),
                super::Origin::Other(
                    "http://sourceware.org/git/?p=glibc.git;a=commitdiff;h=bdb56bac".to_string()
                )
            ))
        );
        assert_eq!(header.forwarded(), None);
        assert_eq!(
            header.author(),
            Some("Ulrich Drepper <drepper@redhat.com>".to_string())
        );
        assert_eq!(header.reviewed_by(), Vec::<&str>::new());
        assert_eq!(header.last_update(), None);
        assert_eq!(header.applied_upstream(), None);
        assert_eq!(
            header.bugs().collect::<Vec<_>>(),
            vec![
                (
                    None,
                    "http://sourceware.org/bugzilla/show_bug.cgi?id=9697".to_string()
                ),
                (
                    Some("Debian".to_string()),
                    "http://bugs.debian.org/510219".to_string()
                ),
            ]
        );
        assert_eq!(
            header.description(),
            Some("Fix regex problems with some multi-bytes characters".to_string())
        );
    }

    #[test]
    fn test_forwarded() {
        let text = r#"Description: Use FHS compliant paths by default
 Upstream is not interested in switching to those paths.
 .
 But we will continue using them in Debian nevertheless to comply with
 our policy.
Forwarded: http://lists.example.com/oct-2006/1234.html
Author: John Doe <johndoe-guest@users.alioth.debian.org>
Last-Update: 2006-12-21
"#;
        let header = PatchHeader::from_str(text).unwrap();

        assert_eq!(header.origin(), None);
        assert_eq!(
            header.forwarded(),
            Some(super::Forwarded::Yes(
                "http://lists.example.com/oct-2006/1234.html".to_string()
            ))
        );
        assert_eq!(
            header.author(),
            Some("John Doe <johndoe-guest@users.alioth.debian.org>".to_string())
        );
        assert_eq!(header.reviewed_by(), Vec::<&str>::new());
        assert_eq!(
            header.last_update(),
            Some(chrono::NaiveDate::from_ymd_opt(2006, 12, 21).unwrap())
        );
        assert_eq!(header.applied_upstream(), None);
        assert_eq!(header.bugs().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            header.description(),
            Some("Use FHS compliant paths by default".to_string())
        );
    }

    #[test]
    fn test_not_forwarded() {
        let text = r#"Description: Workaround for broken symbol resolving on mips/mipsel
 The correct fix will be done in etch and it will require toolchain
 fixes.
Forwarded: not-needed
Origin: vendor, http://bugs.debian.org/cgi-bin/bugreport.cgi?msg=80;bug=265678
Bug-Debian: http://bugs.debian.org/265678
Author: Thiemo Seufer <ths@debian.org>
"#;

        let header = PatchHeader::from_str(text).unwrap();

        assert_eq!(
            header.origin(),
            Some((
                Some(super::OriginCategory::Vendor),
                super::Origin::Other(
                    "http://bugs.debian.org/cgi-bin/bugreport.cgi?msg=80;bug=265678".to_string()
                )
            ))
        );
        assert_eq!(header.forwarded(), Some(super::Forwarded::NotNeeded));
        assert_eq!(
            header.author(),
            Some("Thiemo Seufer <ths@debian.org>".to_string())
        );
        assert_eq!(header.reviewed_by(), Vec::<&str>::new());
        assert_eq!(header.last_update(), None);
        assert_eq!(header.applied_upstream(), None);
        assert_eq!(
            header.bugs().collect::<Vec<_>>(),
            vec![(
                Some("Debian".to_string()),
                "http://bugs.debian.org/265678".to_string()
            ),]
        );

        assert_eq!(
            header.description(),
            Some("Workaround for broken symbol resolving on mips/mipsel".to_string())
        );
    }

    #[test]
    fn test_applied_upstream() {
        let text = r#"Description: Fix widget frobnication speeds
 Frobnicating widgets too quickly tended to cause explosions.
Forwarded: http://lists.example.com/2010/03/1234.html
Author: John Doe <johndoe-guest@users.alioth.debian.org>
Applied-Upstream: 1.2, http://bzr.example.com/frobnicator/trunk/revision/123
Last-Update: 2010-03-29
"#;
        let header = PatchHeader::from_str(text).unwrap();

        assert_eq!(header.origin(), None);
        assert_eq!(
            header.forwarded(),
            Some(super::Forwarded::Yes(
                "http://lists.example.com/2010/03/1234.html".to_string()
            ))
        );
        assert_eq!(
            header.author(),
            Some("John Doe <johndoe-guest@users.alioth.debian.org>".to_string())
        );
        assert_eq!(header.reviewed_by(), Vec::<&str>::new());
        assert_eq!(
            header.last_update(),
            Some(chrono::NaiveDate::from_ymd_opt(2010, 3, 29).unwrap())
        );
        assert_eq!(
            header.applied_upstream(),
            Some(super::AppliedUpstream::Other(
                "1.2, http://bzr.example.com/frobnicator/trunk/revision/123".to_string()
            ))
        );
        assert_eq!(header.bugs().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            header.description(),
            Some("Fix widget frobnication speeds".to_string())
        );
    }

    #[test]
    fn test_vendor_bugs() {
        let text = r#"Description: Fix widget frobnication speeds
Bug: http://bugs.example.com/123
Bug-Debian: http://bugs.debian.org/123
Bug-Ubuntu: http://bugs.launchpad.net/123
"#;

        let header = PatchHeader::from_str(&text).unwrap();

        assert_eq!(
            header.vendor_bugs("Debian").collect::<Vec<_>>(),
            vec!["http://bugs.debian.org/123".to_string()]
        );
        assert_eq!(
            header.vendor_bugs("Ubuntu").collect::<Vec<_>>(),
            vec!["http://bugs.launchpad.net/123".to_string()]
        );
    }
}
