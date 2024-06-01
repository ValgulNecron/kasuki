use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Cache {
    pub json: String,
    pub resp: String,
    pub last_updated: i64,
}