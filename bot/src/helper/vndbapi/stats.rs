use std::sync::Arc;

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::trace;

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
use anyhow::Result;

pub async fn get_stats(vndb_cache: Arc<RwLock<Cache<String, String>>>) -> Result<Stats> {
	let path = "/stats".to_string();

	let response = do_request_cached(path.clone(), vndb_cache).await?;

	trace!("Response: {}", response);

	let response: Stats = serde_json::from_str(&response)?;

	Ok(response)
}
