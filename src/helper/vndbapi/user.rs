use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::AppError;
use crate::helper::vndbapi::common::do_request_cached;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VnUser {
    pub id: String,

    pub lengthvotes: i32,

    pub lengthvotes_sum: i32,

    pub username: String,
}
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn get_user(
    path: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<VnUser, AppError> {
    let response = do_request_cached(path.clone(), vndb_cache).await?;
    let response: HashMap<String, VnUser> =
        serde_json::from_str(&response).map_err(|e| AppError {
            message: format!("Error while parsing response: '{}'", e),
            error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
            error_response_type:
                crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
        })?;
    let response = response.into_iter().next().unwrap().1;
    Ok(response)
}
