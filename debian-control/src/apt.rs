use crate::fields::{MultiArch, Priority, Sha1Checksum, Sha256Checksum, Sha512Checksum};
use crate::relations::Relations;

/// A source package in the APT package manager.
pub struct Source(deb822_lossless::Paragraph);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct MD5Checksum {
    pub md5sum: String,
    pub size: usize,
    pub filename: String,
}

impl std::fmt::Display for MD5Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.md5sum, self.size, self.filename)
    }
}

impl std::str::FromStr for MD5Checksum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let md5sum = parts.next().ok_or(())?;
        let size = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let filename = parts.next().ok_or(())?.to_string();
        Ok(Self {
            md5sum: md5sum.to_string(),
            size,
            filename,
        })
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Source {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        use pyo3::prelude::*;
        let d = self.0.to_object(py);

        let m = py.import_bound("debian.deb822").unwrap();
        let cls = m.getattr("Sources").unwrap();

        cls.call1((d,)).unwrap().to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Source {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Source(ob.extract()?))
    }
}

impl Source {
    pub fn new(paragraph: deb822_lossless::Paragraph) -> Self {
        Self(paragraph)
    }

    pub fn package(&self) -> Option<String> {
        self.0.get("Package").map(|s| s.to_string())
    }

    pub fn set_package(&mut self, package: &str) {
        self.0.insert("Package", package);
    }

    pub fn version(&self) -> Option<debversion::Version> {
        self.0.get("Version").map(|s| s.parse().unwrap())
    }

