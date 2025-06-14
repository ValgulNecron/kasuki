use std::sync::Arc;

use anyhow::{Context, Result};
use cynic::{GraphQlResponse, Operation, QueryFragment, QueryVariables};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

/// Makes a GraphQL request to the Anilist API, with optional caching.
///
/// This function serves as the main entry point for GraphQL requests to Anilist.
/// It supports two modes of operation:
///
/// 1. Direct request mode (`always_update = false`): Bypasses the cache check and
///    directly makes a network request. This is useful when fresh data is required.
///
/// 2. Cache-first mode (`always_update = true`): Checks the cache before making a
///    network request. If the query exists in cache, returns the cached result.
///    Otherwise, makes a network request and updates the cache.
///
/// # Cache Implementation Details
///
/// The cache uses the GraphQL query string as the key and the raw JSON response
/// as the value. This approach allows caching responses regardless of the specific
/// GraphQL type, but requires type-specific deserialization when retrieving from cache.
///
/// # Error Handling
///
/// Errors are propagated with context to help with debugging. Network errors,
/// deserialization errors, and GraphQL-level errors are all handled and logged
/// appropriately.
///
pub async fn make_request_anilist<
	'a,
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, always_update: bool,
	anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>> {
	trace!("Starting GraphQL request to Anilist");
	debug!("GraphQL query type: {}", std::any::type_name::<T>());
	debug!("Always update cache: {}", always_update);

	// The parameter name is counterintuitive - when always_update is false,
	// we bypass the cache and always make a network request.
	// When always_update is true, we check the cache first.
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

/// Checks if a GraphQL query is in the cache and returns the cached result if found.
///
/// This function implements the cache lookup logic for GraphQL queries. It follows these steps:
/// 1. Acquires a read lock on the cache to prevent concurrent modifications
/// 2. Checks if the query exists in the cache
/// 3. If found, deserializes the cached JSON string to the requested type
/// 4. If not found, makes a network request to fetch the data
///
/// # Concurrency Considerations
///
/// The function uses a read lock to allow multiple concurrent readers but blocks writers.
/// The lock is dropped as soon as possible to minimize contention.
///
/// # Cache Key
///
/// The full GraphQL query string is used as the cache key. This ensures that queries
/// with different variables or selections are cached separately.
///
async fn check_cache<
	'a,
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>> {
	trace!("Checking cache for GraphQL query");

	// Create a short hash of the query for logging purposes
	// This makes logs more readable while still allowing query identification
	let query_hash = operation.query.chars().take(20).collect::<String>();
	debug!("Query hash: {}...", query_hash);

	// Clone the cache reference to avoid ownership issues
	let anilist_cache_clone = anilist_cache.clone();

	// Acquire a read lock on the cache
	// Using a read lock allows multiple concurrent readers
	trace!("Acquiring read lock on cache");
	let guard = anilist_cache_clone.read().await;

	// Look up the query in the cache
	trace!("Looking up query in cache");
	let cache = guard.get(&operation.query).await;

	// Drop the lock as soon as possible to reduce contention
	drop(guard);
	trace!("Released cache read lock");

	match cache {
		Some(data) => {
			// Cache hit - deserialize the cached JSON string
			info!("Cache hit for GraphQL query");
			debug!("Deserializing cached response");
			get_type(data).with_context(|| "Failed to deserialize cached GraphQL response")
		},
		None => {
			// Cache miss - make a network request
			info!("Cache miss for GraphQL query, making network request");
			do_request(operation, anilist_cache)
				.await
				.with_context(|| "Failed to make GraphQL request after cache miss")
		},
	}
}

/// Makes a network request to the Anilist GraphQL API and caches the response.
///
/// This function is responsible for:
/// 1. Creating an HTTP client
/// 2. Sending the GraphQL request to the Anilist API
/// 3. Processing the HTTP response
/// 4. Caching the response for future use
/// 5. Deserializing the response to the requested type
///
/// # Network Considerations
///
/// The function makes a synchronous HTTP request using reqwest, which may take
/// significant time depending on network conditions. It includes proper error
/// handling for network failures.
///
/// # Caching Behavior
///
/// After a successful request, the raw JSON response is cached using the GraphQL
/// query string as the key. This allows future requests with the same query to
/// be served from cache without making a network request.
///
/// # Rate Limiting
///
/// The Anilist API has rate limits, and this function does not currently implement
/// retry logic for rate limit errors. In a production environment, consider adding
/// rate limit detection and backoff logic.
///
async fn do_request<
	T: QueryFragment,
	S: QueryVariables + Serialize,
	U: for<'de> Deserialize<'de>,
>(
	operation: Operation<T, S>, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>> {
	// Create a short hash of the query for logging purposes
	let query_hash = operation.query.chars().take(20).collect::<String>();
	info!("Making GraphQL request to Anilist API");
	debug!("Query hash: {}...", query_hash);

	// Create a new HTTP client for this request
	// reqwest clients are relatively cheap to create and can be reused
	trace!("Creating HTTP client");
	let client = Client::new();

	// Prepare the GraphQL request with appropriate headers
	// Content-Type and Accept headers are required for GraphQL requests
	trace!("Preparing GraphQL request");
	debug!("Request URL: https://graphql.anilist.co/");

	// Send the request and handle any network errors
	trace!("Sending GraphQL request");
	let resp = match client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&operation)  // Serialize the operation to JSON
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

	// Extract the response text and handle any errors
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

	// Cache the response for future use
	// This requires a write lock on the cache, which blocks other writers
	trace!("Acquiring write lock on cache");
	anilist_cache
		.write()
		.await
		.insert(operation.query.clone(), response_text.clone())
		.await;
	trace!("Updated cache with new response");

	// Deserialize the response to the requested type
	debug!("Deserializing GraphQL response");
	get_type(response_text).with_context(|| {
		format!(
			"Failed to deserialize GraphQL response for query: {}",
			operation.query
		)
	})
}

/// Deserializes a JSON string into a GraphQL response of the specified type.
///
/// This function is responsible for converting the raw JSON response from the
/// Anilist API into a strongly-typed GraphQL response. It handles both successful
/// responses and responses containing GraphQL-level errors.
///
/// # Type Parameters
///
/// * `U` - The expected data type in the GraphQL response. This is typically
///   a struct generated by the cynic crate that maps to the GraphQL schema.
///
/// # GraphQL Error Handling
///
/// GraphQL responses can contain errors even with a successful HTTP status code.
/// This function checks for GraphQL-level errors in the response and logs them
/// as warnings. The calling code should check the `errors` field of the returned
/// `GraphQlResponse` to handle these errors appropriately.
///
/// # JSON Parsing
///
/// The function uses serde_json to parse the JSON string. If the JSON is invalid
/// or doesn't match the expected structure, a deserialization error is returned.
///
fn get_type<U: for<'de> Deserialize<'de>>(value: String) -> Result<GraphQlResponse<U>> {
	trace!("Deserializing JSON response to GraphQL type");
	debug!("Target type: {}", std::any::type_name::<U>());

	let data = match serde_json::from_str::<GraphQlResponse<U>>(&value) {
		Ok(parsed) => {
			trace!("Successfully deserialized GraphQL response");

			// Check for GraphQL-level errors in the response
			// A GraphQL response can contain errors even with a successful HTTP status code
			// These errors need to be logged and handled appropriately
			if let Some(errors) = &parsed.errors {
				if !errors.is_empty() {
					warn!("GraphQL response contains {} errors", errors.len());
					for (i, error) in errors.iter().enumerate() {
						warn!("GraphQL error #{}: {}", i + 1, error.message);

						// Log additional error details if available
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
			// JSON parsing failed - this could be due to invalid JSON or
			// a mismatch between the JSON structure and the expected type
			error!("Failed to deserialize GraphQL response: {}", e);
			return Err::<GraphQlResponse<U>, anyhow::Error>(e.into())
				.with_context(|| "Failed to parse JSON response from GraphQL");
		},
	};

	debug!("GraphQL deserialization completed successfully");
	Ok(data)
}
