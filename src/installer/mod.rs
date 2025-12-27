//! Package installer for Velocity
//!
//! Implements parallel downloading, extraction, and linking.

pub mod downloader;
pub mod extractor;
pub mod linker;

use std::path::PathBuf;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::core::{VelocityResult};
use crate::resolver::Resolution;
use crate::security::SecurityManager;

pub use downloader::Downloader;
pub use extractor::Extractor;
pub use linker::Linker;

/// Result of an installation
pub struct InstallResult {
    /// Number of packages installed
    pub installed_count: usize,

    /// Number of packages restored from cache
    pub cached_count: usize,

    /// Total bytes downloaded
    pub bytes_downloaded: u64,
}

/// Package installer
pub struct Installer {
    /// Project directory
    project_dir: PathBuf,

    /// Cache manager
    cache: Arc<CacheManager>,

    /// Security manager
    security: Arc<SecurityManager>,

    /// Concurrent download limit
    concurrency: usize,
}

impl Installer {
    /// Create a new installer
    pub fn new(
        project_dir: PathBuf,
        cache: Arc<CacheManager>,
        security: Arc<SecurityManager>,
        concurrency: usize,
    ) -> Self {
        Self {
            project_dir,
            cache,
            security,
            concurrency,
        }
    }

    /// Install packages from a resolution
    pub async fn install(
        &self,
        resolution: &Resolution,
        force: bool,
        prefer_offline: bool,
    ) -> VelocityResult<InstallResult> {
        let mut installed_count = 0;
        let mut cached_count = 0;
        let mut bytes_downloaded = 0u64;

        // Create downloader
        let downloader = Downloader::new(self.cache.clone(), self.concurrency);

        // Download packages that aren't cached
        for pkg in &resolution.to_install {
            if !force && self.cache.has_package(&pkg.name, &pkg.version)? {
                cached_count += 1;
                continue;
            }

            // Verify security before downloading
            self.security.verify_package_allowed(&pkg.name)?;

            // Download
            let bytes = downloader.download(pkg, prefer_offline).await?;
            bytes_downloaded += bytes;

            // Extract to cache
            let extractor = Extractor::new(self.cache.clone(), self.security.clone());
            extractor.extract(pkg).await?;

            installed_count += 1;
        }

        // Count cached packages
        cached_count += resolution.from_cache.len();

        Ok(InstallResult {
            installed_count,
            cached_count,
            bytes_downloaded,
        })
    }

    /// Link packages to node_modules
    pub async fn link(&self, resolution: &Resolution) -> VelocityResult<()> {
        let linker = Linker::new(
            self.project_dir.clone(),
            self.cache.clone(),
        );

        // Create node_modules directory
        let node_modules = self.project_dir.join("node_modules");
        if !node_modules.exists() {
            std::fs::create_dir_all(&node_modules)?;
        }

        // Create .bin directory
        let bin_dir = node_modules.join(".bin");
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir)?;
        }

        // Link all packages
        let all_packages: Vec<_> = resolution.to_install.iter()
            .chain(resolution.from_cache.iter())
            .collect();

        linker.link_packages(&all_packages).await?;

        Ok(())
    }
}
