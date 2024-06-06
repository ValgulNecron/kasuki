use crate::cache::cache_struct::cache::Cache;
use crate::cache::manage::cache_dispatch::{get_database_cache, set_database_cache};
use crate::constant::TIME_BETWEEN_CACHE_UPDATE;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

/// Makes a request to Anilist's GraphQL API.
///
/// This function takes a JSON value and a boolean as parameters.
/// If the boolean is true, it directly makes a request to the API.
/// If the boolean is false, it first tries to get the response from the cache.
/// If the cache does not exist or is outdated, it makes a request to the API.
///
/// # Arguments
///
/// * `json` - A JSON value that represents the request body.
/// * `always_update` - A boolean that determines whether to always make a request to the API.
///
/// # Returns
///
/// * A string that represents the response from the API or the cache.
pub async fn make_request_anilist(json: Value, always_update: bool) -> String {
    if always_update {
        do_request(json, always_update).await
    } else {
        get_cache(json.clone()).await
    }
}

/// Retrieves the response from the cache.
///
/// This function takes a JSON value as a parameter and tries to get the response from the cache.
/// If the cache does not exist or is outdated, it makes a request to the API.
///
/// # Arguments
///
/// * `json` - A JSON value that represents the request body.
///
/// # Returns
///
/// * A string that represents the response from the API or the cache.
async fn get_cache(json: Value) -> String {
    let cache: Option<Cache> = get_database_cache(json.clone()).await.unwrap_or(None);

    if cache.is_none() {
        do_request(json.clone(), false).await
    } else {
        let cache = cache.unwrap();
        let updated_at = cache.last_updated;
        let duration_since_updated = Utc::now().timestamp() - updated_at;
        if duration_since_updated < TIME_BETWEEN_CACHE_UPDATE as i64 {
            cache.resp
        } else {
            do_request(json.clone(), false).await
        }
    }
}

/// Adds the response to the cache.
///
/// This function takes a JSON value and a string as parameters and adds the response to the cache.
///
/// # Arguments
///
/// * `json` - A JSON value that represents the request body.
/// * `resp` - A string that represents the response from the API.
///
/// # Returns
///
/// * A boolean that indicates whether the operation was successful.
async fn add_cache(json: Value, resp: String) -> bool {
    let now = Utc::now().timestamp();
    let cache = Cache {
        json: json.to_string(),
        resp: resp.clone(),
        last_updated: now,
    };
    set_database_cache(cache).await.unwrap_or(());
    true
}

/// Makes a request to Anilist's GraphQL API.
///
/// This function takes a JSON value and a boolean as parameters.
/// It makes a request to the API and adds the response to the cache if the boolean is false.
///
/// # Arguments
///
/// * `json` - A JSON value that represents the request body.
/// * `always_update` - A boolean that determines whether to add the response to the cache.
///
/// # Returns
///
/// * A string that represents the response from the API.
async fn do_request(json: Value, always_update: bool) -> String {
    let client = Client::new();
    let res = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.clone().to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let resp = res.unwrap();
    if !always_update {
        add_cache(json.clone(), resp.clone()).await;
    }
    resp
}
