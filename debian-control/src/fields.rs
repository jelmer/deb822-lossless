use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Priority {
    Required,
    Important,
    Standard,
    Optional,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "required" => Ok(Priority::Required),
            "important" => Ok(Priority::Important),
            "standard" => Ok(Priority::Standard),
            "optional" => Ok(Priority::Optional),
            "extra" => Ok(Priority::Extra),
            _ => Err(()),
        }
    }
}

pub trait Checksum {
    fn filename(&self) -> String;

    fn size(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha1Checksum {
    pub sha1: String,
    pub size: usize,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha1 = parts.next().ok_or(())?;
        let size = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let filename = parts.next().ok_or(())?.to_string();
        Ok(Self {
            sha1: sha1.to_string(),
            size,
            filename,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha256Checksum {
    pub sha256: String,
    pub size: usize,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha256 = parts.next().ok_or(())?;
        let size = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let filename = parts.next().ok_or(())?.to_string();
        Ok(Self {
            sha256: sha256.to_string(),
            size,
            filename,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Sha512Checksum {
    pub sha512: String,
    pub size: usize,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let sha512 = parts.next().ok_or(())?;
        let size = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let filename = parts.next().ok_or(())?.to_string();
        Ok(Self {
            sha512: sha512.to_string(),
            size,
            filename,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageListEntry {
    pub package: String,
    pub package_type: String,
    pub section: String,
    pub priority: Priority,
    pub extra: std::collections::HashMap<String, String>,
}

impl PackageListEntry {
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let package = parts.next().ok_or(())?.to_string();
        let package_type = parts.next().ok_or(())?.to_string();
        let section = parts.next().ok_or(())?.to_string();
        let priority = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        let mut extra = std::collections::HashMap::new();
        for part in parts {
            let mut kv = part.split('=');
            let k = kv.next().ok_or(())?.to_string();
            let v = kv.next().ok_or(())?.to_string();
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum Urgency {
    #[default]
    Low,
    Medium,
    High,
    Emergency,
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


#[derive(PartialEq, Eq, Debug)]
pub enum MultiArch {
    Same,
    Foreign,
    No,
    Allowed,
}

impl std::str::FromStr for MultiArch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "same" => Ok(MultiArch::Same),
            "foreign" => Ok(MultiArch::Foreign),
            "no" => Ok(MultiArch::No),
            "allowed" => Ok(MultiArch::Allowed),
            _ => Err(()),
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


