//! Lockfile handling for Velocity
//!
//! Provides deterministic, tamper-resistant lockfile format.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use crate::core::{VelocityError, VelocityResult};

/// Lockfile version
pub const LOCKFILE_VERSION: u32 = 1;

/// Lockfile filename
pub const LOCKFILE_NAME: &str = "velocity.lock";

/// Main lockfile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile format version
    pub version: u32,

    /// Integrity hash of the lockfile content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,

    /// Resolved packages
    #[serde(default)]
    pub packages: Vec<LockedPackage>,

    /// Workspace package mappings
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub workspaces: HashMap<String, WorkspacePackage>,
}

/// A locked package with resolved version and integrity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockedPackage {
    /// Package name
    pub name: String,

    /// Resolved version
    pub version: String,

    /// Download URL
    pub resolved: String,

    /// Integrity hash (sha512 or sha256)
    pub integrity: String,

    /// Dependencies (name -> version)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,

    /// Peer dependencies (name -> version)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peer_dependencies: Vec<String>,

    /// Optional dependencies (name -> version)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub optional_dependencies: Vec<String>,

    /// Whether this package has install scripts
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_scripts: bool,

    /// CPU architectures this package supports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpu: Vec<String>,

    /// OS platforms this package supports
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub os: Vec<String>,
}

/// Workspace package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// Relative path to the package
    pub path: String,

    /// Package version
    pub version: String,

    /// Dependencies from this workspace package
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            version: LOCKFILE_VERSION,
            integrity: None,
            packages: Vec::new(),
            workspaces: HashMap::new(),
        }
    }
}

impl Lockfile {
    /// Create a new empty lockfile
    pub fn new() -> Self {
        Self::default()
    }

    /// Load lockfile from a directory
    pub fn load(dir: &Path) -> VelocityResult<Option<Self>> {
        let path = dir.join(LOCKFILE_NAME);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let mut lockfile: Lockfile = toml::from_str(&content)?;

        // Verify integrity if present
        if let Some(ref stored_integrity) = lockfile.integrity {
            let computed = lockfile.compute_integrity();
            if computed != *stored_integrity {
                return Err(VelocityError::InvalidLockfile);
            }
        }

        Ok(Some(lockfile))
    }

    /// Save lockfile to a directory
    pub fn save(&mut self, dir: &Path) -> VelocityResult<()> {
        // Sort packages for deterministic output
        self.packages.sort_by(|a, b| {
            a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version))
        });

        // Compute and set integrity
        self.integrity = None; // Clear before computing
        let integrity = self.compute_integrity();
        self.integrity = Some(integrity);

        let path = dir.join(LOCKFILE_NAME);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;

        Ok(())
    }

    /// Compute integrity hash of lockfile content
    fn compute_integrity(&self) -> String {
        let mut lockfile_copy = self.clone();
        lockfile_copy.integrity = None;

        let content = toml::to_string(&lockfile_copy).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        format!("sha256-{}", hex::encode(hash))
    }

    /// Find a package by name and version
    pub fn find_package(&self, name: &str, version: &str) -> Option<&LockedPackage> {
        self.packages
            .iter()
            .find(|p| p.name == name && p.version == version)
    }

    /// Find all versions of a package
    pub fn find_package_versions(&self, name: &str) -> Vec<&LockedPackage> {
        self.packages.iter().filter(|p| p.name == name).collect()
    }

    /// Add or update a package
    pub fn add_package(&mut self, package: LockedPackage) {
        // Remove existing entry if present
        self.packages.retain(|p| !(p.name == package.name && p.version == package.version));
        self.packages.push(package);
    }

    /// Remove a package
    pub fn remove_package(&mut self, name: &str, version: &str) {
        self.packages.retain(|p| !(p.name == name && p.version == version));
    }

    /// Get all package names
    pub fn package_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.packages.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Check if lockfile is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.workspaces.is_empty()
    }

    /// Merge another lockfile into this one
    pub fn merge(&mut self, other: Lockfile) {
        for package in other.packages {
            if !self.packages.iter().any(|p| p.name == package.name && p.version == package.version) {
                self.packages.push(package);
            }
        }

        for (name, workspace) in other.workspaces {
            self.workspaces.entry(name).or_insert(workspace);
        }
    }

    /// Get packages that have install scripts
    pub fn packages_with_scripts(&self) -> Vec<&LockedPackage> {
        self.packages.iter().filter(|p| p.has_scripts).collect()
    }

    /// Compute diff with another lockfile
    pub fn diff(&self, other: &Lockfile) -> LockfileDiff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        // Find added and changed packages
        for pkg in &other.packages {
            match self.find_package(&pkg.name, &pkg.version) {
                None => {
                    // Check if it's a version change
                    if self.packages.iter().any(|p| p.name == pkg.name) {
                        changed.push(pkg.clone());
                    } else {
                        added.push(pkg.clone());
                    }
                }
                Some(existing) if existing != pkg => {
                    changed.push(pkg.clone());
                }
                _ => {}
            }
        }

        // Find removed packages
        for pkg in &self.packages {
            if !other.packages.iter().any(|p| p.name == pkg.name) {
                removed.push(pkg.clone());
            }
        }

        LockfileDiff {
            added,
            removed,
            changed,
        }
    }
}

/// Diff between two lockfiles
#[derive(Debug, Clone)]
pub struct LockfileDiff {
    pub added: Vec<LockedPackage>,
    pub removed: Vec<LockedPackage>,
    pub changed: Vec<LockedPackage>,
}

impl LockfileDiff {
    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lockfile_roundtrip() {
        let dir = tempdir().unwrap();
        
        let mut lockfile = Lockfile::new();
        lockfile.add_package(LockedPackage {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            resolved: "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz".to_string(),
            integrity: "sha512-abc123".to_string(),
            dependencies: vec!["dep1@1.0.0".to_string()],
            peer_dependencies: vec![],
            optional_dependencies: vec![],
            has_scripts: false,
            cpu: vec![],
            os: vec![],
        });

        lockfile.save(dir.path()).unwrap();
        
        let loaded = Lockfile::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.packages[0].name, "test-package");
    }

    #[test]
    fn test_lockfile_integrity() {
        let dir = tempdir().unwrap();
        
        let mut lockfile = Lockfile::new();
        lockfile.add_package(LockedPackage {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            resolved: "https://example.com/test.tgz".to_string(),
            integrity: "sha512-abc".to_string(),
            dependencies: vec![],
            peer_dependencies: vec![],
            optional_dependencies: vec![],
            has_scripts: false,
            cpu: vec![],
            os: vec![],
        });

        lockfile.save(dir.path()).unwrap();
        
        // Tamper with the lockfile
        let path = dir.path().join(LOCKFILE_NAME);
        let content = std::fs::read_to_string(&path).unwrap();
        let tampered = content.replace("1.0.0", "2.0.0");
        std::fs::write(&path, tampered).unwrap();

        // Should fail integrity check
        let result = Lockfile::load(dir.path());
        assert!(result.is_err());
    }
}
