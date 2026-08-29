use std::sync::{Arc, LazyLock};

use crate::cache::CacheInterface;
use anyhow::{Context, Result};
use cynic::{GraphQlResponse, Operation, QueryFragment, QueryVariables};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub async fn make_request_anilist<
	'a,
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, use_cache: bool, anilist_cache: Arc<CacheInterface>,
) -> Result<GraphQlResponse<U>> {
	if use_cache {
		info!("Checking cache");
		let return_data: GraphQlResponse<U> = match check_cache(operation, anilist_cache).await {
			Ok(data) => data,
			Err(e) => {
				error!("GraphQL request failed: {:#}", e);
				return Err(e).with_context(|| "Failed to check cache or make GraphQL request");
			},
		};

		Ok(return_data)
	} else {
		info!("Bypassing cache, making direct request");
		do_request(operation, anilist_cache)
			.await
			.with_context(|| "Failed to make direct GraphQL request to Anilist")
	}
}

async fn check_cache<
	'a,
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, anilist_cache: Arc<CacheInterface>,
) -> Result<GraphQlResponse<U>> {
	// Key = query text + serialized variables so identical queries with different params get separate cache entries
	let key = format!(
		"{}{}",
		operation.query,
		serde_json::to_string(&operation.variables).unwrap_or_default()
	);

	let cache = anilist_cache.read(&key).await?;

	match cache {
		Some(data) => {
			info!("Cache hit for GraphQL query");
			get_type(data).with_context(|| "Failed to deserialize cached GraphQL response")
		},
		None => {
			info!("Cache miss for GraphQL query, making network request");
			do_request(operation, anilist_cache)
				.await
				.with_context(|| "Failed to make GraphQL request after cache miss")
		},
	}
}

async fn do_request<
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, anilist_cache: Arc<CacheInterface>,
) -> Result<GraphQlResponse<U>> {
	let resp = match HTTP_CLIENT
		.post("https://graphql.anilist.co/")
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.json(&operation)
		.send()
		.await
	{
		Ok(resp) => {
			debug!("Received response with status: {}", resp.status());
			resp
		},
		Err(e) => {
			error!("Failed to send GraphQL request: {}", e);
			return Err::<GraphQlResponse<U>, anyhow::Error>(e.into())
				.with_context(|| "Failed to send GraphQL request to Anilist API");
		},
	};

	// Cache raw JSON text instead of deserialized types so the cache is type-agnostic
	let response_text = match resp.text().await {
		Ok(text) => text,
		Err(e) => {
			error!("Failed to extract text from response: {}", e);
			return Err::<GraphQlResponse<U>, anyhow::Error>(e.into())
				.with_context(|| "Failed to extract text from Anilist API response");
		},
	};

	// Reconstruct the same key used in check_cache so reads and writes stay consistent
	let key = format!(
		"{}{}",
		operation.query,
		serde_json::to_string(&operation.variables).unwrap_or_default()
	);
	anilist_cache.write(key, response_text.clone()).await?;

	get_type(response_text).with_context(|| {
		format!(
			"Failed to deserialize GraphQL response for query: {}",
			operation.query
		)
	})
}

// Deserializes raw JSON into the caller's expected response type (deferred from cache/network layer)
fn get_type<U: for<'de> Deserialize<'de>>(value: String) -> Result<GraphQlResponse<U>> {
	let data = match serde_json::from_str::<GraphQlResponse<U>>(&value) {
		Ok(parsed) => {
			// GraphQL can return partial data alongside errors; log them but don't fail
			if let Some(errors) = &parsed.errors {
				if !errors.is_empty() {
					warn!("GraphQL response contains {} errors", errors.len());
					for (i, error) in errors.iter().enumerate() {
						warn!("GraphQL error #{}: {}", i + 1, error.message);
					}
				}
			}

			parsed
		},
		Err(e) => {
			error!("Failed to deserialize GraphQL response: {}", e);
			return Err::<GraphQlResponse<U>, anyhow::Error>(e.into())
				.with_context(|| "Failed to parse JSON response from GraphQL");
		},
	};

	Ok(data)
}
