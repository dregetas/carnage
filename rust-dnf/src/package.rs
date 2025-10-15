use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("Invalid package name: {0}")]
    InvalidName(String),
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub name: String,
    pub arch: String,
}

impl PackageName {
    pub fn new(name: &str, arch: &str) -> Result<Self, PackageError> {
        if name.is_empty() {
            return Err(PackageError::InvalidName(name.to_string()));
        }
        Ok(Self {
            name: name.to_string(),
            arch: arch.to_string(),
        })
    }
    
    pub fn from_string(s: &str) -> Result<Self, PackageError> {
        // Parse strings like "package.x86_64" or just "package"
        if let Some((name, arch)) = s.split_once('.') {
            Self::new(name, arch)
        } else {
            Self::new(s, "x86_64") // Default architecture
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub epoch: u32,
    pub version: String,
    pub release: String,
}

impl Version {
    pub fn new(epoch: u32, version: &str, release: &str) -> Result<Self, PackageError> {
        if version.is_empty() {
            return Err(PackageError::InvalidVersion(version.to_string()));
        }
        Ok(Self {
            epoch,
            version: version.to_string(),
            release: release.to_string(),
        })
    }
    
    pub fn parse(s: &str) -> Result<Self, PackageError> {
        // Simple version parser - you'll want to improve this
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() >= 2 {
            Ok(Self {
                epoch: 0,
                version: parts[0].to_string(),
                release: parts[1].to_string(),
            })
        } else {
            Ok(Self {
                epoch: 0,
                version: s.to_string(),
                release: "1".to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub comparator: Option<String>, // ">", ">=", "=", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: PackageName,
    pub version: Version,
    pub description: String,
    pub summary: String,
    pub dependencies: Vec<Dependency>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub files: Vec<String>,
    pub size: u64,
    pub license: String,
    pub url: String,
}

impl Package {
    pub fn new(
        name: PackageName,
        version: Version,
        description: String,
    ) -> Self {
        Self {
            name,
            version,
            description,
            summary: String::new(),
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            files: Vec::new(),
            size: 0,
            license: String::new(),
            url: String::new(),
        }
    }
}