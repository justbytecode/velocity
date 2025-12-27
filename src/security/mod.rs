//! Security module for Velocity

pub mod ecosystem;
pub mod integrity;
pub mod permissions;
pub mod sandbox;
pub mod supply_chain;

use crate::core::VelocityResult;
use crate::core::config::SecurityConfig;

pub use ecosystem::{EcosystemAnalyzer, EcosystemCategory, SecurityLevel};
pub use permissions::PermissionManager;
pub use supply_chain::{SupplyChainGuard, SecurityAnalysis, RiskLevel};

/// Security manager for enforcing security policies
pub struct SecurityManager {
    config: SecurityConfig,
    permissions: PermissionManager,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            config: config.clone(),
            permissions: PermissionManager::new(config),
        }
    }

    /// Check if a package is allowed to be installed
    pub fn verify_package_allowed(&self, name: &str) -> VelocityResult<()> {
        // Check trusted packages/scopes
        if self.is_trusted(name) {
            return Ok(());
        }

        // Dependency confusion protection
        if self.config.dependency_confusion_protection {
            self.check_dependency_confusion(name)?;
        }

        Ok(())
    }

    /// Check if a package is trusted
    pub fn is_trusted(&self, name: &str) -> bool {
        // Check exact package name
        if self.config.trusted_packages.contains(&name.to_string()) {
            return true;
        }

        // Check scope
        if name.starts_with('@') {
            if let Some(scope) = name.split('/').next() {
                if self.config.trusted_scopes.contains(&scope.to_string()) {
                    return true;
                }
            }
        }

        false
    }

    /// Check for dependency confusion attacks
    fn check_dependency_confusion(&self, name: &str) -> VelocityResult<()> {
        // Scoped packages are less susceptible to dependency confusion
        if name.starts_with('@') {
            return Ok(());
        }

        // Check for suspicious naming patterns
        let suspicious_patterns = [
            "-internal",
            "-private",
            "-corp",
            "-company",
        ];

        for pattern in &suspicious_patterns {
            if name.contains(pattern) {
                tracing::warn!(
                    "Package '{}' matches suspicious pattern '{}'. Consider using a scoped package.",
                    name, pattern
                );
            }
        }

        Ok(())
    }

    /// Check if scripts are allowed
    pub fn scripts_allowed(&self) -> bool {
        self.config.allow_scripts
    }

    /// Check if a script should run for a package
    pub fn should_run_script(&self, package: &str, script: &str) -> VelocityResult<bool> {
        if !self.config.allow_scripts {
            return Ok(false);
        }

        if self.is_trusted(package) {
            return Ok(true);
        }

        // Could prompt user here
        Ok(false)
    }

    /// Check if audit is required on install
    pub fn audit_on_install(&self) -> bool {
        self.config.audit_on_install
    }

    /// Get the permission manager
    pub fn permissions(&self) -> &PermissionManager {
        &self.permissions
    }
}
