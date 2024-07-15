use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::helper::error_management::error_enum::UnknownResponseError;
use crate::helper::vndbapi::common::do_request_cached;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VnUser {
    pub id: String,

    pub lengthvotes: i32,

    pub lengthvotes_sum: i32,

    pub username: String,
}
pub async fn get_user(
    path: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<VnUser, Box<dyn Error>> {
    let response = do_request_cached(path.clone(), vndb_cache).await?;
    let response: HashMap<String, VnUser> = serde_json::from_str(&response)
        .map_err(|e| UnknownResponseError::Json(format!("{:#?}", e)))?;
    let response = response.into_iter().next().unwrap().1;
    Ok(response)
}
