//! Registry response types

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Package metadata from npm registry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,

    /// Package description
    #[serde(default)]
    pub description: String,

    /// Distribution tags (latest, next, etc.)
    #[serde(default, rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,

    /// All versions metadata
    #[serde(default)]
    pub versions: HashMap<String, VersionMetadata>,

    /// Modification time
    #[serde(default)]
    pub time: HashMap<String, String>,

    /// Repository info
    #[serde(default)]
    pub repository: Option<Repository>,

    /// Author
    #[serde(default)]
    pub author: Option<Person>,

    /// Maintainers
    #[serde(default)]
    pub maintainers: Vec<Person>,

    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// License
    #[serde(default)]
    pub license: Option<String>,
}

/// Version-specific metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionMetadata {
    /// Package name
    pub name: String,

    /// Version string
    pub version: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Main entry point
    #[serde(default)]
    pub main: Option<String>,

    /// Module entry point
    #[serde(default)]
    pub module: Option<String>,

    /// Distribution info
    pub dist: DistInfo,

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

    /// Peer dependencies meta
    #[serde(default, rename = "peerDependenciesMeta")]
    pub peer_dependencies_meta: HashMap<String, PeerDependencyMeta>,

    /// Engines
    #[serde(default)]
    pub engines: HashMap<String, String>,

    /// OS requirements
    #[serde(default)]
    pub os: Vec<String>,

    /// CPU requirements
    #[serde(default)]
    pub cpu: Vec<String>,

    /// Scripts
    #[serde(default)]
    pub scripts: HashMap<String, String>,

    /// Binary executables
    #[serde(default)]
    pub bin: Option<serde_json::Value>,

    /// Deprecated message
    #[serde(default)]
    pub deprecated: Option<String>,

    /// Has install scripts
    #[serde(default, rename = "hasInstallScript")]
    pub has_install_script: Option<bool>,
}

impl VersionMetadata {
    /// Check if this version has install scripts
    pub fn has_install_scripts(&self) -> bool {
        if let Some(has_script) = self.has_install_script {
            return has_script;
        }

        // Check for common lifecycle scripts
        self.scripts.contains_key("preinstall")
            || self.scripts.contains_key("install")
            || self.scripts.contains_key("postinstall")
            || self.scripts.contains_key("prepare")
    }
}

/// Distribution information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DistInfo {
    /// Tarball URL
    pub tarball: String,

    /// SHA-512 integrity hash
    #[serde(default)]
    pub integrity: Option<String>,

    /// SHA-1 hash (legacy)
    #[serde(default)]
    pub shasum: Option<String>,

    /// Number of files
    #[serde(default, rename = "fileCount")]
    pub file_count: Option<u32>,

    /// Unpacked size
    #[serde(default, rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,

    /// npm signature
    #[serde(default, rename = "npm-signature")]
    pub npm_signature: Option<String>,

    /// Signatures
    #[serde(default)]
    pub signatures: Vec<Signature>,
}

/// Package signature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signature {
    pub keyid: String,
    pub sig: String,
}

/// Peer dependency metadata
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PeerDependencyMeta {
    #[serde(default)]
    pub optional: bool,
}

/// Repository info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Repository {
    String(String),
    Object {
        #[serde(rename = "type")]
        repo_type: Option<String>,
        url: String,
        directory: Option<String>,
    },
}

/// Person (author/maintainer)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Person {
    String(String),
    Object {
        name: Option<String>,
        email: Option<String>,
        url: Option<String>,
    },
}
