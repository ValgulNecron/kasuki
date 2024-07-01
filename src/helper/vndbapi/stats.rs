use std::sync::Arc;

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::trace;

use crate::helper::error_management::error_enum::AppError;
use crate::helper::vndbapi::common::do_request_cached;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    pub chars: i32,

    pub producers: i32,

    pub releases: i32,

    pub staff: i32,

    pub tags: i32,

    pub traits: i32,

    pub vn: i32,
}
pub async fn get_stats(vndb_cache: Arc<RwLock<Cache<String, String>>>) -> Result<Stats, AppError> {
    let path = "/stats".to_string();
    let response = do_request_cached(path.clone(), vndb_cache).await?;
    trace!("Response: {}", response);
    let response: Stats = serde_json::from_str(&response).map_err(|e| AppError {
        message: format!("Error while parsing response: '{}'", e),
        error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
        error_response_type:
            crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
    })?;
    Ok(response)
}