    pub fn set_version(&mut self, version: debversion::Version) {
        self.0.insert("Version", &version.to_string());
    }

    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer").map(|s| s.to_string())
    }

    pub fn set_maintainer(&mut self, maintainer: &str) {
        self.0.insert("Maintainer", maintainer);
    }

    pub fn uploaders(&self) -> Option<Vec<String>> {
        self.0.get("Uploaders").map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn set_uploaders(&mut self, uploaders: Vec<String>) {
        self.0.insert("Uploaders", &uploaders.join(", "));
    }

    pub fn standards_version(&self) -> Option<String> {
        self.0.get("Standards-Version").map(|s| s.to_string())
    }

    pub fn set_standards_version(&mut self, version: &str) {
        self.0.insert("Standards-Version", version);
    }

    pub fn format(&self) -> Option<String> {
        self.0.get("Format").map(|s| s.to_string())
    }

    pub fn set_format(&mut self, format: &str) {
        self.0.insert("Format", format);
    }

    pub fn vcs_browser(&self) -> Option<String> {
        self.0.get("Vcs-Browser").map(|s| s.to_string())
    }

    pub fn set_vcs_browser(&mut self, url: &str) {
        self.0.insert("Vcs-Browser", url);
    }

    pub fn vcs_git(&self) -> Option<String> {
        self.0.get("Vcs-Git").map(|s| s.to_string())
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

    pub fn vcs_hg(&self) -> Option<String> {
        self.0.get("Vcs-Hg").map(|s| s.to_string())
    }

    pub fn set_vcs_hg(&mut self, url: &str) {
        self.0.insert("Vcs-Hg", url);
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

    pub fn build_depends(&self) -> Option<Relations> {
        self.0.get("Build-Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_build_depends(&mut self, relations: Relations) {
        self.0
            .insert("Build-Depends", relations.to_string().as_str());
    }

    pub fn build_depends_indep(&self) -> Option<Relations> {
        self.0
            .get("Build-Depends-Indep")
            .map(|s| s.parse().unwrap())
    }

    pub fn set_build_depends_indep(&mut self, relations: Relations) {
        self.0.insert("Build-Depends-Indep", &relations.to_string());
    }

    pub fn build_depends_arch(&self) -> Option<Relations> {
        self.0.get("Build-Depends-Arch").map(|s| s.parse().unwrap())
    }

    pub fn set_build_depends_arch(&mut self, relations: Relations) {
        self.0.insert("Build-Depends-Arch", &relations.to_string());
    }

    pub fn build_conflicts(&self) -> Option<Relations> {
        self.0.get("Build-Conflicts").map(|s| s.parse().unwrap())
    }

    pub fn set_build_conflicts(&mut self, relations: Relations) {
        self.0.insert("Build-Conflicts", &relations.to_string());
    }

    pub fn build_conflicts_indep(&self) -> Option<Relations> {
        self.0
            .get("Build-Conflicts-Indep")
            .map(|s| s.parse().unwrap())
    }

    pub fn set_build_conflicts_indep(&mut self, relations: Relations) {
        self.0
            .insert("Build-Conflicts-Indep", &relations.to_string());
    }

    pub fn build_conflicts_arch(&self) -> Option<Relations> {
        self.0
            .get("Build-Conflicts-Arch")
            .map(|s| s.parse().unwrap())
    }

    pub fn set_build_conflicts_arch(&mut self, relations: Relations) {
        self.0
            .insert("Build-Conflicts-Arch", &relations.to_string());
    }

    pub fn binary(&self) -> Option<Relations> {
        self.0.get("Binary").map(|s| s.parse().unwrap())
    }

    pub fn set_binary(&mut self, relations: Relations) {
        self.0.insert("Binary", &relations.to_string());
    }

    pub fn homepage(&self) -> Option<String> {
        self.0.get("Homepage").map(|s| s.to_string())
    }

    pub fn set_homepage(&mut self, url: &str) {
        self.0.insert("Homepage", url);
    }

    pub fn section(&self) -> Option<String> {
        self.0.get("Section").map(|s| s.to_string())
    }

    pub fn set_section(&mut self, section: &str) {
        self.0.insert("Section", section);
    }

    pub fn priority(&self) -> Option<Priority> {
        self.0.get("Priority").and_then(|v| v.parse().ok())
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.0.insert("Priority", priority.to_string().as_str());
    }

    /// The architecture of the package.
    pub fn architecture(&self) -> Option<String> {
        self.0.get("Architecture")
    }

    pub fn set_architecture(&mut self, arch: &str) {
        self.0.insert("Architecture", arch);
    }

    pub fn directory(&self) -> Option<String> {
        self.0.get("Directory").map(|s| s.to_string())
    }

    pub fn set_directory(&mut self, dir: &str) {
        self.0.insert("Directory", dir);
    }

    pub fn testsuite(&self) -> Option<String> {
        self.0.get("Testsuite").map(|s| s.to_string())
    }

    pub fn set_testsuite(&mut self, testsuite: &str) {
        self.0.insert("Testsuite", testsuite);
    }

    pub fn files(&self) -> Vec<MD5Checksum> {
        self.0
            .get("Files")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<MD5Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_files(&mut self, files: Vec<MD5Checksum>) {
        self.0.insert(
            "Files",
            &files
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha1(&self) -> Vec<Sha1Checksum> {
        self.0
            .get("Checksums-Sha1")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha1Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha1(&mut self, checksums: Vec<Sha1Checksum>) {
        self.0.insert(
            "Checksums-Sha1",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha256(&self) -> Vec<Sha256Checksum> {
        self.0
            .get("Checksums-Sha256")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha256Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha256(&mut self, checksums: Vec<Sha256Checksum>) {
        self.0.insert(
            "Checksums-Sha256",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha512(&self) -> Vec<Sha512Checksum> {
        self.0
            .get("Checksums-Sha512")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha512Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha512(&mut self, checksums: Vec<Sha512Checksum>) {
        self.0.insert(
            "Checksums-Sha512",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }
}

impl std::str::FromStr for Source {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

pub struct Package(deb822_lossless::Paragraph);

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Package {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        use pyo3::prelude::*;
        let d = self.0.to_object(py);

        let m = py.import_bound("debian.deb822").unwrap();
        let cls = m.getattr("Packages").unwrap();

        cls.call1((d,)).unwrap().to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Package {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Package(ob.extract()?))
    }
}

impl Package {
    pub fn new(paragraph: deb822_lossless::Paragraph) -> Self {
        Self(paragraph)
    }

    pub fn name(&self) -> Option<String> {
        self.0.get("Package").map(|s| s.to_string())
    }

    pub fn set_name(&mut self, name: &str) {
        self.0.insert("Package", name);
    }

    pub fn version(&self) -> Option<debversion::Version> {
        self.0.get("Version").map(|s| s.parse().unwrap())
    }

    pub fn set_version(&mut self, version: debversion::Version) {
        self.0.insert("Version", &version.to_string());
    }

    pub fn installed_size(&self) -> Option<usize> {
        self.0.get("Installed-Size").map(|s| s.parse().unwrap())
    }

    pub fn set_installed_size(&mut self, size: usize) {
        self.0.insert("Installed-Size", &size.to_string());
    }

    pub fn maintainer(&self) -> Option<String> {
        self.0.get("Maintainer").map(|s| s.to_string())
    }

    pub fn set_maintainer(&mut self, maintainer: &str) {
        self.0.insert("Maintainer", maintainer);
    }

    pub fn architecture(&self) -> Option<String> {
        self.0.get("Architecture").map(|s| s.to_string())
    }

    pub fn set_architecture(&mut self, arch: &str) {
        self.0.insert("Architecture", arch);
    }

    pub fn depends(&self) -> Option<Relations> {
        self.0.get("Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_depends(&mut self, relations: Relations) {
        self.0.insert("Depends", &relations.to_string());
    }

    pub fn recommends(&self) -> Option<Relations> {
        self.0.get("Recommends").map(|s| s.parse().unwrap())
    }

    pub fn set_recommends(&mut self, relations: Relations) {
        self.0.insert("Recommends", &relations.to_string());
    }

    pub fn suggests(&self) -> Option<Relations> {
        self.0.get("Suggests").map(|s| s.parse().unwrap())
    }

    pub fn set_suggests(&mut self, relations: Relations) {
        self.0.insert("Suggests", &relations.to_string());
    }

    pub fn enhances(&self) -> Option<Relations> {
        self.0.get("Enhances").map(|s| s.parse().unwrap())
    }

    pub fn set_enhances(&mut self, relations: Relations) {
        self.0.insert("Enhances", &relations.to_string());
    }

    pub fn pre_depends(&self) -> Option<Relations> {
        self.0.get("Pre-Depends").map(|s| s.parse().unwrap())
    }

    pub fn set_pre_depends(&mut self, relations: Relations) {
        self.0.insert("Pre-Depends", &relations.to_string());
    }

    pub fn breaks(&self) -> Option<Relations> {
        self.0.get("Breaks").map(|s| s.parse().unwrap())
    }

    pub fn set_breaks(&mut self, relations: Relations) {
        self.0.insert("Breaks", &relations.to_string());
    }

    pub fn conflicts(&self) -> Option<Relations> {
        self.0.get("Conflicts").map(|s| s.parse().unwrap())
    }

    pub fn set_conflicts(&mut self, relations: Relations) {
        self.0.insert("Conflicts", &relations.to_string());
    }

    pub fn replaces(&self) -> Option<Relations> {
        self.0.get("Replaces").map(|s| s.parse().unwrap())
    }

    pub fn set_replaces(&mut self, relations: Relations) {
        self.0.insert("Replaces", &relations.to_string());
    }

    pub fn provides(&self) -> Option<Relations> {
        self.0.get("Provides").map(|s| s.parse().unwrap())
    }

    pub fn set_provides(&mut self, relations: Relations) {
        self.0.insert("Provides", &relations.to_string());
    }

    pub fn section(&self) -> Option<String> {
        self.0.get("Section").map(|s| s.to_string())
    }

    pub fn set_section(&mut self, section: &str) {
        self.0.insert("Section", section);
    }

    pub fn priority(&self) -> Option<Priority> {
        self.0.get("Priority").and_then(|v| v.parse().ok())
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.0.insert("Priority", priority.to_string().as_str());
    }

    pub fn description(&self) -> Option<String> {
        self.0.get("Description").map(|s| s.to_string())
    }

    pub fn set_description(&mut self, description: &str) {
        self.0.insert("Description", description);
    }

    pub fn homepage(&self) -> Option<url::Url> {
        self.0.get("Homepage").map(|s| s.parse().unwrap())
    }

    pub fn set_homepage(&mut self, url: &url::Url) {
        self.0.insert("Homepage", url.as_ref());
    }

    pub fn source(&self) -> Option<String> {
        self.0.get("Source").map(|s| s.to_string())
    }

    pub fn set_source(&mut self, source: &str) {
        self.0.insert("Source", source);
    }

    pub fn description_md5(&self) -> Option<String> {
        self.0.get("Description-md5").map(|s| s.to_string())
    }

    pub fn set_description_md5(&mut self, md5: &str) {
        self.0.insert("Description-md5", md5);
    }

    pub fn tags(&self, tag: &str) -> Option<Vec<String>> {
        self.0
            .get(tag)
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
    }

    pub fn set_tags(&mut self, tag: &str, tags: Vec<String>) {
        self.0.insert(tag, &tags.join(", "));
    }

    pub fn filename(&self) -> Option<String> {
        self.0.get("Filename").map(|s| s.to_string())
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.0.insert("Filename", filename);
    }

    pub fn size(&self) -> Option<usize> {
        self.0.get("Size").map(|s| s.parse().unwrap())
    }

    pub fn set_size(&mut self, size: usize) {
        self.0.insert("Size", &size.to_string());
    }

    pub fn md5sum(&self) -> Option<String> {
        self.0.get("MD5sum").map(|s| s.to_string())
    }

    pub fn set_md5sum(&mut self, md5sum: &str) {
        self.0.insert("MD5sum", md5sum);
    }

    pub fn sha256(&self) -> Option<String> {
        self.0.get("SHA256").map(|s| s.to_string())
    }

    pub fn set_sha256(&mut self, sha256: &str) {
        self.0.insert("SHA256", sha256);
    }

    pub fn multi_arch(&self) -> Option<MultiArch> {
        self.0.get("Multi-Arch").map(|s| s.parse().unwrap())
    }

    pub fn set_multi_arch(&mut self, arch: MultiArch) {
        self.0.insert("Multi-Arch", arch.to_string().as_str());
    }
}

impl std::str::FromStr for Package {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

pub struct Release(deb822_lossless::Paragraph);

#[cfg(feature = "python-debian")]
impl pyo3::ToPyObject for Release {
    fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
        use pyo3::prelude::*;
        let d = self.0.to_object(py);

        let m = py.import_bound("debian.deb822").unwrap();
        let cls = m.getattr("Release").unwrap();

        cls.call1((d,)).unwrap().to_object(py)
    }
}

#[cfg(feature = "python-debian")]
impl pyo3::FromPyObject<'_> for Release {
    fn extract_bound(ob: &pyo3::Bound<pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use pyo3::prelude::*;
        Ok(Release(ob.extract()?))
    }
}

impl Release{
    pub fn new(paragraph: deb822_lossless::Paragraph) -> Self {
        Self(paragraph)
    }

    pub fn origin(&self) -> Option<String> {
        self.0.get("Origin").map(|s| s.to_string())
    }

    pub fn set_origin(&mut self, origin: &str) {
        self.0.insert("Origin", origin);
    }

    pub fn label(&self) -> Option<String> {
        self.0.get("Label").map(|s| s.to_string())
    }

    pub fn set_label(&mut self, label: &str) {
        self.0.insert("Label", label);
    }

    pub fn suite(&self) -> Option<String> {
        self.0.get("Suite").map(|s| s.to_string())
    }

    pub fn set_suite(&mut self, suite: &str) {
        self.0.insert("Suite", suite);
    }

    pub fn codename(&self) -> Option<String> {
        self.0.get("Codename").map(|s| s.to_string())
    }

    pub fn set_codename(&mut self, codename: &str) {
        self.0.insert("Codename", codename);
    }

    pub fn changelogs(&self) -> Option<Vec<String>> {
        self.0.get("Changelogs").map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn set_changelogs(&mut self, changelogs: Vec<String>) {
        self.0.insert("Changelogs", &changelogs.join(", "));
    }

    #[cfg(feature = "chrono")]
    pub fn date(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        self.0.get("Date").as_ref().map(|s| chrono::DateTime::parse_from_rfc2822(s).unwrap())
    }

    #[cfg(feature = "chrono")]
    pub fn set_date(&mut self, date: chrono::DateTime<chrono::FixedOffset>) {
        self.0.insert("Date", date.to_rfc2822().as_str());
    }

    #[cfg(feature = "chrono")]
    pub fn valid_until(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        self.0.get("Valid-Until").as_ref().map(|s| chrono::DateTime::parse_from_rfc2822(s).unwrap())
    }

    #[cfg(feature = "chrono")]
    pub fn set_valid_until(&mut self, date: chrono::DateTime<chrono::FixedOffset>) {
        self.0.insert("Valid-Until", date.to_rfc2822().as_str());
    }

    pub fn acquire_by_hash(&self) -> bool {
        self.0.get("Acquire-By-Hash").map(|s| s == "yes").unwrap_or(false)
    }

    pub fn set_acquire_by_hash(&mut self, acquire_by_hash: bool) {
        self.0.insert("Acquire-By-Hash", if acquire_by_hash { "yes" } else { "no" });
    }

    pub fn no_support_for_architecture_all(&self) -> bool {
        self.0.get("No-Support-For-Architecture-All").map(|s| s == "yes").unwrap_or(false)
    }

    pub fn set_no_support_for_architecture_all(&mut self, no_support_for_architecture_all: bool) {
        self.0.insert("No-Support-For-Architecture-All", if no_support_for_architecture_all { "yes" } else { "no" });
    }

    pub fn architectures(&self) -> Option<Vec<String>> {
        self.0.get("Architectures").map(|s| {
            s.split_whitespace()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn set_architectures(&mut self, architectures: Vec<String>) {
        self.0.insert("Architectures", &architectures.join(" "));
    }

    pub fn components(&self) -> Option<Vec<String>> {
        self.0.get("Components").map(|s| {
            s.split_whitespace()
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn set_components(&mut self, components: Vec<String>) {
        self.0.insert("Components", &components.join(" "));
    }

    pub fn description(&self) -> Option<String> {
        self.0.get("Description").map(|s| s.to_string())
    }

    pub fn set_description(&mut self, description: &str) {
        self.0.insert("Description", description);
    }

    pub fn checksums_md5(&self) -> Vec<MD5Checksum> {
        self.0
            .get("MD5Sum")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<MD5Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_md5(&mut self, files: Vec<MD5Checksum>) {
        self.0.insert(
            "MD5Sum",
            &files
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha1(&self) -> Vec<Sha1Checksum> {
        self.0
            .get("SHA1")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha1Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha1(&mut self, checksums: Vec<Sha1Checksum>) {
        self.0.insert(
            "SHA1",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha256(&self) -> Vec<Sha256Checksum> {
        self.0
            .get("SHA256")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha256Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha256(&mut self, checksums: Vec<Sha256Checksum>) {
        self.0.insert(
            "SHA256",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    pub fn checksums_sha512(&self) -> Vec<Sha512Checksum> {
        self.0
            .get("SHA512")
            .map(|s| {
                s.lines()
                    .map(|line| line.parse().unwrap())
                    .collect::<Vec<Sha512Checksum>>()
            })
            .unwrap_or_default()
    }

    pub fn set_checksums_sha512(&mut self, checksums: Vec<Sha512Checksum>) {
        self.0.insert(
            "SHA512",
            &checksums
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }


}

impl std::str::FromStr for Release {
    type Err = deb822_lossless::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::PackageListEntry;

    #[test]
    fn test_parse_package_list() {
        let s = "package1 binary section standard extra1=foo extra2=bar";
        let p: PackageListEntry = s.parse().unwrap();
        assert_eq!(p.package, "package1");
        assert_eq!(p.package_type, "binary");
        assert_eq!(p.section, "section");
        assert_eq!(p.priority, super::Priority::Standard);
        assert_eq!(p.extra.get("extra1"), Some(&"foo".to_string()));
        assert_eq!(p.extra.get("extra2"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_parse_package_list_no_extra() {
        let s = "package1 binary section standard";
        let p: PackageListEntry = s.parse().unwrap();
        assert_eq!(p.package, "package1");
        assert_eq!(p.package_type, "binary");
        assert_eq!(p.section, "section");
        assert_eq!(p.priority, super::Priority::Standard);
        assert!(p.extra.is_empty());
    }

    #[test]
    fn test_files() {
        let s = "md5sum 1234 filename";
        let f: super::MD5Checksum= s.parse().unwrap();
        assert_eq!(f.md5sum, "md5sum");
        assert_eq!(f.size, 1234);
        assert_eq!(f.filename, "filename");
    }

    #[test]
    fn test_sha1_checksum() {
        let s = "sha1 1234 filename";
        let f: super::Sha1Checksum = s.parse().unwrap();
        assert_eq!(f.sha1, "sha1");
        assert_eq!(f.size, 1234);
        assert_eq!(f.filename, "filename");
    }

    #[test]
    fn test_sha256_checksum() {
        let s = "sha256 1234 filename";
        let f: super::Sha256Checksum = s.parse().unwrap();
        assert_eq!(f.sha256, "sha256");
        assert_eq!(f.size, 1234);
        assert_eq!(f.filename, "filename");
    }

    #[test]
    fn test_sha512_checksum() {
        let s = "sha512 1234 filename";
        let f: super::Sha512Checksum = s.parse().unwrap();
        assert_eq!(f.sha512, "sha512");
        assert_eq!(f.size, 1234);
        assert_eq!(f.filename, "filename");
    }

    #[test]
    fn test_source() {
        let s = r#"Package: foo
Version: 1.0
Maintainer: John Doe <john@example.com>
Uploaders: Jane Doe <jane@example.com>
Standards-Version: 3.9.8
Format: 3.0 (quilt)
Vcs-Browser: https://example.com/foo
Vcs-Git: https://example.com/foo.git
Build-Depends: debhelper (>= 9)
Build-Depends-Indep: python
Build-Depends-Arch: gcc
Build-Conflicts: bar
Build-Conflicts-Indep: python
Build-Conflicts-Arch: gcc
Binary: foo, bar
Homepage: https://example.com/foo
Section: devel
Priority: optional
Architecture: any
Directory: pool/main/f/foo
Files:
 25dcf3b4b6b3b3b3b3b3b3b3b3b3b3b3 1234 foo_1.0.tar.gz
Checksums-Sha1:
 b72b5fae3b3b3b3b3b3b3b3b3b3b3b3 1234 foo_1.0.tar.gz
"#;
        let p: super::Source = s.parse().unwrap();
        assert_eq!(p.package(), Some("foo".to_string()));
        assert_eq!(p.version(), Some("1.0".parse().unwrap()));
        assert_eq!(
            p.maintainer(),
            Some("John Doe <john@example.com>".to_string())
        );
        assert_eq!(
            p.uploaders(),
            Some(vec!["Jane Doe <jane@example.com>".to_string()])
        );
        assert_eq!(p.standards_version(), Some("3.9.8".to_string()));
        assert_eq!(p.format(), Some("3.0 (quilt)".to_string()));
        assert_eq!(p.vcs_browser(), Some("https://example.com/foo".to_string()));
        assert_eq!(p.vcs_git(), Some("https://example.com/foo.git".to_string()));
        assert_eq!(
            p.build_depends_indep().map(|x| x.to_string()),
            Some("python".parse().unwrap())
        );
        assert_eq!(p.build_depends(), Some("debhelper (>= 9)".parse().unwrap()));
        assert_eq!(p.build_depends_arch(), Some("gcc".parse().unwrap()));
        assert_eq!(p.build_conflicts(), Some("bar".parse().unwrap()));
        assert_eq!(p.build_conflicts_indep(), Some("python".parse().unwrap()));
        assert_eq!(p.build_conflicts_arch(), Some("gcc".parse().unwrap()));
        assert_eq!(p.binary(), Some("foo, bar".parse().unwrap()));
        assert_eq!(p.homepage(), Some("https://example.com/foo".to_string()));
        assert_eq!(p.section(), Some("devel".to_string()));
        assert_eq!(p.priority(), Some(super::Priority::Optional));
        assert_eq!(p.architecture(), Some("any".to_string()));
        assert_eq!(p.directory(), Some("pool/main/f/foo".to_string()));
        assert_eq!(p.files().len(), 1);
        assert_eq!(
            p.files()[0].md5sum,
            "25dcf3b4b6b3b3b3b3b3b3b3b3b3b3b3".to_string()
        );
        assert_eq!(p.files()[0].size, 1234);
        assert_eq!(p.files()[0].filename, "foo_1.0.tar.gz".to_string());
        assert_eq!(p.checksums_sha1().len(), 1);
        assert_eq!(
            p.checksums_sha1()[0].sha1,
            "b72b5fae3b3b3b3b3b3b3b3b3b3b3b3".to_string()
        );
    }

    #[test]
    fn test_package() {
        let s = r#"Package: foo
Version: 1.0
Source: bar
Maintainer: John Doe <john@example.com>
Architecture: any
Depends: bar
Recommends: baz
Suggests: qux
Enhances: quux
Pre-Depends: quuz
Breaks: corge
Conflicts: grault
Replaces: garply
Provides: waldo
Section: devel
Priority: optional
Description: Foo is a bar
Homepage: https://example.com/foo
Description-md5: 1234
Tags: foo, bar
Filename: pool/main/f/foo/foo_1.0.deb
Size: 1234
Installed-Size: 1234
MD5sum: 1234
SHA256: 1234
Multi-Arch: same
"#;
        let p: super::Package = s.parse().unwrap();
        assert_eq!(p.name(), Some("foo".to_string()));
        assert_eq!(p.version(), Some("1.0".parse().unwrap()));
        assert_eq!(p.source(), Some("bar".to_string()));
        assert_eq!(
            p.maintainer(),
            Some("John Doe <john@example.com>".to_string())
        );
        assert_eq!(p.architecture(), Some("any".to_string()));
        assert_eq!(p.depends(), Some("bar".parse().unwrap()));
        assert_eq!(p.recommends(), Some("baz".parse().unwrap()));
        assert_eq!(p.suggests(), Some("qux".parse().unwrap()));
        assert_eq!(p.enhances(), Some("quux".parse().unwrap()));
        assert_eq!(p.pre_depends(), Some("quuz".parse().unwrap()));
        assert_eq!(p.breaks(), Some("corge".parse().unwrap()));
        assert_eq!(p.conflicts(), Some("grault".parse().unwrap()));
        assert_eq!(p.replaces(), Some("garply".parse().unwrap()));
        assert_eq!(p.provides(), Some("waldo".parse().unwrap()));
        assert_eq!(p.section(), Some("devel".to_string()));
        assert_eq!(p.priority(), Some(super::Priority::Optional));
        assert_eq!(p.description(), Some("Foo is a bar".to_string()));
        assert_eq!(
            p.homepage(),
            Some(url::Url::parse("https://example.com/foo").unwrap())
        );
        assert_eq!(p.description_md5(), Some("1234".to_string()));
        assert_eq!(
            p.tags("Tags"),
            Some(vec!["foo".to_string(), "bar".to_string()])
        );
        assert_eq!(
            p.filename(),
            Some("pool/main/f/foo/foo_1.0.deb".to_string())
        );
        assert_eq!(p.size(), Some(1234));
        assert_eq!(p.installed_size(), Some(1234));
        assert_eq!(p.md5sum(), Some("1234".to_string()));
        assert_eq!(p.sha256(), Some("1234".to_string()));
        assert_eq!(p.multi_arch(), Some(MultiArch::Same));
    }

    #[test]
    fn test_release() {
        let s = include_str!("testdata/Release");
        let release: super::Release = s.parse().unwrap();

        assert_eq!(release.origin(), Some("Debian".to_string()));
        assert_eq!(release.label(), Some("Debian".to_string()));
        assert_eq!(release.suite(), Some("testing".to_string()));
        assert_eq!(release.architectures(), vec!["all".to_string(), "amd64".to_string(), "arm64".to_string(), "armel".to_string(), "armhf".to_string()].into());
        assert_eq!(release.components(), vec!["main".to_string(), "contrib".to_string(), "non-free-firmware".to_string(), "non-free".to_string()].into());
        assert_eq!(release.description(), Some("Debian x.y Testing distribution - Not Released".to_string()));
        assert_eq!(318, release.checksums_md5().len());
    }
}
