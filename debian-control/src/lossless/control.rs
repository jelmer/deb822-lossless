use crate::fields::{MultiArch, Priority};
use crate::lossless::relations::Relations;

fn format_field(name: &str, value: &str) -> String {
    match name {
        "Uploaders" => value
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
            .join(",\n"),
        "Build-Depends" | "Build-Depends-Indep" | "Build-Depends-Arch" | "Build-Conflicts" | "Build-Conflicts-Indep" | "Build-Conflics-Arch" | "Depends" | "Recommends" | "Suggests" | "Enhances" | "Pre-Depends" | "Breaks" => {
            let relations: Relations = value.parse().unwrap();
            let relations = relations.wrap_and_sort();
            relations.to_string()
        },
        _ => value.to_string(),
    }
}

pub struct Control(deb822_lossless::Deb822);

impl Control {
    pub fn new() -> Self {
        Control(deb822_lossless::Deb822::new())
    }

    pub fn as_mut_deb822(&mut self) -> &mut deb822_lossless::Deb822 {
        &mut self.0
    }

    pub fn as_deb822(&self) -> &deb822_lossless::Deb822 {
        &self.0
    }

    pub fn source(&self) -> Option<Source> {
        self.0
            .paragraphs()
            .find(|p| p.get("Source").is_some())
            .map(Source)
    }

    pub fn binaries(&self) -> impl Iterator<Item = Binary> {
        self.0
            .paragraphs()
            .filter(|p| p.get("Package").is_some())
            .map(Binary)
    }

    /// Add a new source package
    ///
    /// # Arguments
    /// * `name` - The name of the source package
    ///
    /// # Returns
    /// The newly created source package
    ///
    /// # Example
    /// ```rust
    /// use debian_control::lossless::control::Control;
    /// let mut control = Control::new();
    /// let source = control.add_source("foo");
    /// assert_eq!(source.name(), Some("foo".to_owned()));
    /// ```
    pub fn add_source(&mut self, name: &str) -> Source {
        let mut p = self.0.add_paragraph();
        p.insert("Source", name);
        self.source().unwrap()
    }

    /// Add new binary package
    ///
    /// # Arguments
    /// * `name` - The name of the binary package
    ///
    /// # Returns
    /// The newly created binary package
    ///
    /// # Example
    /// ```rust
    /// use debian_control::lossless::control::Control;
    /// let mut control = Control::new();
    /// let binary = control.add_binary("foo");
    /// assert_eq!(binary.name(), Some("foo".to_owned()));
    /// ```
    pub fn add_binary(&mut self, name: &str) -> Binary {
        let mut p = self.0.add_paragraph();
        p.insert("Package", name);
        Binary(p)
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, deb822_lossless::Error> {
        Ok(Control(deb822_lossless::Deb822::from_file(path)?))
    }

    pub fn from_file_relaxed<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<(Self, Vec<String>), std::io::Error> {
        let (control, errors) = deb822_lossless::Deb822::from_file_relaxed(path)?;
        Ok((Control(control), errors))
    }

    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, deb822_lossless::Error> {
        Ok(Control(deb822_lossless::Deb822::read(&mut r)?))
    }

    pub fn read_relaxed<R: std::io::Read>(
        mut r: R,
    ) -> Result<(Self, Vec<String>), deb822_lossless::Error> {
        let (control, errors) = deb822_lossless::Deb822::read_relaxed(&mut r)?;
        Ok((Self(control), errors))
    }

    pub fn wrap_and_sort(&mut self, indentation: deb822_lossless::Indentation, immediate_empty_line: bool, max_line_length_one_liner: Option<usize>) {
        let sort_paragraphs = |a: &deb822_lossless::Paragraph, b: &deb822_lossless::Paragraph| -> std::cmp::Ordering {
            // Sort Source before Package
            let a_is_source = a.get("Source").is_some();
            let b_is_source = b.get("Source").is_some();

            if a_is_source && !b_is_source {
                return std::cmp::Ordering::Less;
            } else if !a_is_source && b_is_source {
                return std::cmp::Ordering::Greater;
            } else if a_is_source && b_is_source {
                return a.get("Source").cmp(&b.get("Source"));
            }

            a.get("Package").cmp(&b.get("Package"))
        };

        let wrap_paragraph = |p: &deb822_lossless::Paragraph| -> deb822_lossless::Paragraph {
            // TODO: Add Source/Package specific wrapping
            // TODO: Add support for wrapping and sorting fields
            p.wrap_and_sort(indentation, immediate_empty_line, max_line_length_one_liner, None, Some(&format_field))
        };

        self.0 = self.0.wrap_and_sort(Some(&sort_paragraphs), Some(&wrap_paragraph));
    }
}

impl From<Control> for deb822_lossless::Deb822 {
    fn from(c: Control) -> Self {
        c.0
    }
}

impl From<deb822_lossless::Deb822> for Control {
    fn from(d: deb822_lossless::Deb822) -> Self {
        Control(d)
    }
}

impl Default for Control {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for Control {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Control(s.parse()?))
    }
}

