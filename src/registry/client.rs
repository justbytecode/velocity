//! Registry HTTP client

use std::sync::Arc;

use crate::cache::CacheManager;
use crate::core::{VelocityResult, VelocityError};
use crate::core::config::RegistryConfig;
use crate::registry::types::PackageMetadata;

/// npm registry client
pub struct RegistryClient {
    /// HTTP client
    client: reqwest::Client,
    /// Registry configuration
    config: RegistryConfig,
    /// Cache manager
    cache: Arc<CacheManager>,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: &RegistryConfig, cache: Arc<CacheManager>) -> VelocityResult<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8"
                .parse()
                .unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            format!("velocity/{}", env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .gzip(true)
            .brotli(true)
            .build()
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        Ok(Self {
            client,
            config: config.clone(),
            cache,
        })
    }

    /// Get package metadata from the registry
    pub async fn get_package_metadata(&self, name: &str) -> VelocityResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_metadata(name)? {
            let metadata: PackageMetadata = serde_json::from_str(&cached.data)?;
            return Ok(metadata);
        }

        // Fetch from registry
        let url = self.get_package_url(name);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(VelocityError::PackageNotFound(name.to_string()));
            }
            return Err(VelocityError::Registry(format!(
                "Failed to fetch {}: HTTP {}",
                name,
                response.status()
            )));
        }

        let text = response.text().await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        // Parse and validate
        let metadata: PackageMetadata = serde_json::from_str(&text)?;

        // Cache the response
        self.cache.store_metadata(name, &text)?;

        Ok(metadata)
    }

    /// Get the URL for a package
    fn get_package_url(&self, name: &str) -> String {
        let registry = self.get_registry_for_package(name);
        
        // Handle scoped packages
        let encoded_name = if name.starts_with('@') {
            name.replace('/', "%2f")
        } else {
            name.to_string()
        };

        format!("{}/{}", registry, encoded_name)
    }

    /// Get the registry URL for a package (handles scoped overrides)
    fn get_registry_for_package(&self, name: &str) -> &str {
        if name.starts_with('@') {
            if let Some(scope) = name.split('/').next() {
                if let Some(registry) = self.config.scopes.get(scope) {
                    return registry;
                }
            }
        }

        &self.config.url
    }

    /// Check if a package exists
    pub async fn package_exists(&self, name: &str) -> VelocityResult<bool> {
        let url = self.get_package_url(name);

        let response = self.client
            .head(&url)
            .send()
            .await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Get authentication token for a registry
    pub fn get_auth_token(&self, registry: &str) -> Option<&String> {
        self.config.auth_tokens.get(registry)
    }

    /// Search packages
    pub async fn search(&self, query: &str, limit: usize) -> VelocityResult<Vec<SearchResult>> {
        let url = format!("{}/-/v1/search?text={}&size={}", self.config.url, query, limit);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(VelocityError::Registry(format!(
                "Search failed: HTTP {}",
                response.status()
            )));
        }

        let data: SearchResponse = response.json().await
            .map_err(|e| VelocityError::Network(e.to_string()))?;

        Ok(data.objects.into_iter().map(|o| o.package).collect())
    }
}

/// Search response from npm registry
#[derive(Debug, serde::Deserialize)]
struct SearchResponse {
    objects: Vec<SearchObject>,
}

#[derive(Debug, serde::Deserialize)]
struct SearchObject {
    package: SearchResult,
}

/// Search result
#[derive(Debug, serde::Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
}
