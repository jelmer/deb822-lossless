pub struct Changes(deb822_lossless::Paragraph);

#[derive(Debug)]
pub enum ParseError {
    Deb822(deb822_lossless::Error),
    NoParagraphs,
    MultipleParagraphs,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Deb822(e) => write!(f, "{}", e),
            Self::NoParagraphs => write!(f, "no paragraphs found"),
            Self::MultipleParagraphs => write!(f, "multiple paragraphs found"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<deb822_lossless::Error> for ParseError {
    fn from(e: deb822_lossless::Error) -> Self {
        Self::Deb822(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct File {
    pub md5sum: String,
    pub size: usize,
    pub section: String,
    pub priority: crate::Priority,
    pub filename: String,
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.md5sum, self.size, self.section, self.priority, self.filename
        )
    }
}

impl std::str::FromStr for File {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let md5sum = parts.next().ok_or(())?;
        let size = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let section = parts.next().ok_or(())?.to_string();
        let priority = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let filename = parts.next().ok_or(())?.to_string();
        Ok(Self {
            md5sum: md5sum.to_string(),
            size,
            section,
            priority,
            filename,
        })
    }
}

impl Changes {
    pub fn format(&self) -> Option<String> {
        self.0.get("Format").map(|s| s.to_string())
    }

    pub fn set_format(&mut self, value: &str) {
        self.0.insert("Format", value);
    }

    pub fn source(&self) -> Option<String> {
        self.0.get("Source").map(|s| s.to_string())
    }

    pub fn binary(&self) -> Option<Vec<String>> {
        self.0
            .get("Binary")
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
    }

    pub fn architecture(&self) -> Option<Vec<String>> {
        self.0
            .get("Architecture")
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
    }

    pub fn version(&self) -> Option<debversion::Version> {
        self.0.get("Version").map(|s| s.parse().unwrap())
    }

    pub fn distribution(&self) -> Option<String> {
        self.0.get("Distribution").map(|s| s.to_string())
    }

    pub fn urgency(&self) -> Option<crate::fields::Urgency> {
        self.0.get("Urgency").map(|s| s.parse().unwrap())
    }

    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer").map(|s| s.to_string())
    }

    pub fn changed_by(&self) -> Option<String> {
        self.0.get("Changed-By").map(|s| s.to_string())
    }

    pub fn description(&self) -> Option<String> {
        self.0.get("Description").map(|s| s.to_string())
    }

    pub fn checksums_sha1(&self) -> Option<Vec<crate::fields::Sha1Checksum>> {
        self.0
            .get("Checksums-Sha1")
            .map(|s| s.lines().map(|line| line.parse().unwrap()).collect())
    }

    pub fn checksums_sha256(&self) -> Option<Vec<crate::fields::Sha256Checksum>> {
        self.0
            .get("Checksums-Sha256")
            .map(|s| s.lines().map(|line| line.parse().unwrap()).collect())
    }

    /// Returns the list of files in the source package.
    pub fn files(&self) -> Option<Vec<File>> {
        self.0
            .get("Files")
            .map(|s| s.lines().map(|line| line.parse().unwrap()).collect())
    }

    /// Returns the path to the pool directory for the source package.
    pub fn get_pool_path(&self) -> Option<String> {
        let files = self.files()?;

        let section = &files.first().unwrap().section;

        let section = if let Some((section, _subsection)) = section.split_once('/') {
            section
        } else {
            "main"
        };

        let source = self.source()?;

        let subdir = if source.starts_with("lib") {
            "lib"
        } else {
            &source.chars().next().unwrap().to_string()
        };

        Some(format!("pool/{}/{}/{}", section, subdir, source))
    }

    pub fn new() -> Self {
        let mut slf = Self(deb822_lossless::Paragraph::new());
        slf.set_format("1.8");
        slf
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ParseError> {
        let deb822 = deb822_lossless::Deb822::from_file(path)?;
        let mut paras = deb822.paragraphs();
        let para = match paras.next() {
            Some(para) => para,
            None => return Err(ParseError::NoParagraphs),
        };
        if paras.next().is_some() {
            return Err(ParseError::MultipleParagraphs);
        }
        Ok(Self(para))
    }

    pub fn from_file_relaxed<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<(Self, Vec<String>), std::io::Error> {
        let (mut deb822, mut errors) = deb822_lossless::Deb822::from_file_relaxed(path)?;
        let mut paras = deb822.paragraphs();
        let para = match paras.next() {
            Some(para) => para,
            None => deb822.add_paragraph(),
        };
        if paras.next().is_some() {
            errors.push("multiple paragraphs found".to_string());
        }
        Ok((Self(para), errors))
    }

    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, ParseError> {
        let deb822 = deb822_lossless::Deb822::read(&mut r)?;
        let mut paras = deb822.paragraphs();
        let para = match paras.next() {
            Some(para) => para,
            None => return Err(ParseError::NoParagraphs),
        };
        if paras.next().is_some() {
            return Err(ParseError::MultipleParagraphs);
        }
        Ok(Self(para))
    }

    pub fn read_relaxed<R: std::io::Read>(
        mut r: R,
    ) -> Result<(Self, Vec<String>), deb822_lossless::Error> {
        let (mut deb822, mut errors) = deb822_lossless::Deb822::read_relaxed(&mut r)?;
        let mut paras = deb822.paragraphs();
        let para = match paras.next() {
            Some(para) => para,
            None => deb822.add_paragraph(),
        };
        if paras.next().is_some() {
            errors.push("multiple paragraphs found".to_string());
        }
        Ok((Self(para), errors))
    }
}

impl Default for Changes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Changes {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        self.0.to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Changes {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Changes(ob.extract()?))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new() {
        let changes = super::Changes::new();
        assert_eq!(changes.format(), Some("1.8".to_string()));
    }

    #[test]
    fn test_parse() {
        let changes = r#"Format: 1.8
Date: Fri, 08 Sep 2023 18:23:59 +0100
Source: buildlog-consultant
Binary: python3-buildlog-consultant
Architecture: all
Version: 0.0.34-1
Distribution: unstable
Urgency: medium
Maintainer: Jelmer Vernooĳ <jelmer@debian.org>
Changed-By: Jelmer Vernooĳ <jelmer@debian.org>
Description:
 python3-buildlog-consultant - build log parser and analyser
Changes:
 buildlog-consultant (0.0.34-1) UNRELEASED; urgency=medium
 .
   * New upstream release.
   * Update standards version to 4.6.2, no changes needed.
Checksums-Sha1:
 f1657e628254428ad74542e82c253a181894e8d0 17153 buildlog-consultant_0.0.34-1_amd64.buildinfo
 b44493c05d014bcd59180942d0125b20ddf45d03 2550812 python3-buildlog-consultant_0.0.34-1_all.deb
Checksums-Sha256:
 342a5782bf6a4f282d9002f726d2cac9c689c7e0fa7f61a1b0ecbf4da7916bdb 17153 buildlog-consultant_0.0.34-1_amd64.buildinfo
 7f7e5df81ee23fbbe89015edb37e04f4bb40672fa6e9b1afd4fd698e57db78fd 2550812 python3-buildlog-consultant_0.0.34-1_all.deb
Files:
 aa83112b0f8774a573bcf0b7b5cc12cc 17153 python optional buildlog-consultant_0.0.34-1_amd64.buildinfo
 a55858b90fe0ca728c89c1a1132b45c5 2550812 python optional python3-buildlog-consultant_0.0.34-1_all.deb
"#;
        let changes = super::Changes::read(changes.as_bytes()).unwrap();
        assert_eq!(changes.format(), Some("1.8".to_string()));
        assert_eq!(changes.source(), Some("buildlog-consultant".to_string()));
        assert_eq!(
            changes.binary(),
            Some(vec!["python3-buildlog-consultant".to_string()])
        );
        assert_eq!(changes.architecture(), Some(vec!["all".to_string()]));
        assert_eq!(changes.version(), Some("0.0.34-1".parse().unwrap()));
        assert_eq!(changes.distribution(), Some("unstable".to_string()));
        assert_eq!(changes.urgency(), Some(crate::fields::Urgency::Medium));
        assert_eq!(
            changes.maintainer(),
            Some("Jelmer Vernooĳ <jelmer@debian.org>".to_string())
        );
        assert_eq!(
            changes.changed_by(),
            Some("Jelmer Vernooĳ <jelmer@debian.org>".to_string())
        );
        assert_eq!(
            changes.description(),
            Some("python3-buildlog-consultant - build log parser and analyser".to_string())
        );
        assert_eq!(
            changes.checksums_sha1(),
            Some(vec![
                "f1657e628254428ad74542e82c253a181894e8d0 17153 buildlog-consultant_0.0.34-1_amd64.buildinfo".parse().unwrap(),
                "b44493c05d014bcd59180942d0125b20ddf45d03 2550812 python3-buildlog-consultant_0.0.34-1_all.deb".parse().unwrap()
            ])
        );
        assert_eq!(
            changes.checksums_sha256(),
            Some(vec![
                "342a5782bf6a4f282d9002f726d2cac9c689c7e0fa7f61a1b0ecbf4da7916bdb 17153 buildlog-consultant_0.0.34-1_amd64.buildinfo"
                    .parse()
                    .unwrap(),
                "7f7e5df81ee23fbbe89015edb37e04f4bb40672fa6e9b1afd4fd698e57db78fd 2550812 python3-buildlog-consultant_0.0.34-1_all.deb"
                    .parse()
                    .unwrap()
            ])
        );
        assert_eq!(
            changes.files(),
            Some(vec![
                "aa83112b0f8774a573bcf0b7b5cc12cc 17153 python optional buildlog-consultant_0.0.34-1_amd64.buildinfo".parse().unwrap(),
                "a55858b90fe0ca728c89c1a1132b45c5 2550812 python optional python3-buildlog-consultant_0.0.34-1_all.deb".parse().unwrap()
            ])
        );

        assert_eq!(
            changes.get_pool_path(),
            Some("pool/main/b/buildlog-consultant".to_string())
        );
    }
}
