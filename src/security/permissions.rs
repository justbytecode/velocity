//! Per-package permission management

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::config::SecurityConfig;

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Access to filesystem
    Filesystem,
    /// Access to network
    Network,
    /// Run scripts
    Scripts,
    /// Access to environment variables
    Environment,
    /// Execute child processes
    ChildProcess,
}

/// Permission decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecision {
    /// Permission granted
    Allow,
    /// Permission denied
    Deny,
    /// Ask user at runtime
    Prompt,
}

/// Package permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackagePermissions {
    /// Filesystem access
    pub filesystem: Option<PermissionDecision>,
    /// Network access
    pub network: Option<PermissionDecision>,
    /// Script execution
    pub scripts: Option<PermissionDecision>,
    /// Environment access
    pub environment: Option<PermissionDecision>,
    /// Child process execution
    pub child_process: Option<PermissionDecision>,
}

/// Permission manager
pub struct PermissionManager {
    /// Security configuration
    config: SecurityConfig,
    /// Per-package permissions
    package_permissions: HashMap<String, PackagePermissions>,
    /// Cached decisions (to avoid repeated prompts)
    cached_decisions: parking_lot::RwLock<HashMap<(String, Permission), PermissionDecision>>,
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            config: config.clone(),
            package_permissions: HashMap::new(),
            cached_decisions: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Check if a permission is granted for a package
    pub fn check(&self, package: &str, permission: Permission) -> PermissionDecision {
        // Check cache first
        {
            let cache = self.cached_decisions.read();
            if let Some(decision) = cache.get(&(package.to_string(), permission)) {
                return *decision;
            }
        }

        // Check trusted packages
        if self.is_trusted(package) {
            return PermissionDecision::Allow;
        }

        // Check package-specific permissions
        if let Some(perms) = self.package_permissions.get(package) {
            let decision = match permission {
                Permission::Filesystem => perms.filesystem,
                Permission::Network => perms.network,
                Permission::Scripts => perms.scripts,
                Permission::Environment => perms.environment,
                Permission::ChildProcess => perms.child_process,
            };

            if let Some(d) = decision {
                return d;
            }
        }

        // Default policy
        self.default_permission(permission)
    }

    /// Check if package is trusted
    fn is_trusted(&self, package: &str) -> bool {
        if self.config.trusted_packages.contains(&package.to_string()) {
            return true;
        }

        if package.starts_with('@') {
            if let Some(scope) = package.split('/').next() {
                if self.config.trusted_scopes.contains(&scope.to_string()) {
                    return true;
                }
            }
        }

        false
    }

    /// Get default permission for a type
    fn default_permission(&self, permission: Permission) -> PermissionDecision {
        match permission {
            Permission::Filesystem => PermissionDecision::Prompt,
            Permission::Network => PermissionDecision::Prompt,
            Permission::Scripts => {
                if self.config.allow_scripts {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny
                }
            }
            Permission::Environment => PermissionDecision::Prompt,
            Permission::ChildProcess => PermissionDecision::Deny,
        }
    }

    /// Grant a permission for a package
    pub fn grant(&self, package: &str, permission: Permission) {
        let mut cache = self.cached_decisions.write();
        cache.insert((package.to_string(), permission), PermissionDecision::Allow);
    }

    /// Deny a permission for a package
    pub fn deny(&self, package: &str, permission: Permission) {
        let mut cache = self.cached_decisions.write();
        cache.insert((package.to_string(), permission), PermissionDecision::Deny);
    }

    /// Set package permissions
    pub fn set_package_permissions(&mut self, package: &str, permissions: PackagePermissions) {
        self.package_permissions.insert(package.to_string(), permissions);
    }

    /// Clear cached decisions
    pub fn clear_cache(&self) {
        let mut cache = self.cached_decisions.write();
        cache.clear();
    }
}
