use regex::Regex;
use std::borrow::Cow;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVcs {
    pub repo_url: String,
    pub branch: Option<String>,
    pub subpath: Option<String>,
}

impl FromStr for ParsedVcs {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s: Cow<str> = s.trim().into();
        let mut subpath: Option<String> = None;
        let branch: Option<String>;
        let repo_url: String;
        let re = Regex::new(r" \[([^] ]+)\]").unwrap();

        if let Some(ref m) = re.find(s.as_ref()) {
            subpath = Some(m.as_str()[2..m.as_str().len() - 1].to_string());
            s = Cow::Owned([s[..m.start()].to_string(), s[m.end()..].to_string()].concat());
        }

        if let Some(index) = s.find(" -b ") {
            let (url, branch_str) = s.split_at(index);
            branch = Some(branch_str[4..].to_string());
            repo_url = url.to_string();
        } else {
            branch = None;
            repo_url = s.to_string();
        }

        Ok(ParsedVcs {
            repo_url,
            branch,
            subpath,
        })
    }
}

impl std::fmt::Display for ParsedVcs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.repo_url)?;

        if let Some(branch) = &self.branch {
            write!(f, " -b {}", branch)?;
        }

        if let Some(subpath) = &self.subpath {
            write!(f, " [{}]", subpath)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Vcs {
    Git {
        repo_url: String,
        branch: Option<String>,
        subpath: Option<String>,
    },
    Bzr {
        repo_url: String,
        subpath: Option<String>,
    },
    Hg {
        repo_url: String,
    },
    Svn {
        url: String,
    },
    Cvs {
        root: String,
        module: Option<String>,
    },
}

impl Vcs {
    pub fn from_field(name: &str, value: &str) -> Result<Vcs, String> {
        match name {
            "Git" => {
                let parsed_vcs: ParsedVcs =
                    value.parse::<ParsedVcs>().map_err(|e| e.to_string())?;
                Ok(Vcs::Git {
                    repo_url: parsed_vcs.repo_url,
                    branch: parsed_vcs.branch,
                    subpath: parsed_vcs.subpath,
                })
            }
            "Bzr" => {
                let parsed_vcs: ParsedVcs =
                    value.parse::<ParsedVcs>().map_err(|e| e.to_string())?;
                if parsed_vcs.branch.is_some() {
                    return Err("Invalid branch value for Vcs-Bzr".to_string());
                }
                Ok(Vcs::Bzr {
                    repo_url: parsed_vcs.repo_url,
                    subpath: parsed_vcs.subpath,
                })
            }
            "Hg" => Ok(Vcs::Hg {
                repo_url: value.to_string(),
            }),
            "Svn" => Ok(Vcs::Svn {
                url: value.to_string(),
            }),
            "Cvs" => {
                if let Some((root, module)) = value.split_once(' ') {
                    Ok(Vcs::Cvs {
                        root: root.to_string(),
                        module: Some(module.to_string()),
                    })
                } else {
                    Ok(Vcs::Cvs {
                        root: value.to_string(),
                        module: None,
                    })
                }
            }
            n => Err(format!("Unknown VCS: {}", n)),
        }
    }

    pub fn to_field(&self) -> (&str, String) {
        match self {
            Vcs::Git {
                repo_url,
                branch,
                subpath,
            } => (
                "Git",
                ParsedVcs {
                    repo_url: repo_url.to_string(),
                    branch: branch.clone(),
                    subpath: subpath.clone(),
                }
                .to_string(),
            ),
            Vcs::Bzr { repo_url, subpath } => (
                "Bzr",
                if let Some(subpath) = subpath {
                    format!("{} [{}]", repo_url, subpath)
                } else {
                    repo_url.to_string()
                },
            ),
            Vcs::Hg { repo_url } => ("Hg", repo_url.to_string()),
            Vcs::Svn { url } => ("Svn", url.to_string()),
            Vcs::Cvs { root, module } => ("Cvs", {
                if let Some(module) = module {
                    format!("{} {}", root, module)
                } else {
                    root.to_string()
                }
            }),
        }
    }

    pub fn subpath(&self) -> Option<String> {
        match self {
            Vcs::Git { subpath, .. } => subpath.clone(),
            Vcs::Bzr { subpath, .. } => subpath.clone(),
            _ => None,
        }
    }

    pub fn to_branch_url(&self) -> Option<String> {
        match self {
            Vcs::Git {
                repo_url,
                branch,
                subpath: _,
                // TODO: Proper URL encoding
            } => Some(format!("{},branch={}", repo_url, branch.as_ref().unwrap())),
            Vcs::Bzr {
                repo_url,
                subpath: _,
            } => Some(repo_url.clone()),
            Vcs::Hg { repo_url } => Some(repo_url.clone()),
            Vcs::Svn { url } => Some(url.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vcs_info() {
        let vcs_info = ParsedVcs::from_str("https://github.com/jelmer/example").unwrap();
        assert_eq!(vcs_info.repo_url, "https://github.com/jelmer/example");
        assert_eq!(vcs_info.branch, None);
        assert_eq!(vcs_info.subpath, None);
    }

    #[test]
    fn test_vcs_info_with_branch() {
        let vcs_info = ParsedVcs::from_str("https://github.com/jelmer/example -b branch").unwrap();
        assert_eq!(vcs_info.repo_url, "https://github.com/jelmer/example");
        assert_eq!(vcs_info.branch, Some("branch".to_string()));
        assert_eq!(vcs_info.subpath, None);
    }

    #[test]
    fn test_vcs_info_with_subpath() {
        let vcs_info = ParsedVcs::from_str("https://github.com/jelmer/example [subpath]").unwrap();
        assert_eq!(vcs_info.repo_url, "https://github.com/jelmer/example");
        assert_eq!(vcs_info.branch, None);
        assert_eq!(vcs_info.subpath, Some("subpath".to_string()));
    }

    #[test]
    fn test_vcs_info_with_branch_and_subpath() {
        let vcs_info =
            ParsedVcs::from_str("https://github.com/jelmer/example -b branch [subpath]").unwrap();
        assert_eq!(vcs_info.repo_url, "https://github.com/jelmer/example");
        assert_eq!(vcs_info.branch, Some("branch".to_string()));
        assert_eq!(vcs_info.subpath, Some("subpath".to_string()));
    }

    #[test]
    fn test_eq() {
        let vcs_info1 =
            ParsedVcs::from_str("https://github.com/jelmer/example -b branch [subpath]").unwrap();
        let vcs_info2 =
            ParsedVcs::from_str("https://github.com/jelmer/example -b branch [subpath]").unwrap();
        let vcs_info3 =
            ParsedVcs::from_str("https://example.com/jelmer/example -b branch [subpath]").unwrap();

        assert_eq!(vcs_info1, vcs_info2);
        assert_ne!(vcs_info1, vcs_info3);
    }
}
