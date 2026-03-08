use std::sync::{Arc, LazyLock};

use crate::cache::CacheInterface;
use anyhow::{Context, Result};
use cynic::{GraphQlResponse, Operation, QueryFragment, QueryVariables};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub async fn make_request_anilist<
	'a,
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, always_update: bool, anilist_cache: Arc<CacheInterface>,
) -> Result<GraphQlResponse<U>> {
	trace!("Starting GraphQL request to Anilist");
	debug!("GraphQL query type: {}", std::any::type_name::<T>());
	debug!("Always update cache: {}", always_update);

	if !always_update {
		info!("Bypassing cache check, making direct GraphQL request");
		do_request(operation, anilist_cache)
			.await
			.with_context(|| "Failed to make direct GraphQL request to Anilist")
	} else {
		debug!("Checking cache before making GraphQL request");
		let return_data: GraphQlResponse<U> = match check_cache(operation, anilist_cache).await {
			Ok(data) => {
				debug!("Successfully retrieved GraphQL response");
				data
			},
			Err(e) => {
				error!("GraphQL request failed: {:#}", e);
				return Err(e).with_context(|| "Failed to check cache or make GraphQL request");
			},
		};

		trace!("GraphQL request completed successfully");
		Ok(return_data)
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
	trace!("Checking cache for GraphQL query");

	let query_hash = operation.query.chars().take(20).collect::<String>();
	debug!("Query hash: {}...", query_hash);

	let key = format!(
		"{}{}",
		operation.query,
		serde_json::to_string(&operation.variables).unwrap_or_default()
	);

	trace!("Looking up query in cache");
	let cache = anilist_cache.read(&key).await?;

	match cache {
		Some(data) => {
			info!("Cache hit for GraphQL query");
			debug!("Deserializing cached response");
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
	let query_hash = operation.query.chars().take(20).collect::<String>();
	info!("Making GraphQL request to Anilist API");
	debug!("Query hash: {}...", query_hash);

	trace!("Preparing GraphQL request");
	debug!("Request URL: https://graphql.anilist.co/");

	trace!("Sending GraphQL request");
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

	trace!("Extracting response text");
	let response_text = match resp.text().await {
		Ok(text) => {
			trace!("Successfully extracted response text");
			debug!("Response size: {} bytes", text.len());
			text
		},
		Err(e) => {
			error!("Failed to extract text from response: {}", e);
			return Err::<GraphQlResponse<U>, anyhow::Error>(e.into())
				.with_context(|| "Failed to extract text from Anilist API response");
		},
	};

	let key = format!(
		"{}{}",
		operation.query,
		serde_json::to_string(&operation.variables).unwrap_or_default()
	);
	trace!("Writing to cache");
	anilist_cache.write(key, response_text.clone()).await?;
	trace!("Updated cache with new response");

	debug!("Deserializing GraphQL response");
	get_type(response_text).with_context(|| {
		format!(
			"Failed to deserialize GraphQL response for query: {}",
			operation.query
		)
	})
}

fn get_type<U: for<'de> Deserialize<'de>>(value: String) -> Result<GraphQlResponse<U>> {
	trace!("Deserializing JSON response to GraphQL type");
	debug!("Target type: {}", std::any::type_name::<U>());

	let data = match serde_json::from_str::<GraphQlResponse<U>>(&value) {
		Ok(parsed) => {
			trace!("Successfully deserialized GraphQL response");

			if let Some(errors) = &parsed.errors {
				if !errors.is_empty() {
					warn!("GraphQL response contains {} errors", errors.len());
					for (i, error) in errors.iter().enumerate() {
						warn!("GraphQL error #{}: {}", i + 1, error.message);

						if let Some(locations) = &error.locations {
							debug!("Error locations: {:?}", locations);
						}
						if let Some(path) = &error.path {
							debug!("Error path: {:?}", path);
						}
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

	debug!("GraphQL deserialization completed successfully");
	Ok(data)
}
