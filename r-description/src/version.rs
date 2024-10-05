//! R Version strings
use std::cmp::Ordering;

// Struct to represent a version with major, minor, patch, and an optional pre-release tag
#[derive(Debug, PartialEq, Eq, std::hash::Hash, Clone)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: Option<u32>,
    pre_release: Option<String>, // Pre-release version like "alpha", "beta", etc.
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format the version string as "major.minor.patch" or "major.minor.patch-pre_release"
        write!(
            f,
            "{}.{}{}{}",
            self.major,
            self.minor,
            self.patch.map(|p| format!(".{}", p)).unwrap_or_default(),
            self.pre_release
                .as_ref()
                .map(|s| format!("-{}", s))
                .unwrap_or_default()
        )
    }
}

impl Version {
    /// Create a new version
    fn new(major: u32, minor: u32, patch: Option<u32>, pre_release: Option<&str>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: pre_release.map(|s| s.to_string()),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split the version string by '.' and '-' to get major, minor, patch, and pre-release
        let mut parts = s.splitn(2, '-');
        let version = parts
            .next()
            .ok_or(format!("Invalid version string: {}", s))?;
        let pre_release = parts.next();

        let mut parts = version.split('.');
        let major = parts
            .next()
            .ok_or(format!("Invalid version string: {}", s))?
            .parse()
            .map_err(|_| format!("Invalid major version: {}", s))?;
        let minor = parts
            .next()
            .ok_or(format!("Invalid version string: {}", s))?
            .parse()
            .map_err(|_| format!("Invalid minor version: {}", s))?;
        let patch = if let Some(patch) = parts.next() {
            Some(
                patch
                    .parse()
                    .map_err(|_| format!("Invalid patch version: {}", s))?,
            )
        } else {
            None
        };

        Ok(Self::new(major, minor, patch, pre_release))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major, minor, and patch versions in order
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => self.compare_pre_release(other),
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Version {
    fn compare_pre_release(&self, other: &Self) -> Ordering {
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use std::str::FromStr;

    #[test]
    fn test_version_from_str() {
        use std::str::FromStr;

        let version = Version::from_str("1.2.3").unwrap();
        assert_eq!(version, Version::new(1, 2, Some(3), None));

        let version = Version::from_str("1.2.3-alpha").unwrap();
        assert_eq!(version, Version::new(1, 2, Some(3), Some("alpha")));

        let version = Version::from_str("1.2.3-beta").unwrap();
        assert_eq!(version, Version::new(1, 2, Some(3), Some("beta")));
    }

    #[test]
    fn test_version_cmp() {
        use std::cmp::Ordering;

        let v1 = Version::from_str("1.2.3").unwrap();
        let v2 = Version::from_str("1.2.3").unwrap();
        assert_eq!(v1.cmp(&v2), Ordering::Equal);

        let v1 = Version::from_str("1.2.3").unwrap();
        let v2 = Version::from_str("1.2.4").unwrap();
        assert_eq!(v1.cmp(&v2), Ordering::Less);

        let v1 = Version::from_str("1.2.3").unwrap();
        let v2 = Version::from_str("1.2.3-alpha").unwrap();
        assert_eq!(v1.cmp(&v2), Ordering::Greater);

        let v1 = Version::from_str("1.2.3-alpha").unwrap();
        let v2 = Version::from_str("1.2.3-beta").unwrap();
        assert_eq!(v1.cmp(&v2), Ordering::Less);
    }
}
