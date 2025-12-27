//! Parallel package downloader

use std::sync::Arc;
use futures::stream::{self, StreamExt};

use crate::cache::CacheManager;
use crate::core::{VelocityError, VelocityResult};
use crate::resolver::ResolvedPackage;

/// Parallel package downloader
pub struct Downloader {
    /// Cache manager
    cache: Arc<CacheManager>,

    /// HTTP client
    client: reqwest::Client,

    /// Maximum concurrent downloads
    concurrency: usize,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(cache: Arc<CacheManager>, concurrency: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .gzip(true)
            .brotli(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            cache,
            client,
            concurrency,
        }
    }

    /// Download a single package
    pub async fn download(&self, package: &ResolvedPackage, prefer_offline: bool) -> VelocityResult<u64> {
        // Check cache first
        if prefer_offline {
            if self.cache.has_package(&package.name, &package.version)? {
                return Ok(0);
            }
        }

        // Download tarball
        let response = self.client
            .get(&package.tarball_url)
            .send()
            .await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VelocityError::Network(format!(
                "Failed to download {}: HTTP {}",
                package.name,
                response.status()
            )));
        }

        let content_length = response.content_length().unwrap_or(0);

        // Get the bytes
        let bytes = response.bytes().await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        // Verify integrity if provided
        if !package.integrity.is_empty() {
            self.verify_integrity(&bytes, &package.integrity, &package.name)?;
        }

        // Save to cache
        self.cache.store_tarball(&package.name, &package.version, &bytes)?;

        Ok(content_length)
    }

    /// Download multiple packages in parallel
    pub async fn download_all(&self, packages: &[ResolvedPackage]) -> VelocityResult<u64> {
        let total_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));

        let results: Vec<VelocityResult<()>> = stream::iter(packages.iter())
            .map(|pkg| {
                let client = self.client.clone();
                let cache = self.cache.clone();
                let total = total_bytes.clone();
                let pkg = pkg.clone();

                async move {
                    // Check cache
                    if cache.has_package(&pkg.name, &pkg.version)? {
                        return Ok(());
                    }

                    // Download
                    let response = client
                        .get(&pkg.tarball_url)
                        .send()
                        .await
                        .map_err(|e| VelocityError::Network(e.to_string()))?;

                    if !response.status().is_success() {
                        return Err(VelocityError::Network(format!(
                            "Failed to download {}: HTTP {}",
                            pkg.name,
                            response.status()
                        )));
                    }

                    let bytes = response.bytes().await
                        .map_err(|e| VelocityError::Network(e.to_string()))?;

                    // Verify integrity
                    if !pkg.integrity.is_empty() {
                        verify_integrity_static(&bytes, &pkg.integrity, &pkg.name)?;
                    }

                    // Store
                    cache.store_tarball(&pkg.name, &pkg.version, &bytes)?;

                    total.fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);

                    Ok(())
                }
            })
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        // Check for errors
        for result in results {
            result?;
        }

        Ok(total_bytes.load(std::sync::atomic::Ordering::Relaxed))
    }

    /// Verify package integrity
    fn verify_integrity(&self, data: &[u8], integrity: &str, package: &str) -> VelocityResult<()> {
        verify_integrity_static(data, integrity, package)
    }
}

/// Static integrity verification function
fn verify_integrity_static(data: &[u8], integrity: &str, package: &str) -> VelocityResult<()> {
    use sha2::{Sha256, Sha512, Digest};

    let (algorithm, expected_hash) = if let Some(hash) = integrity.strip_prefix("sha512-") {
        ("sha512", hash)
    } else if let Some(hash) = integrity.strip_prefix("sha256-") {
        ("sha256", hash)
    } else {
        // Unknown format, skip verification but warn
        tracing::warn!("Unknown integrity format for {}: {}", package, integrity);
        return Ok(());
    };

    let computed_hash = match algorithm {
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hasher.finalize())
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hasher.finalize())
        }
        _ => return Ok(()), // Unknown algorithm
    };

    if computed_hash != expected_hash {
        return Err(VelocityError::IntegrityCheckFailed {
            package: package.to_string(),
            expected: expected_hash.to_string(),
            actual: computed_hash,
        });
    }

    Ok(())
}
