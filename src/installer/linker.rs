//! Package linker for node_modules

use std::path::PathBuf;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::core::VelocityResult;
use crate::resolver::ResolvedPackage;

/// Package linker
pub struct Linker {
    /// Project directory
    project_dir: PathBuf,

    /// Cache manager
    cache: Arc<CacheManager>,
}

impl Linker {
    /// Create a new linker
    pub fn new(project_dir: PathBuf, cache: Arc<CacheManager>) -> Self {
        Self { project_dir, cache }
    }

    /// Link packages to node_modules
    pub async fn link_packages(&self, packages: &[&ResolvedPackage]) -> VelocityResult<()> {
        let node_modules = self.project_dir.join("node_modules");

        for package in packages {
            let source = self.cache.get_package_dir(&package.name, &package.version);
            
            if !source.exists() {
                tracing::warn!("Package not in cache: {}@{}", package.name, package.version);
                continue;
            }

            // Determine target path (handle scoped packages)
            let target = if package.name.starts_with('@') {
                let parts: Vec<&str> = package.name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let scope_dir = node_modules.join(parts[0]);
                    std::fs::create_dir_all(&scope_dir)?;
                    scope_dir.join(parts[1])
                } else {
                    node_modules.join(&package.name)
                }
            } else {
                node_modules.join(&package.name)
            };

            // Remove existing if present
            if target.exists() {
                std::fs::remove_dir_all(&target)?;
            }

            // Try to create hard link or copy
            self.link_or_copy(&source, &target)?;

            // Link binaries
            self.link_binaries(&target, &package.name)?;
        }

        Ok(())
    }

    /// Link or copy a package
    fn link_or_copy(&self, source: &PathBuf, target: &PathBuf) -> VelocityResult<()> {
        // Try hard linking first (fastest)
        #[cfg(unix)]
        {
            if let Err(_) = std::os::unix::fs::symlink(source, target) {
                // Fall back to copy
                self.copy_dir(source, target)?;
            }
            return Ok(());
        }

        #[cfg(windows)]
        {
            // On Windows, try junction for directories
            if let Err(_) = junction::create(source, target) {
                // Fall back to copy
                self.copy_dir(source, target)?;
            }
            return Ok(());
        }

        #[cfg(not(any(unix, windows)))]
        {
            self.copy_dir(source, target)?;
            Ok(())
        }
    }

    /// Copy a directory recursively
    fn copy_dir(&self, source: &PathBuf, target: &PathBuf) -> VelocityResult<()> {
        std::fs::create_dir_all(target)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                self.copy_dir(&source_path, &target_path)?;
            } else {
                std::fs::copy(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// Link binary executables
    fn link_binaries(&self, package_dir: &PathBuf, package_name: &str) -> VelocityResult<()> {
        let bin_dir = self.project_dir.join("node_modules").join(".bin");

        // Read package.json to find binaries
        let package_json_path = package_dir.join("package.json");
        if !package_json_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&package_json_path)?;
        let pkg: serde_json::Value = serde_json::from_str(&content)?;

        // Handle "bin" field
        if let Some(bin) = pkg.get("bin") {
            match bin {
                serde_json::Value::String(path) => {
                    // Single binary with package name
                    let bin_name = package_name.split('/').last().unwrap_or(package_name);
                    self.create_bin_link(&bin_dir, bin_name, package_dir, path)?;
                }
                serde_json::Value::Object(bins) => {
                    // Multiple binaries
                    for (name, path) in bins {
                        if let Some(path_str) = path.as_str() {
                            self.create_bin_link(&bin_dir, name, package_dir, path_str)?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a binary link
    fn create_bin_link(
        &self,
        bin_dir: &PathBuf,
        name: &str,
        package_dir: &PathBuf,
        path: &str,
    ) -> VelocityResult<()> {
        let source = package_dir.join(path);
        
        if !source.exists() {
            return Ok(());
        }

        #[cfg(unix)]
        {
            let target = bin_dir.join(name);
            let _ = std::fs::remove_file(&target);
            std::os::unix::fs::symlink(&source, &target)?;

            // Make executable
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&source)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&source, perms)?;
        }

        #[cfg(windows)]
        {
            // Create cmd wrapper on Windows
            let cmd_target = bin_dir.join(format!("{}.cmd", name));
            let source_relative = pathdiff::diff_paths(&source, bin_dir)
                .unwrap_or_else(|| source.clone());
            
            let cmd_content = format!(
                "@ECHO off\r\nnode \"%~dp0\\{}\" %*\r\n",
                source_relative.display()
            );
            std::fs::write(&cmd_target, cmd_content)?;

            // Also create a PowerShell script
            let ps1_target = bin_dir.join(format!("{}.ps1", name));
            let ps1_content = format!(
                "#!/usr/bin/env pwsh\r\nnode \"$PSScriptRoot\\{}\" $args\r\nexit $LASTEXITCODE\r\n",
                source_relative.display()
            );
            std::fs::write(&ps1_target, ps1_content)?;
        }

        Ok(())
    }
}
