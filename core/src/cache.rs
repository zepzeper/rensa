use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const DEFAULT_TTL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Debug)]
pub enum CacheError {
    NotFound(PathBuf),
    ReadError(std::io::Error),
    WriteError(std::io::Error),
    DeserializationError(serde_json::Error),
    SerializationError(serde_json::Error),
    Expired,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound(path) => write!(f, "Cache directory not found: {}", path.display()),
            CacheError::ReadError(e) => write!(f, "Cache read error: {}", e),
            CacheError::WriteError(e) => write!(f, "Cache write error: {}", e),
            CacheError::DeserializationError(e) => write!(f, "Cache deserialization error: {}", e),
            CacheError::SerializationError(e) => write!(f, "Cache serialization error: {}", e),
            CacheError::Expired => write!(f, "Cache expired"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<serde_json::Error> for CacheError {
    fn from(e: serde_json::Error) -> Self {
        CacheError::SerializationError(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    pub ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        Self {
            data,
            timestamp: SystemTime::UNIX_EPOCH
                .elapsed()
                .unwrap_or_default()
                .as_secs(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return true;
        }

        let elapsed = SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        elapsed > self.timestamp + self.ttl_seconds
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub trait Cacheable {
    fn cache_key(&self) -> String;
    fn cache_dir(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct CacheManager {
    base_path: PathBuf,
    default_ttl: Duration,
}

impl CacheManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            default_ttl: DEFAULT_TTL,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    pub fn path_for(&self, dir: &str, key: &str) -> PathBuf {
        self.base_path.join(dir).join(format!("{}.json", key))
    }

    pub fn get<T>(&self, dir: &str, key: &str) -> Result<Option<CacheEntry<T>>, CacheError>
    where
        T: for<'a> serde::de::Deserialize<'a>,
    {
        let cache_path = self.path_for(dir, key);

        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path).map_err(CacheError::ReadError)?;
        let entry: CacheEntry<T> = serde_json::from_str(&content)
            .map_err(CacheError::DeserializationError)?;

        if entry.is_expired() {
            return Ok(None);
        }

        Ok(Some(entry))
    }

    pub fn set<T>(&self, dir: &str, key: &str, data: &T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let cache_path = self.path_for(dir, key);

        if let Some(parent) = cache_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(CacheError::WriteError)?;
            }
        }

        let entry = CacheEntry::new(data, self.default_ttl.as_secs());
        let content = serde_json::to_string_pretty(&entry)?;
        fs::write(&cache_path, content).map_err(CacheError::WriteError)?;

        Ok(())
    }

    pub fn exists(&self, dir: &str, key: &str) -> bool {
        self.path_for(dir, key).exists()
    }

    pub fn clear(&self, dir: &str) -> Result<(), CacheError> {
        let dir_path = self.base_path.join(dir);

        if dir_path.exists() {
            fs::remove_dir_all(&dir_path).map_err(CacheError::WriteError)?;
            fs::create_dir_all(&dir_path).map_err(CacheError::WriteError)?;
        }

        Ok(())
    }

    pub fn clean_expired(&self, dir: &str) -> Result<(), CacheError> {
        let dir_path = self.base_path.join(dir);

        if !dir_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&dir_path).map_err(CacheError::ReadError)? {
            let entry = entry.map_err(CacheError::ReadError)?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path).map_err(CacheError::ReadError) {
                    if let Ok(entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&content) {
                        if entry.is_expired() {
                            fs::remove_file(&path).map_err(CacheError::WriteError)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn sanitize_cache_key(key: &str) -> String {
    key.replace('/', "-")
        .replace(':', "_")
        .replace(' ', "_")
        .to_lowercase()
}

pub fn create_cache_manager_from_env() -> Option<CacheManager> {
    std::env::var("RENSA_CACHE_DIR")
        .ok()
        .map(|path| CacheManager::new(PathBuf::from(path)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path().to_path_buf());

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let key = sanitize_cache_key("test-package-1.0.0");
        cache.set("test", &key, &data).unwrap();

        let retrieved = cache.get::<TestData>("test", &key).unwrap().unwrap();
        assert_eq!(retrieved.data.name, "test");
        assert_eq!(retrieved.data.value, 42);
    }

    #[tokio::test]
    async fn test_cache_missing() {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path().to_path_buf());

        let result = cache.get::<TestData>("test", "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_expired() {
        let temp_dir = TempDir::new().unwrap();
        let cache = CacheManager::new(temp_dir.path().to_path_buf()).with_ttl(Duration::from_secs(0));

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let key = sanitize_cache_key("test-package-1.0.0");
        cache.set("test", &key, &data).unwrap();

        let result = cache.get::<TestData>("test", &key).unwrap();
        assert!(result.is_none());
    }
}
