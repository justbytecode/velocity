//! Configuration handling for Velocity
//!
//! Supports velocity.toml, .velocityrc, and environment variable overrides.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::env;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;

use crate::core::{VelocityError, VelocityResult};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Registry configuration
    pub registry: RegistryConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Network configuration
    pub network: NetworkConfig,

    /// Workspace configuration
    pub workspace: WorkspaceConfig,

    /// Telemetry configuration (opt-in only)
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RegistryConfig {
    /// Primary registry URL
    pub url: String,

    /// Scoped registry overrides
    #[serde(default)]
    pub scopes: HashMap<String, String>,

    /// Authentication tokens
    #[serde(default)]
    pub auth_tokens: HashMap<String, String>,

    /// Mirror registries for fallback
    #[serde(default)]
    pub mirrors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Global cache directory
    pub dir: Option<PathBuf>,

    /// Maximum cache size in bytes (0 = unlimited)
    pub max_size: u64,

    /// Cache TTL in seconds for metadata
    pub metadata_ttl: u64,

    /// Enable offline mode
    pub offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Require integrity verification
    pub require_integrity: bool,

    /// Allow scripts to run
    pub allow_scripts: bool,

    /// Trusted scopes (no permission prompts)
    #[serde(default)]
    pub trusted_scopes: Vec<String>,

    /// Trusted packages (no permission prompts)
    #[serde(default)]
    pub trusted_packages: Vec<String>,

    /// Enable dependency confusion protection
    pub dependency_confusion_protection: bool,

    /// Audit on install
    pub audit_on_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Connection timeout in seconds
    pub timeout: u64,

    /// Maximum concurrent downloads
    pub concurrency: usize,

    /// Retry attempts for failed downloads
    pub retries: u32,

    /// Proxy URL
    pub proxy: Option<String>,

    /// Skip SSL verification (dangerous!)
    pub insecure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
    /// Workspace packages glob patterns
    #[serde(default)]
    pub packages: Vec<String>,

    /// Hoist dependencies to root
    pub hoist: bool,

    /// Shared lockfile
    pub shared_lockfile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Enable telemetry (opt-in)
    pub enabled: bool,

    /// Anonymous usage statistics only
    pub anonymous: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            network: NetworkConfig::default(),
            workspace: WorkspaceConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "https://registry.npmjs.org".to_string(),
            scopes: HashMap::new(),
            auth_tokens: HashMap::new(),
            mirrors: vec![],
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dir: None,
            max_size: 0, // Unlimited
            metadata_ttl: 300, // 5 minutes
            offline: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_integrity: true,
            allow_scripts: false, // Secure by default
            trusted_scopes: vec![],
            trusted_packages: vec![],
            dependency_confusion_protection: true,
            audit_on_install: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            concurrency: 16,
            retries: 3,
            proxy: None,
            insecure: false,
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            packages: vec!["packages/*".to_string()],
            hoist: true,
            shared_lockfile: true,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            anonymous: true,
        }
    }
}

impl Config {
    /// Load configuration from project directory and merge with defaults
    pub fn load(project_dir: &Path) -> VelocityResult<Self> {
        let mut config = Config::default();

        // Try loading velocity.toml
        let toml_path = project_dir.join("velocity.toml");
        if toml_path.exists() {
            let content = std::fs::read_to_string(&toml_path)?;
            let file_config: Config = toml::from_str(&content)?;
            config = config.merge(file_config);
        }

        // Try loading .velocityrc (JSON format)
        let rc_path = project_dir.join(".velocityrc");
        if rc_path.exists() {
            let content = std::fs::read_to_string(&rc_path)?;
            let file_config: Config = serde_json::from_str(&content)?;
            config = config.merge(file_config);
        }

        // Apply environment variable overrides
        config = config.apply_env_overrides();

        Ok(config)
    }

    /// Merge another config into this one (other takes precedence)
    fn merge(self, other: Config) -> Self {
        Self {
            registry: RegistryConfig {
                url: if other.registry.url != RegistryConfig::default().url {
                    other.registry.url
                } else {
                    self.registry.url
                },
                scopes: {
                    let mut merged = self.registry.scopes;
                    merged.extend(other.registry.scopes);
                    merged
                },
                auth_tokens: {
                    let mut merged = self.registry.auth_tokens;
                    merged.extend(other.registry.auth_tokens);
                    merged
                },
                mirrors: if !other.registry.mirrors.is_empty() {
                    other.registry.mirrors
                } else {
                    self.registry.mirrors
                },
            },
            cache: CacheConfig {
                dir: other.cache.dir.or(self.cache.dir),
                max_size: if other.cache.max_size != 0 {
                    other.cache.max_size
                } else {
                    self.cache.max_size
                },
                metadata_ttl: other.cache.metadata_ttl,
                offline: other.cache.offline || self.cache.offline,
            },
            security: other.security,
            network: other.network,
            workspace: other.workspace,
            telemetry: other.telemetry,
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(mut self) -> Self {
        if let Ok(registry) = env::var("VELOCITY_REGISTRY") {
            self.registry.url = registry;
        }

        if let Ok(cache_dir) = env::var("VELOCITY_CACHE_DIR") {
            self.cache.dir = Some(PathBuf::from(cache_dir));
        }

        if let Ok(offline) = env::var("VELOCITY_OFFLINE") {
            self.cache.offline = offline == "1" || offline.to_lowercase() == "true";
        }

        if let Ok(concurrency) = env::var("VELOCITY_CONCURRENCY") {
            if let Ok(n) = concurrency.parse() {
                self.network.concurrency = n;
            }
        }

        if let Ok(timeout) = env::var("VELOCITY_TIMEOUT") {
            if let Ok(n) = timeout.parse() {
                self.network.timeout = n;
            }
        }

        self
    }

    /// Get the cache directory, creating it if necessary
    pub fn cache_dir(&self) -> VelocityResult<PathBuf> {
        if let Some(ref dir) = self.cache.dir {
            std::fs::create_dir_all(dir)?;
            return Ok(dir.clone());
        }

        let project_dirs = ProjectDirs::from("com", "velocity", "velocity")
            .ok_or_else(|| VelocityError::config("Could not determine cache directory"))?;

        let cache_dir = project_dirs.cache_dir().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    /// Save configuration to velocity.toml
    pub fn save(&self, project_dir: &Path) -> VelocityResult<()> {
        let toml_path = project_dir.join("velocity.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(toml_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.registry.url, "https://registry.npmjs.org");
        assert!(config.security.require_integrity);
        assert!(!config.security.allow_scripts);
    }

    #[test]
    fn test_config_load_empty_dir() {
        let dir = tempdir().unwrap();
        let config = Config::load(dir.path()).unwrap();
        assert_eq!(config.registry.url, "https://registry.npmjs.org");
    }
}
