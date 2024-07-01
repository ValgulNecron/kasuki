use std::sync::Arc;

use moka::future::Cache;
use tokio::sync::RwLock;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn do_request_cached(
    path: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<String, AppError> {
    let cache = vndb_cache.read().await.get(&path).await;
    if let Some(cached) = cache {
        return Ok(cached);
    }
    do_request(path, vndb_cache).await
}

pub async fn do_request(
    path: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.vndb.org/kana{}", path);
    let res = client
        .get(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError {
            message: format!("Error while sending request: '{}'", e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Unknown,
        })?;
    let response_text = res.text().await.map_err(|e| AppError {
        message: format!("Error while reading response: '{}'", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    vndb_cache
        .write()
        .await
        .insert(path, response_text.clone())
        .await;
    Ok(response_text)
}

pub async fn do_request_cached_with_json(
    path: String,
    json: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<String, AppError> {
    let key = format!("{}_{}", path, json);
    let cache = vndb_cache.read().await.get(&key).await;
    if let Some(cached) = cache {
        return Ok(cached);
    }
    do_request_with_json(path, json, vndb_cache).await
}

pub async fn do_request_with_json(
    path: String,
    json: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<String, AppError> {
    let key = format!("{}_{}", path, json);
    let client = reqwest::Client::new();
    let url = format!("https://api.vndb.org/kana{}", path);
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json)
        .send()
        .await
        .map_err(|e| AppError {
            message: format!("Error while sending request: '{}'", e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Unknown,
        })?;
    let response_text = res.text().await.map_err(|e| AppError {
        message: format!("Error while reading response: '{}'", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    vndb_cache
        .write()
        .await
        .insert(key, response_text.clone())
        .await;
    Ok(response_text)
}
