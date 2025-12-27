//! Error types for Velocity

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for Velocity operations
pub type VelocityResult<T> = Result<T, VelocityError>;

/// Main error type for Velocity
#[derive(Error, Debug)]
pub enum VelocityError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Version not found: {package}@{version}")]
    VersionNotFound { package: String, version: String },

    #[error("Invalid version constraint: {0}")]
    InvalidVersionConstraint(String),

    #[error("Version conflict: {package} requires {required} but {found} was resolved")]
    VersionConflict {
        package: String,
        required: String,
        found: String,
    },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Integrity check failed for {package}: expected {expected}, got {actual}")]
    IntegrityCheckFailed {
        package: String,
        expected: String,
        actual: String,
    },

    #[error("Path traversal attack detected in package {package}: {path}")]
    PathTraversal { package: String, path: String },

    #[error("Permission denied: {permission} for package {package}")]
    PermissionDenied { package: String, permission: String },

    #[error("Script execution failed: {script} in {package}")]
    ScriptFailed { package: String, script: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Lockfile corrupted or invalid")]
    InvalidLockfile,

    #[error("Project not initialized. Run 'velocity init' first.")]
    NotInitialized,

    #[error("Package.json not found at {0}")]
    PackageJsonNotFound(PathBuf),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: operation took too long")]
    Timeout,

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("User input error: {0}")]
    Dialoguer(String),

    #[error("{0}")]
    Other(String),
}

impl From<dialoguer::Error> for VelocityError {
    fn from(err: dialoguer::Error) -> Self {
        VelocityError::Dialoguer(err.to_string())
    }
}

impl VelocityError {
    /// Create a generic error from a string
    pub fn other<S: Into<String>>(msg: S) -> Self {
        VelocityError::Other(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        VelocityError::Config(msg.into())
    }

    /// Create a registry error
    pub fn registry<S: Into<String>>(msg: S) -> Self {
        VelocityError::Registry(msg.into())
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(msg: S) -> Self {
        VelocityError::Cache(msg.into())
    }

    /// Create a template error
    pub fn template<S: Into<String>>(msg: S) -> Self {
        VelocityError::Template(msg.into())
    }

    /// Create a workspace error
    pub fn workspace<S: Into<String>>(msg: S) -> Self {
        VelocityError::Workspace(msg.into())
    }

    /// Create a migration error
    pub fn migration<S: Into<String>>(msg: S) -> Self {
        VelocityError::Migration(msg.into())
    }

    /// Get exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            VelocityError::PackageNotFound(_) => 2,
            VelocityError::VersionNotFound { .. } => 2,
            VelocityError::IntegrityCheckFailed { .. } => 3,
            VelocityError::PermissionDenied { .. } => 4,
            VelocityError::UserCancelled => 130,
            VelocityError::NotInitialized => 5,
            _ => 1,
        }
    }
}
