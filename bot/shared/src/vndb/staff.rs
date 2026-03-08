use anyhow::Result;
use std::sync::Arc;

use crate::cache::CacheInterface;
use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn get_staff(
	value: String, vndb_cache: Arc<CacheInterface>, client: &reqwest::Client,
) -> Result<StaffRoot> {
	let value = value.to_lowercase();

	let value = value.trim();

	let start_with_v = value.starts_with('v');

	let is_number = value.chars().skip(1).all(|c| c.is_numeric());

	let json = if start_with_v && is_number {
		format!(
			r#"{{"filters": ["id", "=", "{}"], "fields": "id,aid,ismain,name,lang,gender,description"}}"#,
			value
		)
	} else {
		format!(
			r#"{{"filters": ["search", "=", "{}"], "fields": "id,aid,ismain,name,lang,gender,description"}}"#,
			value
		)
	};

	let path = "/staff".to_string();

	let response = crate::vndb::common::do_request_cached_with_json(
		path.clone(),
		json.to_string(),
		vndb_cache,
		client,
	)
	.await?;

	info!("VNDB response: {}", response);

	let response: StaffRoot = serde_json::from_str(&response)?;

	Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Staff {
	pub ismain: bool,

	pub aid: i32,

	pub name: String,

	pub gender: Option<String>,

	pub lang: String,

	pub description: String,

	pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct StaffRoot {
	pub results: Vec<Staff>,

	pub more: bool,
}