pub struct Source(deb822_lossless::Paragraph);

impl From<Source> for deb822_lossless::Paragraph {
    fn from(s: Source) -> Self {
        s.0
    }
}

impl From<deb822_lossless::Paragraph> for Source {
    fn from(p: deb822_lossless::Paragraph) -> Self {
        Source(p)
    }
}

impl Source {
    /// The name of the source package.
    pub fn name(&self) -> Option<String> {
        self.0.get("Source")
    }

    pub fn wrap_and_sort(&mut self, indentation: deb822_lossless::Indentation, immediate_empty_line: bool, max_line_length_one_liner: Option<usize>) {
        self.0 = self.0.wrap_and_sort(
            indentation,
            immediate_empty_line,
            max_line_length_one_liner,
            None,
            Some(&format_field),
        );
    }

    pub fn as_mut_deb822(&mut self) -> &mut deb822_lossless::Paragraph {
        &mut self.0
    }

    pub fn as_deb822(&self) -> &deb822_lossless::Paragraph {
        &self.0
    }

    pub fn set_name(&mut self, name: &str) {
        self.0.insert("Source", name);
    }

    /// The default section of the packages built from this source package.
    pub fn section(&self) -> Option<String> {
        self.0.get("Section")
    }

    pub fn set_section(&mut self, section: Option<&str>) {
        if let Some(section) = section {
            self.0.insert("Section", section);
        } else {
            self.0.remove("Section");
        }
    }

    /// The default priority of the packages built from this source package.
    pub fn priority(&self) -> Option<Priority> {
        self.0.get("Priority").and_then(|v| v.parse().ok())
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        if let Some(priority) = priority {
            self.0.insert("Priority", priority.to_string().as_str());
        } else {
            self.0.remove("Priority");
        }
    }

