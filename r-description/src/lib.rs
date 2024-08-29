/// A library for parsing and manipulating R DESCRIPTION files.
///
/// See https://r-pkgs.org/description.html for more information.
///
/// See the ``lossless`` module for a lossless parser that is
/// forgiving in the face of errors and preserves formatting while editing
/// at the expense of a more complex API.

use deb822_lossless::{FromDeb822, ToDeb822, FromDeb822Paragraph, ToDeb822Paragraph};

#[derive(Debug, PartialEq, Eq)]
pub struct RCode(String);

impl std::str::FromStr for RCode {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl std::fmt::Display for RCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(FromDeb822, ToDeb822, Debug, PartialEq, Eq)]
pub struct RDescription {
    #[deb822(field = "Package")]
    pub name: String,

    #[deb822(field = "Description")]
    pub description: String,

    #[deb822(field = "Title")]
    pub title: String,

    #[deb822(field = "Maintainer")]
    pub maintainer: Option<String>,

    #[deb822(field = "Authors@R")]
    pub authors: Option<RCode>,

    #[deb822(field = "Version")]
    pub version: Option<String>,

    #[deb822(field = "Encoding")]
    pub encoding: Option<String>,

    #[deb822(field = "License")]
    pub license: Option<String>,

    #[deb822(field = "URL")]
    pub url: Option<url::Url>,

    #[deb822(field = "BugReports")]
    pub bug_reports: Option<String>,

    #[deb822(field = "Imports")]
    pub imports: Option<String>,

    #[deb822(field = "Suggests")]
    pub suggests: Option<String>,

    #[deb822(field = "Depends")]
    pub depends: Option<String>,

    #[deb822(field = "LinkingTo")]
    pub linking_to: Option<String>,

    #[deb822(field = "LazyData")]
    pub lazy_data: Option<String>,

    #[deb822(field = "Collate")]
    pub collate: Option<String>,

    #[deb822(field = "VignetteBuilder")]
    pub vignette_builder: Option<String>,

    #[deb822(field = "SystemRequirements")]
    pub system_requirements: Option<String>,

    #[deb822(field = "Date")]
    pub date: Option<String>,
}

impl std::str::FromStr for RDescription {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let para: deb822_lossless::Paragraph = s.parse().map_err(|e: deb822_lossless::ParseError| e.to_string())?;
        Ok(Self::from_paragraph(&para)?)
    }
}

impl ToString for RDescription {
    fn to_string(&self) -> String {
        self.to_paragraph().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r###"Package: mypackage
Title: What the Package Does (One Line, Title Case)
Version: 0.0.0.9000
Authors@R: 
    person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
           comment = c(ORCID = "YOUR-ORCID-ID"))
Description: What the package does (one paragraph).
License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a
    license
Encoding: UTF-8
Roxygen: list(markdown = TRUE)
RoxygenNote: 7.3.2
"###;
        let desc: RDescription = s.parse().unwrap();

        assert_eq!(desc.name, "mypackage".to_string());
        assert_eq!(desc.title, "What the Package Does (One Line, Title Case)".to_string());
        assert_eq!(desc.version, Some("0.0.0.9000".to_string()));
        assert_eq!(desc.authors, Some(RCode(r#"person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
comment = c(ORCID = "YOUR-ORCID-ID"))"#.to_string())));
        assert_eq!(desc.description, "What the package does (one paragraph).".to_string());
        assert_eq!(desc.license, Some("`use_mit_license()`, `use_gpl3_license()` or friends to pick a\nlicense".to_string()));
        assert_eq!(desc.encoding, Some("UTF-8".to_string()));

        assert_eq!(desc.to_string(), r###"Package: mypackage
Description: What the package does (one paragraph).
Title: What the Package Does (One Line, Title Case)
Authors@R: person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
 comment = c(ORCID = "YOUR-ORCID-ID"))
Version: 0.0.0.9000
Encoding: UTF-8
License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a
 license
"###);
    }
}
