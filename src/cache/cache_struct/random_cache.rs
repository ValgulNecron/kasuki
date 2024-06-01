use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// CacheStats is a struct that represents the statistics of a cache.
/// It is derived from a row in a SQL database and can be serialized and deserialized.
#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct RandomCache {
    /// response is a String that represents the response of the cache.
    pub response: String,
    /// last_updated is an i64 that represents the timestamp of the last update of the cache.
    pub last_updated: i64,
    /// last_page is an i64 that represents the last page of the cache.
    pub last_page: i64,
    /// random_type is a String that represents the key of the cache.
    pub random_type: String,
}
