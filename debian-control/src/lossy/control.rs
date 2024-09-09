use deb822_lossless::{FromDeb822, ToDeb822};
use deb822_lossless::{FromDeb822Paragraph, ToDeb822Paragraph};
use crate::fields::Priority;
use crate::lossy::relations::Relations;

fn deserialize_yesno(s: &str) -> Result<bool, String> {
    match s {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(format!("invalid value for yesno: {}", s))
    }
}

fn serialize_yesno(b: &bool) -> String {
    if *b {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

#[derive(FromDeb822, ToDeb822, Default)]
pub struct Source {
    #[deb822(field = "Source")]
    pub name: String,
    #[deb822(field = "Build-Depends")]
    pub build_depends: Option<Relations>,
    #[deb822(field = "Build-Depends-Indep")]
    pub build_depends_indep: Option<Relations>,
    #[deb822(field = "Build-Depends-Arch")]
    pub build_depends_arch: Option<Relations>,
    #[deb822(field = "Build-Conflicts")]
    pub build_conflicts: Option<Relations>,
    #[deb822(field = "Build-Conflicts-Indep")]
    pub build_conflicts_indep: Option<Relations>,
    #[deb822(field = "Build-Conflicts-Arch")]
    pub build_conflicts_arch: Option<Relations>,
    #[deb822(field = "Standards-Version")]
    pub standards_version: Option<String>,
    #[deb822(field = "Homepage")]
    pub homepage: Option<url::Url>,
    #[deb822(field = "Section")]
    pub section: Option<String>,
    #[deb822(field = "Priority")]
    pub priority: Option<Priority>,
    #[deb822(field = "Maintainer")]
    pub maintainer: Option<String>,
    #[deb822(field = "Uploaders")]
    pub uploaders: Option<String>,
    #[deb822(field = "Architecture")]
    pub architecture: Option<String>,
    #[deb822(field = "Rules-Requires-Root", deserialize_with = deserialize_yesno, serialize_with = serialize_yesno)]
    pub rules_requires_root: Option<bool>,
    #[deb822(field = "Testsuite")]
    pub testsuite: Option<String>,
}

#[derive(FromDeb822, ToDeb822, Default)]
pub struct Binary {
    #[deb822(field = "Package")]
    pub name: String,
    #[deb822(field = "Depends")]
    pub depends: Option<Relations>,
    #[deb822(field = "Recommends")]
    pub recommends: Option<Relations>,
    #[deb822(field = "Suggests")]
    pub suggests: Option<Relations>,
    #[deb822(field = "Enhances")]
    pub enhances: Option<Relations>,
    #[deb822(field = "Pre-Depends")]
    pub pre_depends: Option<Relations>,
    #[deb822(field = "Breaks")]
    pub breaks: Option<Relations>,
    #[deb822(field = "Conflicts")]
    pub conflicts: Option<Relations>,
    #[deb822(field = "Replaces")]
    pub replaces: Option<Relations>,
    #[deb822(field = "Provides")]
    pub provides: Option<Relations>,
    #[deb822(field = "Built-Using")]
    pub built_using: Option<Relations>,
    #[deb822(field = "Architecture")]
    pub architecture: Option<String>,
    #[deb822(field = "Section")]
    pub section: Option<String>,
    #[deb822(field = "Priority")]
    pub priority: Option<Priority>,
    #[deb822(field = "Multi-Arch")]
    pub multi_arch: Option<crate::fields::MultiArch>,
    #[deb822(field = "Essential", deserialize_with = deserialize_yesno, serialize_with = serialize_yesno)]
    pub essential: Option<bool>,
    #[deb822(field = "Description")]
    pub description: Option<String>,
}

pub struct Control {
    pub source: Source,
    pub binaries: Vec<Binary>
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.source.to_paragraph())?;
        for binary in &self.binaries {
            f.write_str("\n")?;
            write!(f, "{}", binary.to_paragraph())?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Control {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deb822: deb822_lossless::Deb822 = s.parse()
            .map_err(|e| format!("parse error: {}", e))?;

        let mut source: Option<Source> = None;
        let mut binaries: Vec<Binary> = Vec::new();

        for para in deb822.paragraphs() {
            if para.get("Package").is_some() {
                let binary: Binary = Binary::from_paragraph(&para)?;
                binaries.push(binary);
            } else if para.get("Source").is_some() {
                if source.is_some() {
                    return Err("more than one source paragraph".to_string());
                }
                source = Some(Source::from_paragraph(&para)?);
            } else {
                return Err("paragraph without Source or Package field".to_string());
            }
        }

        Ok(Control {
            source: source.ok_or_else(||"no source paragraph".to_string())?,
            binaries
        })
    }
}

impl Default for Control {
    fn default() -> Self {
        Self::new()
    }
}

impl Control {
    pub fn new() -> Self {
        Self {
            source: Source::default(),
            binaries: Vec::new()
        }
    }

    pub fn add_binary(&mut self, name: &str) -> &mut Binary {
        let binary = Binary {
            name: name.to_string(),
            ..Default::default()
        };
        self.binaries.push(binary);
        self.binaries.last_mut().unwrap()
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
        let source = &control.source;

        assert_eq!(source.name, "foo".to_owned());
        assert_eq!(source.section, Some("libs".to_owned()));
        assert_eq!(source.priority, Some(super::Priority::Optional));
        assert_eq!(
            source.homepage,
            Some("https://example.com".parse().unwrap())
        );
        let bd = source.build_depends.as_ref().unwrap();
        let mut entries = bd.iter().collect::<Vec<_>>();
        assert_eq!(entries.len(), 2);
        let rel = entries[0].pop().unwrap();
        assert_eq!(rel.name, "bar");
        assert_eq!(
            rel.version,
            Some((
                VersionConstraint::GreaterThanEqual,
                "1.0.0".parse().unwrap()
            ))
        );
        let rel = entries[1].pop().unwrap();
        assert_eq!(rel.name, "baz");
        assert_eq!(
            rel.version,
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
        let binary = &control.binaries[0];
        assert_eq!(
            binary.description,
            Some("this is the short description\nAnd the longer one\n.\nis on the next lines"
                .to_owned())
        );
    }
}
