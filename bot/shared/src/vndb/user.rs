use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::vndb::common::do_request_cached;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct VnUser {
	pub id: String,

	pub lengthvotes: i32,

	pub lengthvotes_sum: i32,

	pub username: String,
}

use crate::cache::CacheInterface;
use anyhow::Result;

pub async fn get_user(
	path: String, vndb_cache: Arc<RwLock<CacheInterface>>, client: &reqwest::Client,
) -> Result<VnUser> {
	let response = do_request_cached(path.clone(), vndb_cache, client).await?;

	let response: HashMap<String, VnUser> = serde_json::from_str(&response)?;

	let response = response.into_iter().next().unwrap().1;

	Ok(response)
}
