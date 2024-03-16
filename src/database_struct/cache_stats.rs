use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct CacheStats {
    pub response: String,
    pub last_updated: i64,
    pub last_page: i64,
}