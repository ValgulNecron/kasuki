use std::error::Error;
use std::sync::Arc;

use cynic::{GraphQlResponse, Operation, QueryFragment, QueryVariables};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub async fn make_request_anilist<
    'a,
    T: QueryFragment,
    S: QueryVariables + Serialize,
    U: for<'de> Deserialize<'de>,
>(
    operation: Operation<T, S>,
    always_update: bool,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>, Box<dyn Error>> {
    if !always_update {
        do_request(operation, anilist_cache).await
    } else {
        let return_data: GraphQlResponse<U> = check_cache(operation, anilist_cache).await?;
        Ok(return_data)
    }
}

async fn check_cache<
    'a,
    T: QueryFragment,
    S: QueryVariables + Serialize,
    U: for<'de> Deserialize<'de>,
>(
    operation: Operation<T, S>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>, Box<dyn Error>> {
    let anilist_cache_clone = anilist_cache.clone();
    let guard = anilist_cache_clone.read().await;
    let cache = guard.get(&operation.query).await;
    drop(guard);
    match cache {
        Some(data) => get_type(data),
        None => do_request(operation, anilist_cache).await,
    }
}

async fn do_request<
    T: QueryFragment,
    S: QueryVariables + Serialize,
    U: for<'de> Deserialize<'de>,
>(
    operation: Operation<T, S>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<GraphQlResponse<U>, Box<dyn Error>> {
    let client = Client::new();
    let resp = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&operation)
        .send()
        .await?;

    let response_text = resp.text().await?;
    anilist_cache
        .write()
        .await
        .insert(operation.query, response_text.clone())
        .await;
    get_type(response_text)
}

fn get_type<U: for<'de> Deserialize<'de>>(
    value: String,
) -> Result<GraphQlResponse<U>, Box<dyn Error>> {
    let data = serde_json::from_str::<GraphQlResponse<U>>(&value)?;
    Ok(data)
}