    /// The maintainer of the package.
    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer")
    }

    pub fn set_maintainer(&mut self, maintainer: &str) {
        self.0.insert("Maintainer", maintainer);
    }

    /// The build dependencies of the package.
    pub fn build_depends(&self) -> Option<Relations> {
        self.0.get("Build-Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_build_depends(&mut self, relations: &Relations) {
        self.0
            .insert("Build-Depends", relations.to_string().as_str());
    }

    pub fn build_depends_indep(&self) -> Option<Relations> {
        self.0
            .get("Build-Depends-Indep")
            .map(|s| s.parse().unwrap())
    }

    pub fn build_depends_arch(&self) -> Option<Relations> {
        self.0.get("Build-Depends-Arch").map(|s| s.parse().unwrap())
    }

    pub fn build_conflicts(&self) -> Option<Relations> {
        self.0.get("Build-Conflicts").map(|s| s.parse().unwrap())
    }

    pub fn build_conflicts_indep(&self) -> Option<Relations> {
        self.0
            .get("Build-Conflicts-Indep")
            .map(|s| s.parse().unwrap())
    }

    pub fn build_conflicts_arch(&self) -> Option<Relations> {
        self.0
            .get("Build-Conflicts-Arch")
            .map(|s| s.parse().unwrap())
    }

    pub fn standards_version(&self) -> Option<String> {
        self.0.get("Standards-Version")
    }

    pub fn set_standards_version(&mut self, version: &str) {
        self.0.insert("Standards-Version", version);
    }

    pub fn homepage(&self) -> Option<url::Url> {
        self.0.get("Homepage").and_then(|s| s.parse().ok())
    }

    pub fn set_homepage(&mut self, homepage: &url::Url) {
        self.0.insert("Homepage", homepage.to_string().as_str());
    }

    pub fn vcs_git(&self) -> Option<String> {
        self.0.get("Vcs-Git")
    }

    pub fn set_vcs_git(&mut self, url: &str) {
        self.0.insert("Vcs-Git", url);
    }

    pub fn vcs_svn(&self) -> Option<String> {
        self.0.get("Vcs-Svn").map(|s| s.to_string())
    }

    pub fn set_vcs_svn(&mut self, url: &str) {
        self.0.insert("Vcs-Svn", url);
    }

    pub fn vcs_bzr(&self) -> Option<String> {
        self.0.get("Vcs-Bzr").map(|s| s.to_string())
    }

    pub fn set_vcs_bzr(&mut self, url: &str) {
        self.0.insert("Vcs-Bzr", url);
    }

    pub fn vcs_arch(&self) -> Option<String> {
        self.0.get("Vcs-Arch").map(|s| s.to_string())
    }

    pub fn set_vcs_arch(&mut self, url: &str) {
        self.0.insert("Vcs-Arch", url);
    }

    pub fn vcs_svk(&self) -> Option<String> {
        self.0.get("Vcs-Svk").map(|s| s.to_string())
    }

    pub fn set_vcs_svk(&mut self, url: &str) {
        self.0.insert("Vcs-Svk", url);
    }

    pub fn vcs_darcs(&self) -> Option<String> {
        self.0.get("Vcs-Darcs").map(|s| s.to_string())
    }

    pub fn set_vcs_darcs(&mut self, url: &str) {
        self.0.insert("Vcs-Darcs", url);
    }

    pub fn vcs_mtn(&self) -> Option<String> {
        self.0.get("Vcs-Mtn").map(|s| s.to_string())
    }

    pub fn set_vcs_mtn(&mut self, url: &str) {
        self.0.insert("Vcs-Mtn", url);
    }

    pub fn vcs_cvs(&self) -> Option<String> {
        self.0.get("Vcs-Cvs").map(|s| s.to_string())
    }

    pub fn set_vcs_cvs(&mut self, url: &str) {
        self.0.insert("Vcs-Cvs", url);
    }

    pub fn vcs_hg(&self) -> Option<String> {
        self.0.get("Vcs-Hg").map(|s| s.to_string())
    }

    pub fn set_vcs_hg(&mut self, url: &str) {
        self.0.insert("Vcs-Hg", url);
    }

    pub fn vcs_browser(&self) -> Option<String> {
        self.0.get("Vcs-Browser")
    }

    pub fn set_vcs_browser(&mut self, url: Option<&str>) {
        if let Some(url) = url {
            self.0.insert("Vcs-Browser", url);
        } else {
            self.0.remove("Vcs-Browser");
        }
    }

    pub fn uploaders(&self) -> Option<Vec<String>> {
        self.0
            .get("Uploaders")
            .map(|s| s.split(',').map(|s| s.trim().to_owned()).collect())
    }

    pub fn set_uploaders(&mut self, uploaders: &[&str]) {
        self.0.insert(
            "Uploaders",
            uploaders
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
    }

    pub fn architecture(&self) -> Option<String> {
        self.0.get("Architecture")
    }

    pub fn set_architecture(&mut self, arch: Option<&str>) {
        if let Some(arch) = arch {
            self.0.insert("Architecture", arch);
        } else {
            self.0.remove("Architecture");
        }
    }

    pub fn rules_requires_root(&self) -> Option<bool> {
        self.0
            .get("Rules-Requires-Root")
            .map(|s| match s.to_lowercase().as_str() {
                "yes" => true,
                "no" => false,
                _ => panic!("invalid Rules-Requires-Root value"),
            })
    }

    pub fn testsuite(&self) -> Option<String> {
        self.0.get("Testsuite")
    }

    pub fn set_testsuite(&mut self, testsuite: &str) {
        self.0.insert("Testsuite", testsuite);
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Source {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        self.0.to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Source {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Source(ob.extract()?))
    }
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Binary(deb822_lossless::Paragraph);

impl From<Binary> for deb822_lossless::Paragraph {
    fn from(b: Binary) -> Self {
        b.0
    }
}

impl From<deb822_lossless::Paragraph> for Binary {
    fn from(p: deb822_lossless::Paragraph) -> Self {
        Binary(p)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Binary {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        self.0.to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Binary {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Binary(ob.extract()?))
    }
}

impl Binary {
    pub fn new() -> Self {
        Binary(deb822_lossless::Paragraph::new())
    }

    pub fn as_mut_deb822(&mut self) -> &mut deb822_lossless::Paragraph {
        &mut self.0
    }

    pub fn as_deb822(&self) -> &deb822_lossless::Paragraph {
        &self.0
    }

    pub fn wrap_and_sort(&mut self, indentation: deb822_lossless::Indentation, immediate_empty_line: bool, max_line_length_one_liner: Option<usize>) {
        self.0 = self.0.wrap_and_sort(
            indentation,
            immediate_empty_line,
            max_line_length_one_liner,
            None,
            Some(&format_field),
        );
    }

    /// The name of the package.
    pub fn name(&self) -> Option<String> {
        self.0.get("Package")
    }

    pub fn set_name(&mut self, name: &str) {
        self.0.insert("Package", name);
    }

    /// The section of the package.
    pub fn section(&self) -> Option<String> {
        self.0.get("Section")
    }

    pub fn set_section(&mut self, section: Option<&str>) {
        if let Some(section) = section {
            self.0.insert("Section", section);
        } else {
            self.0.remove("Section");
        }
    }

    /// The priority of the package.
    pub fn priority(&self) -> Option<Priority> {
        self.0.get("Priority").and_then(|v| v.parse().ok())
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        if let Some(priority) = priority {
            self.0.insert("Priority", priority.to_string().as_str());
        } else {
            self.0.remove("Priority");
        }
    }

    /// The architecture of the package.
    pub fn architecture(&self) -> Option<String> {
        self.0.get("Architecture")
    }

    pub fn set_architecture(&mut self, arch: Option<&str>) {
        if let Some(arch) = arch {
            self.0.insert("Architecture", arch);
        } else {
            self.0.remove("Architecture");
        }
    }

    /// The dependencies of the package.
    pub fn depends(&self) -> Option<Relations> {
        self.0.get("Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_depends(&mut self, depends: Option<&Relations>) {
        if let Some(depends) = depends {
            self.0.insert("Depends", depends.to_string().as_str());
        } else {
            self.0.remove("Depends");
        }
    }

    pub fn recommends(&self) -> Option<Relations> {
        self.0.get("Recommends").map(|s| s.parse().unwrap())
    }

    pub fn set_recommends(&mut self, recommends: Option<&Relations>) {
        if let Some(recommends) = recommends {
            self.0.insert("Recommends", recommends.to_string().as_str());
        } else {
            self.0.remove("Recommends");
        }
    }

    pub fn suggests(&self) -> Option<Relations> {
        self.0.get("Suggests").map(|s| s.parse().unwrap())
    }

    pub fn set_suggests(&mut self, suggests: Option<&Relations>) {
        if let Some(suggests) = suggests {
            self.0.insert("Suggests", suggests.to_string().as_str());
        } else {
            self.0.remove("Suggests");
        }
    }

    pub fn enhances(&self) -> Option<Relations> {
        self.0.get("Enhances").map(|s| s.parse().unwrap())
    }

    pub fn set_enhances(&mut self, enhances: Option<&Relations>) {
        if let Some(enhances) = enhances {
            self.0.insert("Enhances", enhances.to_string().as_str());
        } else {
            self.0.remove("Enhances");
        }
    }

    pub fn pre_depends(&self) -> Option<Relations> {
        self.0.get("Pre-Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_pre_depends(&mut self, pre_depends: Option<&Relations>) {
        if let Some(pre_depends) = pre_depends {
            self.0
                .insert("Pre-Depends", pre_depends.to_string().as_str());
        } else {
            self.0.remove("Pre-Depends");
        }
    }

    pub fn breaks(&self) -> Option<Relations> {
        self.0.get("Breaks").map(|s| s.parse().unwrap())
    }

    pub fn set_breaks(&mut self, breaks: Option<&Relations>) {
        if let Some(breaks) = breaks {
            self.0.insert("Breaks", breaks.to_string().as_str());
        } else {
            self.0.remove("Breaks");
        }
    }

    pub fn conflicts(&self) -> Option<Relations> {
        self.0.get("Conflicts").map(|s| s.parse().unwrap())
    }

    pub fn set_conflicts(&mut self, conflicts: Option<&Relations>) {
        if let Some(conflicts) = conflicts {
            self.0.insert("Conflicts", conflicts.to_string().as_str());
        } else {
            self.0.remove("Conflicts");
        }
    }

    pub fn replaces(&self) -> Option<Relations> {
        self.0.get("Replaces").map(|s| s.parse().unwrap())
    }

    pub fn set_replaces(&mut self, replaces: Option<&Relations>) {
        if let Some(replaces) = replaces {
            self.0.insert("Replaces", replaces.to_string().as_str());
        } else {
            self.0.remove("Replaces");
        }
    }

    pub fn provides(&self) -> Option<Relations> {
        self.0.get("Provides").map(|s| s.parse().unwrap())
    }

    pub fn set_provides(&mut self, provides: Option<&Relations>) {
        if let Some(provides) = provides {
            self.0.insert("Provides", provides.to_string().as_str());
        } else {
            self.0.remove("Provides");
        }
    }

    pub fn built_using(&self) -> Option<Relations> {
        self.0.get("Built-Using").map(|s| s.parse().unwrap())
    }

    pub fn set_built_using(&mut self, built_using: Option<&Relations>) {
        if let Some(built_using) = built_using {
            self.0
                .insert("Built-Using", built_using.to_string().as_str());
        } else {
            self.0.remove("Built-Using");
        }
    }

    pub fn multi_arch(&self) -> Option<MultiArch> {
        self.0.get("Multi-Arch").map(|s| s.parse().unwrap())
    }

    pub fn set_multi_arch(&mut self, multi_arch: Option<MultiArch>) {
        if let Some(multi_arch) = multi_arch {
            self.0.insert("Multi-Arch", multi_arch.to_string().as_str());
        } else {
            self.0.remove("Multi-Arch");
        }
    }

    pub fn essential(&self) -> bool {
        self.0.get("Essential").map(|s| s == "yes").unwrap_or(false)
    }

    pub fn set_essential(&mut self, essential: bool) {
        if essential {
            self.0.insert("Essential", "yes");
        } else {
            self.0.remove("Essential");
        }
    }

    /// Binary package description
    pub fn description(&self) -> Option<String> {
        self.0.get("Description")
    }

    pub fn set_description(&mut self, description: Option<&str>) {
        if let Some(description) = description {
            self.0.insert("Description", description);
        } else {
            self.0.remove("Description");
        }
    }

    pub fn homepage(&self) -> Option<url::Url> {
        self.0.get("Homepage").and_then(|s| s.parse().ok())
    }

    pub fn set_homepage(&mut self, url: &url::Url) {
        self.0.insert("Homepage", url.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relations::VersionConstraint;
    #[test]
    fn test_parse() {
        let control: Control = r#"Source: foo
Section: libs
Priority: optional
Build-Depends: bar (>= 1.0.0), baz (>= 1.0.0)
Homepage: https://example.com

"#
        .parse()
        .unwrap();
        let source = control.source().unwrap();

        assert_eq!(source.name(), Some("foo".to_owned()));
        assert_eq!(source.section(), Some("libs".to_owned()));
        assert_eq!(source.priority(), Some(super::Priority::Optional));
        assert_eq!(
            source.homepage(),
            Some("https://example.com".parse().unwrap())
        );
        let bd = source.build_depends().unwrap();
        let entries = bd.entries().collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
        let rel = entries[0].relations().collect::<Vec<_>>().pop().unwrap();
        assert_eq!(rel.name(), "bar");
        assert_eq!(
            rel.version(),
            Some((
                VersionConstraint::GreaterThanEqual,
                "1.0.0".parse().unwrap()
            ))
        );
        let rel = entries[1].relations().collect::<Vec<_>>().pop().unwrap();
        assert_eq!(rel.name(), "baz");
        assert_eq!(
            rel.version(),
            Some((
                VersionConstraint::GreaterThanEqual,
                "1.0.0".parse().unwrap()
            ))
        );
    }

    #[test]
    fn test_description() {
        let control: Control = r#"Source: foo

Package: foo
Description: this is the short description
 And the longer one
 .
 is on the next lines
"#
        .parse()
        .unwrap();
        let binary = control.binaries().next().unwrap();
        assert_eq!(
            binary.description(),
            Some(
                "this is the short description\nAnd the longer one\n.\nis on the next lines"
                    .to_owned()
            )
        );
    }

    #[test]
    fn test_as_mut_deb822() {
        let mut control = Control::new();
        let deb822 = control.as_mut_deb822();
        let mut p = deb822.add_paragraph();
        p.insert("Source", "foo");
        assert_eq!(control.source().unwrap().name(), Some("foo".to_owned()));
    }

    #[test]
    fn test_as_deb822() {
        let control = Control::new();
        let _deb822: &deb822_lossless::Deb822 = control.as_deb822();
    }

    #[test]
    fn test_set_depends() {
        let mut control = Control::new();
        let mut binary = control.add_binary("foo");
        let relations: Relations = "bar (>= 1.0.0)".parse().unwrap();
        binary.set_depends(Some(&relations));
    }

    #[test]
    fn test_wrap_and_sort() {
        let mut control: Control = r#"Package: blah
Section:     libs



Package: foo
Description: this is a 
      bar
      blah
"#.parse().unwrap();
        control.wrap_and_sort(deb822_lossless::Indentation::Spaces(2), false, None);
        let expected = r#"Package: blah
Section: libs

Package: foo
Description: this is a 
  bar
  blah
"#.to_owned();
        assert_eq!(control.to_string(), expected);
    }

    #[test]
    fn test_wrap_and_sort_source() {
        let mut control: Control = r#"Source: blah
Depends: foo, bar   (<=  1.0.0)

"#
        .parse()
        .unwrap();
        control.wrap_and_sort(deb822_lossless::Indentation::Spaces(2), true, None);
        let expected = r#"Source: blah
Depends: bar (<= 1.0.0), foo
"#.to_owned();
        assert_eq!(control.to_string(), expected);
    }
}
