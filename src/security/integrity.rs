//! Integrity verification for packages

use sha2::{Sha256, Sha512, Digest};

use crate::core::{VelocityResult, VelocityError};

/// Integrity checker for package verification
pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Verify integrity of data against an integrity string
    pub fn verify(data: &[u8], integrity: &str) -> VelocityResult<bool> {
        if integrity.is_empty() {
            return Ok(true); // No integrity to check
        }

        let (algorithm, expected) = Self::parse_integrity(integrity)?;
        let computed = Self::compute_hash(data, &algorithm);

        Ok(computed == expected)
    }

    /// Verify and return detailed result
    pub fn verify_detailed(
        data: &[u8],
        integrity: &str,
        package: &str,
    ) -> VelocityResult<()> {
        if integrity.is_empty() {
            return Ok(());
        }

        let (algorithm, expected) = Self::parse_integrity(integrity)?;
        let computed = Self::compute_hash(data, &algorithm);

        if computed != expected {
            return Err(VelocityError::IntegrityCheckFailed {
                package: package.to_string(),
                expected,
                actual: computed,
            });
        }

        Ok(())
    }

    /// Compute integrity hash for data
    pub fn compute(data: &[u8], algorithm: &str) -> String {
        let hash = Self::compute_hash(data, algorithm);
        format!("{}-{}", algorithm, hash)
    }

    /// Parse an integrity string into algorithm and hash
    fn parse_integrity(integrity: &str) -> VelocityResult<(String, String)> {
        if let Some(hash) = integrity.strip_prefix("sha512-") {
            return Ok(("sha512".to_string(), hash.to_string()));
        }

        if let Some(hash) = integrity.strip_prefix("sha256-") {
            return Ok(("sha256".to_string(), hash.to_string()));
        }

        if let Some(hash) = integrity.strip_prefix("sha1-") {
            return Ok(("sha1".to_string(), hash.to_string()));
        }

        Err(VelocityError::other(format!(
            "Unsupported integrity format: {}",
            integrity
        )))
    }

    /// Compute hash using specified algorithm
    fn compute_hash(data: &[u8], algorithm: &str) -> String {
        match algorithm {
            "sha512" => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hasher.finalize(),
                )
            }
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hasher.finalize(),
                )
            }
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        let data = b"Hello, World!";
        let hash = IntegrityChecker::compute(data, "sha256");
        assert!(hash.starts_with("sha256-"));
    }

    #[test]
    fn test_verify_sha256() {
        let data = b"Hello, World!";
        let integrity = IntegrityChecker::compute(data, "sha256");
        assert!(IntegrityChecker::verify(data, &integrity).unwrap());
    }
}
