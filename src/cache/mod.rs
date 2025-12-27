//! Content-addressable cache for Velocity

pub mod store;

use std::path::{Path, PathBuf};

use crate::core::VelocityResult;
use crate::core::config::CacheConfig;

pub use store::ContentStore;

/// Cache manager for package storage
pub struct CacheManager {
    /// Cache root directory
    cache_dir: PathBuf,

    /// Content store for tarballs
    content_store: ContentStore,

    /// Configuration
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(cache_dir: &Path, config: &CacheConfig) -> VelocityResult<Self> {
        let cache_dir = cache_dir.to_path_buf();
        
        // Create cache directories
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(cache_dir.join("tarballs"))?;
        std::fs::create_dir_all(cache_dir.join("content"))?;
        std::fs::create_dir_all(cache_dir.join("metadata"))?;

        let content_store = ContentStore::new(cache_dir.join("content"))?;

        Ok(Self {
            cache_dir,
            content_store,
            config: config.clone(),
        })
    }

    /// Check if a package is cached
    pub fn has_package(&self, name: &str, version: &str) -> VelocityResult<bool> {
        let package_dir = self.get_package_dir(name, version);
        Ok(package_dir.exists())
    }

    /// Get the path to a package's extracted directory
    pub fn get_package_dir(&self, name: &str, version: &str) -> PathBuf {
        let safe_name = name.replace('/', "+").replace('@', "");
        self.cache_dir.join("content").join(&safe_name).join(version)
    }

    /// Get the path to a package's tarball
    pub fn get_tarball_path(&self, name: &str, version: &str) -> PathBuf {
        let safe_name = name.replace('/', "+").replace('@', "");
        self.cache_dir
            .join("tarballs")
            .join(format!("{}-{}.tgz", safe_name, version))
    }

    /// Store a tarball in the cache
    pub fn store_tarball(&self, name: &str, version: &str, data: &[u8]) -> VelocityResult<()> {
        let tarball_path = self.get_tarball_path(name, version);
        
        // Ensure parent directory exists
        if let Some(parent) = tarball_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&tarball_path, data)?;
        Ok(())
    }

    /// Get cached metadata for a package
    pub fn get_metadata(&self, name: &str) -> VelocityResult<Option<CachedMetadata>> {
        let safe_name = name.replace('/', "+").replace('@', "");
        let metadata_path = self.cache_dir.join("metadata").join(format!("{}.json", safe_name));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&metadata_path)?;
        let cached: CachedMetadata = serde_json::from_str(&content)?;

        // Check TTL
        let age = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - cached.cached_at;

        if age > self.config.metadata_ttl {
            // Expired
            return Ok(None);
        }

        Ok(Some(cached))
    }

    /// Store metadata for a package
    pub fn store_metadata(&self, name: &str, data: &str) -> VelocityResult<()> {
        let safe_name = name.replace('/', "+").replace('@', "");
        let metadata_path = self.cache_dir.join("metadata").join(format!("{}.json", safe_name));

        let cached = CachedMetadata {
            data: data.to_string(),
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let content = serde_json::to_string(&cached)?;
        std::fs::write(&metadata_path, content)?;

        Ok(())
    }

    /// Clear the entire cache
    pub fn clear(&self) -> VelocityResult<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(self.cache_dir.join("tarballs"))?;
            std::fs::create_dir_all(self.cache_dir.join("content"))?;
            std::fs::create_dir_all(self.cache_dir.join("metadata"))?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> VelocityResult<CacheStats> {
        let mut total_size = 0u64;
        let mut package_count = 0usize;
        let mut tarball_count = 0usize;

        // Count content
        let content_dir = self.cache_dir.join("content");
        if content_dir.exists() {
            for entry in walkdir::WalkDir::new(&content_dir) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                    } else if entry.file_type().is_dir() && entry.depth() == 2 {
                        package_count += 1;
                    }
                }
            }
        }

        // Count tarballs
        let tarball_dir = self.cache_dir.join("tarballs");
        if tarball_dir.exists() {
            for entry in std::fs::read_dir(&tarball_dir)? {
                if let Ok(entry) = entry {
                    if entry.file_type()?.is_file() {
                        tarball_count += 1;
                        total_size += entry.metadata()?.len();
                    }
                }
            }
        }

        Ok(CacheStats {
            total_size,
            package_count,
            tarball_count,
        })
    }

    /// Check if offline mode is enabled
    pub fn is_offline(&self) -> bool {
        self.config.offline
    }
}

/// Cached metadata entry
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CachedMetadata {
    pub data: String,
    pub cached_at: u64,
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_size: u64,
    pub package_count: usize,
    pub tarball_count: usize,
}
