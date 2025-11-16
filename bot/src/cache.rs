use crate::config::CacheConfig;
use anyhow::Result;

pub struct CacheInterface {
    pub conf: CacheConfig,
}

impl CacheInterface {
    pub fn new(conf: CacheConfig) -> Self {
        Self { conf }
    }
    pub fn read(self, key: String) -> Result<String>  {
        Ok(String::new())
    }
    
    pub fn write(self, key: String, value: String) -> Result<()>  {
        Ok(())
    }
}