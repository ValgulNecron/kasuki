use anyhow::Result;
use std::sync::Arc;

use crate::cache::CacheInterface;
use tokio::sync::RwLock;

pub async fn do_request_cached(
	path: String, vndb_cache: Arc<RwLock<CacheInterface>>,
	client: &reqwest::Client,
) -> Result<String> {
	let cache = vndb_cache.read().await.read(&path).await?;

	if let Some(cached) = cache {
		return Ok(cached);
	}

	do_request(path, vndb_cache, client).await
}

pub async fn do_request(
	path: String, vndb_cache: Arc<RwLock<CacheInterface>>, client: &reqwest::Client,
) -> Result<String> {
	let url = format!("https://api.vndb.org/kana{}", path);

	let res = client
		.get(url)
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.send()
		.await?;

	let response_text = res.text().await?;

	vndb_cache
		.write()
		.await
		.write(path, response_text.clone())
		.await?;

	Ok(response_text)
}

pub async fn do_request_cached_with_json(
	path: String, json: String, vndb_cache: Arc<RwLock<CacheInterface>>,
	client: &reqwest::Client,
) -> Result<String> {
	let key = format!("{}_{}", path, json);

	let cache = vndb_cache.read().await.read(&key).await?;

	if let Some(cached) = cache {
		return Ok(cached);
	}

	do_request_with_json(path, json, vndb_cache, client).await
}

pub async fn do_request_with_json(
	path: String, json: String, vndb_cache: Arc<RwLock<CacheInterface>>,
	client: &reqwest::Client,
) -> Result<String> {
	let key = format!("{}_{}", path, json);

	let url = format!("https://api.vndb.org/kana{}", path);

	let res = client
		.post(url)
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.body(json)
		.send()
		.await?;

	let response_text = res.text().await?;

	vndb_cache
		.write()
		.await
		.write(key, response_text.clone())
		.await?;

	Ok(response_text)
}
