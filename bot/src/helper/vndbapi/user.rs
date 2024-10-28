use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::helper::vndbapi::common::do_request_cached;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct VnUser {
	pub id: String,

	pub lengthvotes: i32,

	pub lengthvotes_sum: i32,

	pub username: String,
}

use anyhow::Result;
pub async fn get_user(
	path: String, vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<VnUser> {
	let response = do_request_cached(path.clone(), vndb_cache).await?;

	let response: HashMap<String, VnUser> = serde_json::from_str(&response)?;

	let response = response.into_iter().next().unwrap().1;

	Ok(response)
}
