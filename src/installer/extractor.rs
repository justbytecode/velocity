//! Package tarball extractor with security checks

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::cache::CacheManager;
use crate::core::{VelocityError, VelocityResult};
use crate::resolver::ResolvedPackage;
use crate::security::SecurityManager;

/// Package extractor
pub struct Extractor {
    /// Cache manager
    cache: Arc<CacheManager>,

    /// Security manager
    security: Arc<SecurityManager>,
}

impl Extractor {
    /// Create a new extractor
    pub fn new(cache: Arc<CacheManager>, security: Arc<SecurityManager>) -> Self {
        Self { cache, security }
    }

    /// Extract a package from its tarball
    pub async fn extract(&self, package: &ResolvedPackage) -> VelocityResult<PathBuf> {
        let tarball_path = self.cache.get_tarball_path(&package.name, &package.version);

        if !tarball_path.exists() {
            return Err(VelocityError::cache(format!(
                "Tarball not found for {}@{}",
                package.name, package.version
            )));
        }

        let extract_dir = self.cache.get_package_dir(&package.name, &package.version);

        // Skip if already extracted
        if extract_dir.exists() {
            return Ok(extract_dir);
        }

        // Create extraction directory
        std::fs::create_dir_all(&extract_dir)?;

        // Read tarball
        let tarball_data = std::fs::read(&tarball_path)?;

        // Decompress
        let decoder = GzDecoder::new(&tarball_data[..]);
        let mut archive = Archive::new(decoder);

        // Extract with security checks
        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.into_owned();

            // Security check: path traversal protection
            self.check_path_traversal(&entry_path, &package.name)?;

            // npm packages have a "package/" prefix
            let relative_path = entry_path
                .strip_prefix("package/")
                .or_else(|_| entry_path.strip_prefix("package"))
                .unwrap_or(&entry_path);

            let target_path = extract_dir.join(relative_path);

            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract file
            if entry.header().entry_type().is_file() {
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;
                std::fs::write(&target_path, content)?;

                // Set permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(mode) = entry.header().mode() {
                        let _ = std::fs::set_permissions(
                            &target_path,
                            std::fs::Permissions::from_mode(mode),
                        );
                    }
                }
            } else if entry.header().entry_type().is_dir() {
                std::fs::create_dir_all(&target_path)?;
            }
        }

        Ok(extract_dir)
    }

    /// Check for path traversal attacks
    fn check_path_traversal(&self, path: &Path, package: &str) -> VelocityResult<()> {
        let path_str = path.to_string_lossy();

        // Check for ..
        if path_str.contains("..") {
            return Err(VelocityError::PathTraversal {
                package: package.to_string(),
                path: path_str.to_string(),
            });
        }

        // Check for absolute paths
        if path.is_absolute() {
            return Err(VelocityError::PathTraversal {
                package: package.to_string(),
                path: path_str.to_string(),
            });
        }

        // Check for suspicious characters
        if path_str.contains('\0') {
            return Err(VelocityError::PathTraversal {
                package: package.to_string(),
                path: "null byte in path".to_string(),
            });
        }

        Ok(())
    }
}
