//! Package types and package.json handling

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::core::{VelocityError, VelocityResult};

/// A package with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Package description
    #[serde(default)]
    pub description: String,

    /// Dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    /// Dev dependencies
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,

    /// Peer dependencies
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,

    /// Optional dependencies
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
}

/// package.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    /// Package name
    pub name: String,

    /// Package version
    #[serde(default = "default_version")]
    pub version: String,

    /// Package description
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// Main entry point
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,

    /// Module entry point (ESM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// Types entry point
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,

    /// Package type (commonjs or module)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub package_type: Option<String>,

    /// Scripts
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scripts: HashMap<String, String>,

    /// Dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,

    /// Dev dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty", rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,

    /// Peer dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty", rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,

    /// Optional dependencies
    #[serde(default, skip_serializing_if = "HashMap::is_empty", rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,

    /// Workspace configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<WorkspacesConfig>,

    /// Package manager
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "packageManager")]
    pub package_manager: Option<String>,

    /// Private package flag
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub private: bool,

    /// License
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<serde_json::Value>,

    /// Repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<serde_json::Value>,

    /// Keywords
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// Engines
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub engines: HashMap<String, String>,

    /// Files to include in package
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,

    /// Binary executables
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<serde_json::Value>,

    /// Exports map
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exports: Option<serde_json::Value>,

    /// Other fields (preserved during round-trip)
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Workspace configuration in package.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspacesConfig {
    /// Simple array of glob patterns
    Patterns(Vec<String>),
    /// Object with packages and nohoist
    Object {
        packages: Vec<String>,
        #[serde(default)]
        nohoist: Vec<String>,
    },
}

impl PackageJson {
    /// Load package.json from a path
    pub fn load(path: &Path) -> VelocityResult<Self> {
        let package_json_path = if path.is_dir() {
            path.join("package.json")
        } else {
            path.to_path_buf()
        };

        if !package_json_path.exists() {
            return Err(VelocityError::PackageJsonNotFound(package_json_path));
        }

        let content = std::fs::read_to_string(&package_json_path)?;
        let package_json: PackageJson = serde_json::from_str(&content)?;
        Ok(package_json)
    }

    /// Save package.json to a path
    pub fn save(&self, path: &Path) -> VelocityResult<()> {
        let package_json_path = if path.is_dir() {
            path.join("package.json")
        } else {
            path.to_path_buf()
        };

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(package_json_path, content)?;
        Ok(())
    }

    /// Create a new minimal package.json
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            main: None,
            module: None,
            types: None,
            package_type: None,
            scripts: HashMap::new(),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            workspaces: None,
            package_manager: Some("velocity@0.1.0".to_string()),
            private: false,
            license: Some("MIT".to_string()),
            author: None,
            repository: None,
            keywords: Vec::new(),
            engines: HashMap::new(),
            files: Vec::new(),
            bin: None,
            exports: None,
            other: HashMap::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: &str, version: &str, dev: bool) {
        if dev {
            self.dev_dependencies.insert(name.to_string(), version.to_string());
        } else {
            self.dependencies.insert(name.to_string(), version.to_string());
        }
    }

    /// Remove a dependency
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
            || self.dev_dependencies.remove(name).is_some()
            || self.peer_dependencies.remove(name).is_some()
            || self.optional_dependencies.remove(name).is_some()
    }

    /// Get all dependencies (combined)
    pub fn all_dependencies(&self) -> HashMap<String, String> {
        let mut deps = self.dependencies.clone();
        deps.extend(self.dev_dependencies.clone());
        deps.extend(self.optional_dependencies.clone());
        deps
    }

    /// Get production dependencies only
    pub fn production_dependencies(&self) -> HashMap<String, String> {
        let mut deps = self.dependencies.clone();
        deps.extend(self.optional_dependencies.clone());
        deps
    }

    /// Check if this is a workspace root
    pub fn is_workspace_root(&self) -> bool {
        self.workspaces.is_some()
    }

    /// Get workspace patterns
    pub fn workspace_patterns(&self) -> Vec<String> {
        match &self.workspaces {
            Some(WorkspacesConfig::Patterns(patterns)) => patterns.clone(),
            Some(WorkspacesConfig::Object { packages, .. }) => packages.clone(),
            None => Vec::new(),
        }
    }

    /// Check if package has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
            || !self.dev_dependencies.is_empty()
            || !self.optional_dependencies.is_empty()
    }
}

impl Default for PackageJson {
    fn default() -> Self {
        Self::new("my-project")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_package_json_roundtrip() {
        let dir = tempdir().unwrap();
        
        let mut pkg = PackageJson::new("test-package");
        pkg.add_dependency("react", "^18.0.0", false);
        pkg.add_dependency("typescript", "^5.0.0", true);
        
        pkg.save(dir.path()).unwrap();
        
        let loaded = PackageJson::load(dir.path()).unwrap();
        assert_eq!(loaded.name, "test-package");
        assert_eq!(loaded.dependencies.get("react").unwrap(), "^18.0.0");
        assert_eq!(loaded.dev_dependencies.get("typescript").unwrap(), "^5.0.0");
    }

    #[test]
    fn test_workspace_patterns() {
        let mut pkg = PackageJson::new("monorepo");
        pkg.workspaces = Some(WorkspacesConfig::Patterns(vec![
            "packages/*".to_string(),
            "apps/*".to_string(),
        ]));

        let patterns = pkg.workspace_patterns();
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"packages/*".to_string()));
    }
}
