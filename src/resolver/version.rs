//! SemVer version constraint parsing and matching

use crate::core::{VelocityError, VelocityResult};

/// A version constraint (e.g., ^1.0.0, ~2.1.0, >=3.0.0)
#[derive(Debug, Clone)]
pub enum VersionConstraint {
    /// Exact version (1.0.0)
    Exact(semver::Version),
    /// Caret range (^1.0.0 - compatible with 1.x.x)
    Caret(semver::Version),
    /// Tilde range (~1.0.0 - compatible with 1.0.x)
    Tilde(semver::Version),
    /// Greater than or equal (>=1.0.0)
    GreaterOrEqual(semver::Version),
    /// Greater than (>1.0.0)
    GreaterThan(semver::Version),
    /// Less than or equal (<=1.0.0)
    LessOrEqual(semver::Version),
    /// Less than (<1.0.0)
    LessThan(semver::Version),
    /// Any version (*)
    Any,
    /// Range (>=1.0.0 <2.0.0)
    Range(Box<VersionConstraint>, Box<VersionConstraint>),
    /// Latest tag
    Latest,
}

impl VersionConstraint {
    /// Parse a version constraint string
    pub fn parse(s: &str) -> VelocityResult<Self> {
        let s = s.trim();

        // Handle special cases
        if s.is_empty() || s == "*" || s == "latest" {
            return Ok(VersionConstraint::Any);
        }

        // Handle workspace protocol
        if s.starts_with("workspace:") {
            return Ok(VersionConstraint::Any);
        }

        // Handle npm/file/git protocols
        if s.starts_with("npm:") || s.starts_with("file:") || s.starts_with("git") || s.contains("://") {
            return Ok(VersionConstraint::Any);
        }

        // Handle x-ranges (1.x, 1.0.x)
        if s.contains('x') || s.contains('X') {
            let cleaned = s.replace('x', "0").replace('X', "0");
            if let Ok(v) = semver::Version::parse(&cleaned) {
                return Ok(VersionConstraint::Caret(v));
            }
        }

        // Handle range with space (>=1.0.0 <2.0.0)
        if s.contains(' ') {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 {
                let left = Self::parse(parts[0])?;
                let right = Self::parse(parts[1])?;
                return Ok(VersionConstraint::Range(Box::new(left), Box::new(right)));
            }
        }

        // Handle || (or)
        if s.contains("||") {
            // For simplicity, use the first constraint
            let first = s.split("||").next().unwrap().trim();
            return Self::parse(first);
        }

        // Handle hyphen range (1.0.0 - 2.0.0)
        if s.contains(" - ") {
            let parts: Vec<&str> = s.split(" - ").collect();
            if parts.len() == 2 {
                let left = Self::parse_version(parts[0].trim())?;
                let right = Self::parse_version(parts[1].trim())?;
                return Ok(VersionConstraint::Range(
                    Box::new(VersionConstraint::GreaterOrEqual(left)),
                    Box::new(VersionConstraint::LessOrEqual(right)),
                ));
            }
        }

        // Handle prefix operators
        if let Some(rest) = s.strip_prefix(">=") {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::GreaterOrEqual(v));
        }

