//! Content-addressable store for cached data

use std::path::PathBuf;
use sha2::{Sha256, Digest};

use crate::core::VelocityResult;

/// Content-addressable storage
pub struct ContentStore {
    /// Store directory
    store_dir: PathBuf,
}

impl ContentStore {
    /// Create a new content store
    pub fn new(store_dir: PathBuf) -> VelocityResult<Self> {
        std::fs::create_dir_all(&store_dir)?;
        Ok(Self { store_dir })
    }

    /// Store content by its hash
    pub fn store(&self, content: &[u8]) -> VelocityResult<String> {
        let hash = self.hash(content);
        let path = self.hash_path(&hash);

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Only write if not exists (content-addressable)
        if !path.exists() {
            std::fs::write(&path, content)?;
        }

        Ok(hash)
    }

    /// Get content by hash
    pub fn get(&self, hash: &str) -> VelocityResult<Option<Vec<u8>>> {
        let path = self.hash_path(hash);
        if path.exists() {
            Ok(Some(std::fs::read(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Check if content exists
    pub fn has(&self, hash: &str) -> bool {
        self.hash_path(hash).exists()
    }

    /// Get the path for a hash
    fn hash_path(&self, hash: &str) -> PathBuf {
        // Use first 2 chars as subdirectory for better filesystem performance
        let (prefix, rest) = hash.split_at(2.min(hash.len()));
        self.store_dir.join(prefix).join(rest)
    }

    /// Compute SHA-256 hash of content
    fn hash(&self, content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    /// Remove content by hash
    pub fn remove(&self, hash: &str) -> VelocityResult<bool> {
        let path = self.hash_path(hash);
        if path.exists() {
            std::fs::remove_file(&path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get total size of the store
    pub fn size(&self) -> VelocityResult<u64> {
        let mut total = 0u64;
        for entry in walkdir::WalkDir::new(&self.store_dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    total += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        Ok(total)
    }

    /// Clear the entire store
    pub fn clear(&self) -> VelocityResult<()> {
        if self.store_dir.exists() {
            std::fs::remove_dir_all(&self.store_dir)?;
            std::fs::create_dir_all(&self.store_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_and_retrieve() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path().to_path_buf()).unwrap();

        let content = b"Hello, World!";
        let hash = store.store(content).unwrap();

        let retrieved = store.get(&hash).unwrap().unwrap();
        assert_eq!(retrieved, content);
    }

    #[test]
    fn test_deduplication() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path().to_path_buf()).unwrap();

        let content = b"Duplicate content";
        let hash1 = store.store(content).unwrap();
        let hash2 = store.store(content).unwrap();

        assert_eq!(hash1, hash2);
    }
}
