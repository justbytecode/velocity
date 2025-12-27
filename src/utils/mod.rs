//! Utility functions for Velocity

mod performance;

use std::path::Path;
use sha2::{Sha256, Digest};

pub use performance::*;

/// Compute SHA-256 hash of data
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of a file
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let data = std::fs::read(path)?;
    Ok(sha256(&data))
}

/// Normalize a package name for filesystem storage
pub fn normalize_package_name(name: &str) -> String {
    name.replace('/', "+").replace('@', "")
}

/// Check if a path is safe (no traversal)
pub fn is_safe_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    !path_str.contains("..") && !path.is_absolute()
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Format duration as human-readable string
pub fn format_duration(millis: u128) -> String {
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60000 {
        format!("{:.2}s", millis as f64 / 1000.0)
    } else {
        let seconds = millis / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}m {}s", minutes, remaining_seconds)
    }
}

/// Check if running in CI environment
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("CONTINUOUS_INTEGRATION").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("TRAVIS").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
}

/// Get the current platform triple
pub fn platform_triple() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
}

/// Parse package specifier (name@version)
pub fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if spec.starts_with('@') {
        // Scoped package like @scope/name@version
        if let Some(at_idx) = spec[1..].find('@') {
            let at_idx = at_idx + 1; // Adjust for the initial @
            let name = spec[..at_idx].to_string();
            let version = spec[at_idx + 1..].to_string();
            return (name, Some(version));
        }
        (spec.to_string(), None)
    } else if let Some(at_idx) = spec.find('@') {
        let name = spec[..at_idx].to_string();
        let version = spec[at_idx + 1..].to_string();
        (name, Some(version))
    } else {
        (spec.to_string(), None)
    }
}

/// Validate package name
pub fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 214 {
        return false;
    }

    // Must not start with . or _
    if name.starts_with('.') || name.starts_with('_') {
        return false;
    }

    // Must be lowercase
    if name != name.to_lowercase() {
        return false;
    }

    // Check valid characters
    let valid_chars = |c: char| {
        c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || c == '-'
            || c == '_'
            || c == '.'
            || c == '@'
            || c == '/'
    };

    name.chars().all(valid_chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"Hello, World!");
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_normalize_package_name() {
        assert_eq!(normalize_package_name("react"), "react");
        assert_eq!(normalize_package_name("@types/node"), "types+node");
        assert_eq!(normalize_package_name("@scope/package"), "scope+package");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("foo/bar")));
        assert!(!is_safe_path(Path::new("../foo")));
        assert!(!is_safe_path(Path::new("foo/../bar")));
    }

    #[test]
    fn test_parse_package_spec() {
        assert_eq!(parse_package_spec("react"), ("react".to_string(), None));
        assert_eq!(
            parse_package_spec("react@18.2.0"),
            ("react".to_string(), Some("18.2.0".to_string()))
        );
        assert_eq!(
            parse_package_spec("@types/node@20.0.0"),
            ("@types/node".to_string(), Some("20.0.0".to_string()))
        );
    }

    #[test]
    fn test_valid_package_name() {
        assert!(is_valid_package_name("react"));
        assert!(is_valid_package_name("@types/node"));
        assert!(is_valid_package_name("lodash.debounce"));
        assert!(!is_valid_package_name(""));
        assert!(!is_valid_package_name("React")); // uppercase
        assert!(!is_valid_package_name("_private")); // starts with _
    }
}
