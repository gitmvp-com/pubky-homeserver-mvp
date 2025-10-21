use anyhow::{Context, Result};
use heed::{Database, EnvOpenOptions};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Storage {
    env: Arc<heed::Env>,
    db: Database<heed::types::Str, heed::types::Bytes>,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create storage directory: {}", path.display()))?;

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(10 * 1024 * 1024 * 1024) // 10 GB
                .max_dbs(1)
                .open(path)
                .with_context(|| format!("Failed to open LMDB environment at: {}", path.display()))?
        };

        let mut wtxn = env.write_txn()
            .context("Failed to create write transaction")?;
        let db = env.create_database(&mut wtxn, Some("data"))
            .context("Failed to create database")?;
        wtxn.commit()
            .context("Failed to commit database creation")?;

        Ok(Self {
            env: Arc::new(env),
            db,
        })
    }

    /// Store a key-value pair
    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut wtxn = self.env.write_txn()
            .context("Failed to create write transaction")?;
        self.db.put(&mut wtxn, key, value)
            .with_context(|| format!("Failed to store key: {}", key))?;
        wtxn.commit()
            .context("Failed to commit transaction")?;
        Ok(())
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let rtxn = self.env.read_txn()
            .context("Failed to create read transaction")?;
        let value = self.db.get(&rtxn, key)
            .with_context(|| format!("Failed to read key: {}", key))?;
        Ok(value.map(|v| v.to_vec()))
    }

    /// Delete a key
    pub fn delete(&self, key: &str) -> Result<bool> {
        let mut wtxn = self.env.write_txn()
            .context("Failed to create write transaction")?;
        let deleted = self.db.delete(&mut wtxn, key)
            .with_context(|| format!("Failed to delete key: {}", key))?;
        wtxn.commit()
            .context("Failed to commit transaction")?;
        Ok(deleted)
    }

    /// List all keys
    pub fn list_keys(&self) -> Result<Vec<String>> {
        let rtxn = self.env.read_txn()
            .context("Failed to create read transaction")?;
        let mut keys = Vec::new();
        let iter = self.db.iter(&rtxn)
            .context("Failed to create iterator")?;
        for item in iter {
            let (key, _) = item.context("Failed to read item from iterator")?;
            keys.push(key.to_string());
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage() -> Result<()> {
        let dir = tempdir()?;
        let storage = Storage::new(dir.path())?;

        // Test put and get
        storage.put("test_key", b"test_value")?;
        let value = storage.get("test_key")?;
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test list keys
        let keys = storage.list_keys()?;
        assert!(keys.contains(&"test_key".to_string()));

        // Test delete
        let deleted = storage.delete("test_key")?;
        assert!(deleted);
        let value = storage.get("test_key")?;
        assert_eq!(value, None);

        Ok(())
    }
}
