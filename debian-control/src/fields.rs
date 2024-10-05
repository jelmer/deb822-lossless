//! Fields for the control file
use std::str::FromStr;

/// Priority of a package
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Priority {
    /// Required
    Required,

    /// Important
    Important,

    /// Standard
    Standard,

    /// Optional
    Optional,

    /// Extra
    Extra,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Priority::Required => "required",
            Priority::Important => "important",
            Priority::Standard => "standard",
            Priority::Optional => "optional",
            Priority::Extra => "extra",
        })
    }
}

impl std::str::FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "required" => Ok(Priority::Required),
            "important" => Ok(Priority::Important),
            "standard" => Ok(Priority::Standard),
            "optional" => Ok(Priority::Optional),
            "extra" => Ok(Priority::Extra),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

/// A checksum of a file
pub trait Checksum {
    /// Filename
    fn filename(&self) -> String;

    /// Size of the file, in bytes
    fn size(&self) -> usize;
}

/// SHA1 checksum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha1Checksum {
    /// SHA1 checksum
    pub sha1: String,

    /// Size of the file, in bytes
    pub size: usize,

    /// Filename
    pub filename: String,
}

impl Checksum for Sha1Checksum {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for Sha1Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.sha1, self.size, self.filename)
    }
}

impl std::str::FromStr for Sha1Checksum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha1 = parts.next().ok_or_else(|| "Missing sha1".to_string())?;
        let size = parts
            .next()
            .ok_or_else(|| "Missing size".to_string())?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let filename = parts
            .next()
            .ok_or_else(|| "Missing filename".to_string())?
            .to_string();
        Ok(Self {
            sha1: sha1.to_string(),
            size,
            filename,
        })
    }
}

/// SHA-256 checksum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha256Checksum {
    /// SHA-256 checksum
    pub sha256: String,

    /// Size of the file, in bytes
    pub size: usize,

    /// Filename
    pub filename: String,
}

impl Checksum for Sha256Checksum {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for Sha256Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.sha256, self.size, self.filename)
    }
}

impl std::str::FromStr for Sha256Checksum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha256 = parts.next().ok_or_else(|| "Missing sha256".to_string())?;
        let size = parts
            .next()
            .ok_or_else(|| "Missing size".to_string())?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let filename = parts
            .next()
            .ok_or_else(|| "Missing filename".to_string())?
            .to_string();
        Ok(Self {
            sha256: sha256.to_string(),
            size,
            filename,
        })
    }
}

/// SHA-512 checksum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha512Checksum {
    /// SHA-512 checksum
    pub sha512: String,

    /// Size of the file, in bytes
    pub size: usize,

    /// Filename
    pub filename: String,
}

impl Checksum for Sha512Checksum {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for Sha512Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.sha512, self.size, self.filename)
    }
}

impl std::str::FromStr for Sha512Checksum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha512 = parts.next().ok_or_else(|| "Missing sha512".to_string())?;
        let size = parts
            .next()
            .ok_or_else(|| "Missing size".to_string())?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let filename = parts
            .next()
            .ok_or_else(|| "Missing filename".to_string())?
            .to_string();
        Ok(Self {
            sha512: sha512.to_string(),
            size,
            filename,
        })
    }
}

/// An MD5 checksum of a file
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Md5Checksum {
    /// The MD5 checksum
    pub md5sum: String,
    /// The size of the file
    pub size: usize,
    /// The filename
    pub filename: String,
}

impl std::fmt::Display for Md5Checksum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.md5sum, self.size, self.filename)
    }
}

impl std::str::FromStr for Md5Checksum {
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

impl Checksum for Md5Checksum {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

/// A package list entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageListEntry {
    /// Package name
    pub package: String,

    /// Package type
    pub package_type: String,

    /// Section
    pub section: String,

    /// Priority
    pub priority: Priority,

    /// Extra fields
    pub extra: std::collections::HashMap<String, String>,
}

impl PackageListEntry {
    /// Create a new package list entry
    pub fn new(package: &str, package_type: &str, section: &str, priority: Priority) -> Self {
        Self {
            package: package.to_string(),
            package_type: package_type.to_string(),
            section: section.to_string(),
            priority,
            extra: std::collections::HashMap::new(),
        }
    }
}

impl std::fmt::Display for PackageListEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.package, self.package_type, self.section, self.priority
        )?;
        for (k, v) in &self.extra {
            write!(f, " {}={}", k, v)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for PackageListEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let package = parts
            .next()
            .ok_or_else(|| "Missing package".to_string())?
            .to_string();
        let package_type = parts
            .next()
            .ok_or_else(|| "Missing package type".to_string())?
            .to_string();
        let section = parts
            .next()
            .ok_or_else(|| "Missing section".to_string())?
            .to_string();
        let priority = parts
            .next()
            .ok_or_else(|| "Missing priority".to_string())?
            .parse()?;
        let mut extra = std::collections::HashMap::new();
        for part in parts {
            let mut kv = part.split('=');
            let k = kv
                .next()
                .ok_or_else(|| "Missing key".to_string())?
                .to_string();
            let v = kv
                .next()
                .ok_or_else(|| "Missing value".to_string())?
                .to_string();
            extra.insert(k, v);
        }
        Ok(Self {
            package,
            package_type,
            section,
            priority,
            extra,
        })
    }
}

/// Urgency of a particular package version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum Urgency {
    /// Low
    #[default]
    Low,
    /// Medium
    Medium,
    /// High
    High,
    /// Emergency
    Emergency,
    /// Critical
    Critical,
}

impl std::fmt::Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Urgency::Low => f.write_str("low"),
            Urgency::Medium => f.write_str("medium"),
            Urgency::High => f.write_str("high"),
            Urgency::Emergency => f.write_str("emergency"),
            Urgency::Critical => f.write_str("critical"),
        }
    }
}

impl FromStr for Urgency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Urgency::Low),
            "medium" => Ok(Urgency::Medium),
            "high" => Ok(Urgency::High),
            "emergency" => Ok(Urgency::Emergency),
            "critical" => Ok(Urgency::Critical),
            _ => Err(format!("invalid urgency: {}", s)),
        }
    }
}

/// Multi-arch policy
#[derive(PartialEq, Eq, Debug, Default)]
pub enum MultiArch {
    /// Indicates that the package is identical across all architectures. The package can satisfy dependencies for other architectures.
    Same,
    /// The package can be installed alongside the same package of other architectures. It doesn't provide files that conflict with other architectures.
    Foreign,
    /// The package is only for its native architecture and cannot satisfy dependencies for other architectures.
    #[default]
    No,
    /// Similar to "foreign", but the package manager may choose not to install it for foreign architectures if a native package is available.
    Allowed,
}

impl std::str::FromStr for MultiArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "same" => Ok(MultiArch::Same),
            "foreign" => Ok(MultiArch::Foreign),
            "no" => Ok(MultiArch::No),
            "allowed" => Ok(MultiArch::Allowed),
            _ => Err(format!("Invalid multiarch: {}", s)),
        }
    }
}

impl std::fmt::Display for MultiArch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            MultiArch::Same => "same",
            MultiArch::Foreign => "foreign",
            MultiArch::No => "no",
            MultiArch::Allowed => "allowed",
        })
    }
}
