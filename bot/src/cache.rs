use crate::config::CacheConfig;
use anyhow::Result;
use moka::future::Cache;
use std::sync::Arc;

pub struct CacheInterface {
    pub conf: CacheConfig,
    pub cache: Arc<Cache<String, String>>,
}

impl CacheInterface {
    pub fn new(conf: CacheConfig) -> Self {
        // Use default cache configuration
        let cache = Cache::builder()
            .max_capacity(10_000) // Default max capacity
            .time_to_live(std::time::Duration::from_secs(3600)) // Default 1 hour TTL
            .build();
        
        Self {
            conf,
            cache: Arc::new(cache),
        }
    }
    
    pub async fn read(&self, key: String) -> Result<Option<String>> {
        Ok(self.cache.get(&key).await)
    }
    
    pub async fn write(&self, key: String, value: String) -> Result<()> {
        self.cache.insert(key, value).await;
        Ok(())
    }
    
    pub fn get_cache(&self) -> Arc<Cache<String, String>> {
        self.cache.clone()
    }
}