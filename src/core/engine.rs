//! Core engine coordinating all Velocity operations

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::core::{Config, Lockfile, PackageJson, VelocityError, VelocityResult};
use crate::installer::Installer;
use crate::registry::RegistryClient;
use crate::resolver::Resolver;
use crate::security::SecurityManager;
use crate::workspace::WorkspaceManager;

/// Main engine for Velocity operations
pub struct Engine {
    /// Project root directory
    pub project_dir: PathBuf,

    /// Configuration
    pub config: Config,

    /// Registry client
    pub registry: Arc<RegistryClient>,

    /// Cache manager
    pub cache: Arc<CacheManager>,

    /// Security manager
    pub security: Arc<SecurityManager>,

    /// Workspace manager (if applicable)
    pub workspace: Option<WorkspaceManager>,
}

impl Engine {
    /// Create a new engine for the given project directory
    pub async fn new(project_dir: &Path) -> VelocityResult<Self> {
        let project_dir = project_dir.canonicalize().unwrap_or_else(|_| project_dir.to_path_buf());
        let config = Config::load(&project_dir)?;

        let cache_dir = config.cache_dir()?;
        let cache = Arc::new(CacheManager::new(&cache_dir, &config.cache)?);

        let registry = Arc::new(RegistryClient::new(&config.registry, cache.clone())?);

        let security = Arc::new(SecurityManager::new(&config.security));

        // Check for workspace
        let workspace = if let Ok(pkg) = PackageJson::load(&project_dir) {
            if pkg.is_workspace_root() {
                Some(WorkspaceManager::new(&project_dir, &config.workspace)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            project_dir,
            config,
            registry,
            cache,
            security,
            workspace,
        })
    }

    /// Check if project is initialized
    pub fn is_initialized(&self) -> bool {
        self.project_dir.join("package.json").exists()
    }

    /// Get the package.json for this project
    pub fn package_json(&self) -> VelocityResult<PackageJson> {
        PackageJson::load(&self.project_dir)
    }

    /// Get the lockfile for this project
    pub fn lockfile(&self) -> VelocityResult<Option<Lockfile>> {
        Lockfile::load(&self.project_dir)
    }

    /// Create a dependency resolver
    pub fn resolver(&self) -> Resolver {
        Resolver::new(self.registry.clone(), self.cache.clone())
    }

    /// Create an installer
    pub fn installer(&self) -> Installer {
        Installer::new(
            self.project_dir.clone(),
            self.cache.clone(),
            self.security.clone(),
            self.config.network.concurrency,
        )
    }

    /// Get node_modules path
    pub fn node_modules_path(&self) -> PathBuf {
        self.project_dir.join("node_modules")
    }

    /// Check if node_modules exists
    pub fn has_node_modules(&self) -> bool {
        self.node_modules_path().exists()
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> VelocityResult<PathBuf> {
        self.config.cache_dir()
    }

    /// Check if running in a workspace
    pub fn is_workspace(&self) -> bool {
        self.workspace.is_some()
    }

    /// Get workspace packages
    pub fn workspace_packages(&self) -> VelocityResult<Vec<PathBuf>> {
        if let Some(ref workspace) = self.workspace {
            workspace.find_packages()
        } else {
            Ok(vec![])
        }
    }

    /// Ensure project is initialized
    pub fn ensure_initialized(&self) -> VelocityResult<()> {
        if !self.is_initialized() {
            return Err(VelocityError::NotInitialized);
        }
        Ok(())
    }
}
