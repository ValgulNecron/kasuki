use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::vndb::common::do_request_cached;

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
use crate::cache::CacheInterface;
use anyhow::Result;

pub async fn get_stats(vndb_cache: Arc<CacheInterface>, client: &reqwest::Client) -> Result<Stats> {
	let path = "/stats".to_string();

	let response = do_request_cached(path.clone(), vndb_cache, client).await?;

	trace!("Response: {}", response);

	let response: Stats = serde_json::from_str(&response)?;

	Ok(response)
}
