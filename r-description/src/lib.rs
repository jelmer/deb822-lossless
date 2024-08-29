/// A library for parsing and manipulating R DESCRIPTION files.
///
/// See https://r-pkgs.org/description.html for more information.

use deb822_lossless::Paragraph;

pub struct RDescription(Paragraph);

impl std::fmt::Display for RDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for RDescription {
    fn default() -> Self {
        Self(Paragraph::new())
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(deb822_lossless::ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<deb822_lossless::ParseError> for Error {
    fn from(e: deb822_lossless::ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::str::FromStr for RDescription {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Paragraph::from_str(s)?))
    }
}

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

pub struct Relations(String);

impl RDescription {
    pub fn new() -> Self {
        Self(Paragraph::new())
    }

    pub fn package(&self) -> Option<String> {
        self.0.get("Package")
    }
    pub fn set_package(&mut self, package: &str) {
        self.0.insert("Package", package);
    }

    /// One line description of the package, and is often shown in a package listing
    ///
    /// It should be plain text (no markup), capitalised like a title, and NOT end in a period.
    /// Keep it short: listings will often truncate the title to 65 characters.
    pub fn title(&self) -> Option<String> {
        self.0.get("Title")
    }

    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer")
    }

    pub fn set_maintainer(&mut self, maintainer: &str) {
        self.0.insert("Maintainer", maintainer);
    }

    pub fn authors(&self) -> Option<RCode> {
        self.0.get("Authors@R").map(|s| s.parse().unwrap())
    }

    pub fn set_authors(&mut self, authors: &RCode) {
        self.0.insert("Authors@R", &authors.to_string());
    }

    pub fn set_title(&mut self, title: &str) {
        self.0.insert("Title", title);
    }

    pub fn description(&self) -> Option<String> {
        self.0.get("Description")
    }

    pub fn set_description(&mut self, description: &str) {
        self.0.insert("Description", description);
    }

    pub fn version(&self) -> Option<String> {
        self.0.get("Version")
    }

    pub fn set_version(&mut self, version: &str) {
        self.0.insert("Version", version);
    }

    pub fn encoding(&self) -> Option<String> {
        self.0.get("Encoding")
    }

    pub fn set_encoding(&mut self, encoding: &str) {
        self.0.insert("Encoding", encoding);
    }

    pub fn license(&self) -> Option<String> {
        self.0.get("License")
    }

    pub fn set_license(&mut self, license: &str) {
        self.0.insert("License", license);
    }

    pub fn roxygen_note(&self) -> Option<String> {
        self.0.get("RoxygenNote")
    }

    pub fn set_roxygen_note(&mut self, roxygen_note: &str) {
        self.0.insert("RoxygenNote", roxygen_note);
    }

    pub fn roxygen(&self) -> Option<String> {
        self.0.get("Roxygen")
    }

    pub fn set_roxygen(&mut self, roxygen: &str) {
        self.0.insert("Roxygen", roxygen);
    }

    pub fn url(&self) -> Option<url::Url> {
        self.0.get("URL").as_ref().map(|s| url::Url::parse(s.as_str()).unwrap())
    }

    pub fn set_url(&mut self, url: &url::Url) {
        self.0.insert("URL", url.as_str());
    }

    pub fn bug_reports(&self) -> Option<url::Url> {
        self.0.get("BugReports").map(|s| url::Url::parse(s.as_str()).unwrap())
    }

    pub fn set_bug_reports(&mut self, bug_reports: &url::Url) {
        self.0.insert("BugReports", bug_reports.as_str());
    }

    pub fn imports(&self) -> Option<Vec<String>> {
        self.0.get("Imports").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_imports(&mut self, imports: &[&str]) {
        self.0.insert("Imports", &imports.join(", "));
    }

    pub fn suggests(&self) -> Option<Vec<String>> {
        self.0.get("Suggests").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_suggests(&mut self, suggests: &[&str]) {
        self.0.insert("Suggests", &suggests.join(", "));
    }

    pub fn depends(&self) -> Option<Vec<String>> {
        self.0.get("Depends").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_depends(&mut self, depends: &[&str]) {
        self.0.insert("Depends", &depends.join(", "));
    }

    pub fn linking_to(&self) -> Option<Vec<String>> {
        self.0.get("LinkingTo").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_linking_to(&mut self, linking_to: &[&str]) {
        self.0.insert("LinkingTo", &linking_to.join(", "));
    }

    pub fn lazy_data(&self) -> Option<bool> {
        self.0.get("LazyData").map(|s| s == "true")
    }

    pub fn set_lazy_data(&mut self, lazy_data: bool) {
        self.0.insert("LazyData", if lazy_data { "true" } else { "false" });
    }

    pub fn collate(&self) -> Option<String> {
        self.0.get("Collate")
    }

    pub fn set_collate(&mut self, collate: &str) {
        self.0.insert("Collate", collate);
    }


    pub fn vignette_builder(&self) -> Option<Vec<String>> {
        self.0.get("VignetteBuilder").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_vignette_builder(&mut self, vignette_builder: &[&str]) {
        self.0.insert("VignetteBuilder", &vignette_builder.join(", "));
    }

    pub fn system_requirements(&self) -> Option<Vec<String>> {
        self.0.get("SystemRequirements").map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_system_requirements(&mut self, system_requirements: &[&str]) {
        self.0.insert("SystemRequirements", &system_requirements.join(", "));
    }

    pub fn date(&self) -> Option<String> {
        self.0.get("Date")
    }

    pub fn set_date(&mut self, date: &str) {
        self.0.insert("Date", date);
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

        assert_eq!(desc.package(), Some("mypackage".to_string()));
        assert_eq!(desc.title(), Some("What the Package Does (One Line, Title Case)".to_string()));
        assert_eq!(desc.version(), Some("0.0.0.9000".to_string()));
        assert_eq!(desc.authors(), Some(RCode(r#"person("First", "Last", , "first.last@example.com", role = c("aut", "cre"),
comment = c(ORCID = "YOUR-ORCID-ID"))"#.to_string())));
        assert_eq!(desc.description(), Some("What the package does (one paragraph).".to_string()));
        assert_eq!(desc.license(), Some("`use_mit_license()`, `use_gpl3_license()` or friends to pick a\nlicense".to_string()));
        assert_eq!(desc.encoding(), Some("UTF-8".to_string()));
        assert_eq!(desc.roxygen(), Some("list(markdown = TRUE)".to_string()));
        assert_eq!(desc.roxygen_note(), Some("7.3.2".to_string()));

        assert_eq!(desc.to_string(), s);
    }
}