        if let Some(rest) = s.strip_prefix("<=") {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::LessOrEqual(v));
        }

        if let Some(rest) = s.strip_prefix('>') {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::GreaterThan(v));
        }

        if let Some(rest) = s.strip_prefix('<') {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::LessThan(v));
        }

        if let Some(rest) = s.strip_prefix('^') {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::Caret(v));
        }

        if let Some(rest) = s.strip_prefix('~') {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::Tilde(v));
        }

        if let Some(rest) = s.strip_prefix('=') {
            let v = Self::parse_version(rest.trim())?;
            return Ok(VersionConstraint::Exact(v));
        }

        // Try parsing as exact version
        match Self::parse_version(s) {
            Ok(v) => Ok(VersionConstraint::Exact(v)),
            Err(_) => {
                // Fallback to Any for unparseable constraints
                tracing::warn!("Could not parse version constraint: {}, treating as any", s);
                Ok(VersionConstraint::Any)
            }
        }
    }

    /// Parse a version string, handling partial versions
    fn parse_version(s: &str) -> VelocityResult<semver::Version> {
        let s = s.trim().trim_start_matches('v');

        // Handle partial versions
        let parts: Vec<&str> = s.split('.').collect();
        let version_str = match parts.len() {
            1 => format!("{}.0.0", parts[0]),
            2 => format!("{}.{}.0", parts[0], parts[1]),
            _ => s.to_string(),
        };

        // Remove any pre-release or build metadata for initial parse
        let base_version = version_str.split('-').next().unwrap_or(&version_str);
        let base_version = base_version.split('+').next().unwrap_or(base_version);

        semver::Version::parse(base_version)
            .or_else(|_| semver::Version::parse(&version_str))
            .map_err(|_| VelocityError::InvalidVersionConstraint(s.to_string()))
    }

    /// Check if a version matches this constraint
    pub fn matches(&self, version: &semver::Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Caret(v) => {
                if v.major == 0 {
                    if v.minor == 0 {
                        // ^0.0.x -> >=0.0.x <0.0.(x+1)
                        version.major == 0 && version.minor == 0 && version.patch == v.patch
                    } else {
                        // ^0.y.z -> >=0.y.z <0.(y+1).0
                        version.major == 0 && version.minor == v.minor && version.patch >= v.patch
                    }
                } else {
                    // ^x.y.z -> >=x.y.z <(x+1).0.0
                    version.major == v.major && version >= v
                }
            }
            VersionConstraint::Tilde(v) => {
                // ~x.y.z -> >=x.y.z <x.(y+1).0
                version.major == v.major && version.minor == v.minor && version.patch >= v.patch
            }
            VersionConstraint::GreaterOrEqual(v) => version >= v,
            VersionConstraint::GreaterThan(v) => version > v,
            VersionConstraint::LessOrEqual(v) => version <= v,
            VersionConstraint::LessThan(v) => version < v,
            VersionConstraint::Any | VersionConstraint::Latest => true,
            VersionConstraint::Range(left, right) => left.matches(version) && right.matches(version),
        }
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Exact(v) => write!(f, "{}", v),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
            VersionConstraint::GreaterOrEqual(v) => write!(f, ">={}", v),
            VersionConstraint::GreaterThan(v) => write!(f, ">{}", v),
            VersionConstraint::LessOrEqual(v) => write!(f, "<={}", v),
            VersionConstraint::LessThan(v) => write!(f, "<{}", v),
            VersionConstraint::Any => write!(f, "*"),
            VersionConstraint::Latest => write!(f, "latest"),
            VersionConstraint::Range(l, r) => write!(f, "{} {}", l, r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_caret() {
        let c = VersionConstraint::parse("^1.0.0").unwrap();
        let v1 = semver::Version::new(1, 0, 0);
        let v2 = semver::Version::new(1, 5, 0);
        let v3 = semver::Version::new(2, 0, 0);

        assert!(c.matches(&v1));
        assert!(c.matches(&v2));
        assert!(!c.matches(&v3));
    }

    #[test]
    fn test_parse_tilde() {
        let c = VersionConstraint::parse("~1.2.0").unwrap();
        let v1 = semver::Version::new(1, 2, 0);
        let v2 = semver::Version::new(1, 2, 5);
        let v3 = semver::Version::new(1, 3, 0);

        assert!(c.matches(&v1));
        assert!(c.matches(&v2));
        assert!(!c.matches(&v3));
    }

    #[test]
    fn test_parse_range() {
        let c = VersionConstraint::parse(">=1.0.0 <2.0.0").unwrap();
        let v1 = semver::Version::new(1, 0, 0);
        let v2 = semver::Version::new(1, 9, 9);
        let v3 = semver::Version::new(2, 0, 0);

        assert!(c.matches(&v1));
        assert!(c.matches(&v2));
        assert!(!c.matches(&v3));
    }
}
