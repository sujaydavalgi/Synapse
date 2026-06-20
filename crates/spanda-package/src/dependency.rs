//! dependency support for Spanda.
//!
use crate::error::{PackageError, PackageResult};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parsed dependency specification from `[dependencies]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Registry version constraint, e.g. `"0.1.0"` or `">=0.1.0, <1.0.0"`.
    Version(String),

    /// Inline table: `{ version = "0.1.0" }`, `{ path = "../lib" }`, or `{ git = "..." }`.
    Detail(DependencyDetail),
}

/// Detailed dependency source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyDetail {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub rev: Option<String>,
}

impl DependencySpec {
    pub fn parse_version_req(&self) -> PackageResult<Option<VersionReq>> {
        // Parse version req.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // PackageResult<Option<VersionReq>>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.parse_version_req();

        // Dispatch based on the enum variant or current state.
        match self {
            Self::Version(v) => Ok(Some(parse_version_req(v)?)),
            Self::Detail(d) => {
                // Emit output when version provides a v.
                if let Some(v) = &d.version {
                    Ok(Some(parse_version_req(v)?))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn source_kind(&self) -> DependencySourceKind {
        // Source kind.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // DependencySourceKind.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.source_kind();

        // Dispatch based on the enum variant or current state.
        match self {
            Self::Version(_) => DependencySourceKind::Registry,
            Self::Detail(d) => {
                // Proceed only when is some is available.
                if d.path.is_some() {
                    DependencySourceKind::Local
                } else if d.git.is_some() {
                    DependencySourceKind::Git
                } else {
                    DependencySourceKind::Registry
                }
            }
        }
    }

    pub fn local_path(&self, project_root: &std::path::Path) -> Option<PathBuf> {
        // Local path.
        //
        // Parameters:
        // - `self` — method receiver
        // - `project_root` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.local_path(project_root);

        // Dispatch based on the enum variant or current state.
        match self {
            Self::Detail(d) if d.path.is_some() => {
                let p = d.path.as_ref().unwrap();

                // Take this path when p.is absolute().
                if p.is_absolute() {
                    Some(p.clone())
                } else {
                    Some(project_root.join(p))
                }
            }
            _ => None,
        }
    }

    pub fn git_url(&self) -> Option<&str> {
        // Git url.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.git_url();

        // Dispatch based on the enum variant or current state.
        match self {
            Self::Detail(d) => d.git.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencySourceKind {
    Registry,
    Local,
    Git,
}

/// Resolved dependency entry stored in the lockfile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedDependency {
    pub name: String,
    pub version: String,
    pub source: LockedSource,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LockedSource {
    Registry {
        registry: String,
    },
    Local {
        path: PathBuf,
    },
    Git {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
    },
}

pub fn parse_version_req(spec: &str) -> PackageResult<VersionReq> {
    // Parse version req.
    //
    // Parameters:
    // - `spec` — input value
    //
    // Returns:
    // PackageResult<VersionReq>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::dependency::parse_version_req(spec);

    // Produce from) as the result.
    VersionReq::parse(spec).map_err(PackageError::from)
}

pub fn parse_version(spec: &str) -> PackageResult<Version> {
    // Parse version.
    //
    // Parameters:
    // - `spec` — input value
    //
    // Returns:
    // PackageResult<Version>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::dependency::parse_version(spec);

    // Produce from) as the result.
    Version::parse(spec).map_err(PackageError::from)
}

pub fn version_satisfies(version: &Version, req: &VersionReq) -> bool {
    // Version satisfies.
    //
    // Parameters:
    // - `version` — input value
    // - `req` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::dependency::version_satisfies(version, req);

    // Produce matches as the result.
    req.matches(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_constraint() {
        // Parses version constraint.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::dependency::parses_version_constraint();

        let req = parse_version_req("^0.1.0").unwrap();
        assert!(version_satisfies(&Version::new(0, 1, 5), &req));
        assert!(!version_satisfies(&Version::new(0, 2, 0), &req));
    }

    #[test]
    fn detects_local_dependency() {
        // Detects local dependency.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::dependency::detects_local_dependency();

        let spec = DependencySpec::Detail(DependencyDetail {
            version: None,
            path: Some(PathBuf::from("../lib")),
            git: None,
            branch: None,
            tag: None,
            rev: None,
        });
        assert_eq!(spec.source_kind(), DependencySourceKind::Local);
    }

    #[test]
    fn detects_git_dependency() {
        // Detects git dependency.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::dependency::detects_git_dependency();

        let spec = DependencySpec::Detail(DependencyDetail {
            version: None,
            path: None,
            git: Some("https://github.com/spanda/spanda-ros2".into()),
            branch: Some("main".into()),
            tag: None,
            rev: None,
        });
        assert_eq!(spec.source_kind(), DependencySourceKind::Git);
        assert_eq!(
            spec.git_url(),
            Some("https://github.com/spanda/spanda-ros2")
        );
    }
}
