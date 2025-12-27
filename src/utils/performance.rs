//! High-performance parallel operations

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Performance metrics collector
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Total bytes downloaded
    pub bytes_downloaded: AtomicU64,
    /// Total bytes read from cache
    pub bytes_from_cache: AtomicU64,
    /// Number of packages resolved
    pub packages_resolved: AtomicUsize,
    /// Number of packages installed
    pub packages_installed: AtomicUsize,
    /// Number of packages from cache
    pub packages_cached: AtomicUsize,
    /// Number of HTTP requests made
    pub http_requests: AtomicUsize,
    /// Number of cache hits
    pub cache_hits: AtomicUsize,
    /// Number of cache misses
    pub cache_misses: AtomicUsize,
    /// Start time
    start_time: Option<Instant>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn add_downloaded(&self, bytes: u64) {
        self.bytes_downloaded.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_from_cache(&self, bytes: u64) {
        self.bytes_from_cache.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn inc_resolved(&self) {
        self.packages_resolved.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_installed(&self) {
        self.packages_installed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cached(&self) {
        self.packages_cached.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_http_requests(&self) {
        self.http_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.map(|s| s.elapsed()).unwrap_or_default()
    }

    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            elapsed: self.elapsed(),
            bytes_downloaded: self.bytes_downloaded.load(Ordering::Relaxed),
            bytes_from_cache: self.bytes_from_cache.load(Ordering::Relaxed),
            packages_resolved: self.packages_resolved.load(Ordering::Relaxed),
            packages_installed: self.packages_installed.load(Ordering::Relaxed),
            packages_cached: self.packages_cached.load(Ordering::Relaxed),
            cache_hit_rate: {
                let hits = self.cache_hits.load(Ordering::Relaxed);
                let misses = self.cache_misses.load(Ordering::Relaxed);
                if hits + misses > 0 {
                    (hits as f64 / (hits + misses) as f64) * 100.0
                } else {
                    0.0
                }
            },
        }
    }
}

/// Summary of performance metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub elapsed: Duration,
    pub bytes_downloaded: u64,
    pub bytes_from_cache: u64,
    pub packages_resolved: usize,
    pub packages_installed: usize,
    pub packages_cached: usize,
    pub cache_hit_rate: f64,
}

impl MetricsSummary {
    pub fn download_speed(&self) -> f64 {
        if self.elapsed.as_secs_f64() > 0.0 {
            (self.bytes_downloaded as f64) / self.elapsed.as_secs_f64() / 1024.0 / 1024.0
        } else {
            0.0
        }
    }
}

/// Parallel task executor with concurrency control
pub struct ParallelExecutor {
    /// Maximum concurrent tasks
    semaphore: Arc<Semaphore>,
    /// Performance metrics
    metrics: Arc<PerformanceMetrics>,
}

impl ParallelExecutor {
    /// Create a new executor with specified concurrency
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            metrics: Arc::new(PerformanceMetrics::new()),
        }
    }

    /// Get the semaphore for rate limiting
    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    /// Get metrics
    pub fn metrics(&self) -> Arc<PerformanceMetrics> {
        self.metrics.clone()
    }

    /// Execute tasks in parallel with concurrency control
    pub async fn execute_all<T, F, Fut>(
        &self,
        items: Vec<T>,
        f: F,
    ) -> Vec<Result<(), crate::core::VelocityError>>
    where
        T: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), crate::core::VelocityError>> + Send,
    {
        use futures::stream::{self, StreamExt};

        let semaphore = self.semaphore.clone();
        let f = Arc::new(f);

        stream::iter(items)
            .map(|item| {
                let semaphore = semaphore.clone();
                let f = f.clone();
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    f(item).await
                }
            })
            .buffer_unordered(self.semaphore.available_permits())
            .collect()
            .await
    }
}

/// HTTP client optimized for npm registry
pub struct OptimizedHttpClient {
    client: reqwest::Client,
    metrics: Arc<PerformanceMetrics>,
}

impl OptimizedHttpClient {
    pub fn new(metrics: Arc<PerformanceMetrics>) -> Self {
        let client = reqwest::Client::builder()
            // Enable HTTP/2
            .http2_prior_knowledge()
            // Connection pooling
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(Duration::from_secs(90))
            // Timeouts
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(300))
            // Compression
            .gzip(true)
            .brotli(true)
            .deflate(true)
            // User agent
            .user_agent(format!("velocity/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, metrics }
    }

    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.metrics.inc_http_requests();
        self.client.get(url).send().await
    }

    pub async fn get_bytes(&self, url: &str) -> Result<bytes::Bytes, reqwest::Error> {
        self.metrics.inc_http_requests();
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        self.metrics.add_downloaded(bytes.len() as u64);
        Ok(bytes)
    }
}

/// Memory-efficient string pool for deduplication
pub struct StringPool {
    pool: dashmap::DashMap<String, Arc<str>>,
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            pool: dashmap::DashMap::new(),
        }
    }

    /// Intern a string, returning a reference-counted pointer
    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Some(existing) = self.pool.get(s) {
            return existing.clone();
        }
        
        let arc: Arc<str> = s.into();
        self.pool.insert(s.to_string(), arc.clone());
        arc
    }

    /// Get pool size
    pub fn len(&self) -> usize {
        self.pool.len()
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool() {
        let pool = StringPool::new();
        
        let a = pool.intern("react");
        let b = pool.intern("react");
        
        // Should be the same Arc
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn test_metrics() {
        let metrics = PerformanceMetrics::new();
        
        metrics.add_downloaded(1000);
        metrics.inc_installed();
        metrics.cache_hit();
        
        let summary = metrics.summary();
        assert_eq!(summary.bytes_downloaded, 1000);
        assert_eq!(summary.packages_installed, 1);
    }
}
