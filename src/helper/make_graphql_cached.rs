use crate::cache::cache_struct::cache::Cache;
use crate::cache::manage::cache_dispatch::{get_database_cache, set_database_cache};
use crate::constant::TIME_BETWEEN_CACHE_UPDATE;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

pub async fn make_request_anilist<'a, T: Deserialize<'a>>(json: Value, always_update: bool) -> Result<T, AppError> {
    return if always_update {
        do_request(json, always_update).await
    } else {
        let return_data: T = get_cache(json.clone()).await?;
        Ok(return_data)
    };
}

async fn get_cache<'a, T: Deserialize<'a>>(json: Value) -> Result<T, AppError> {
    let cache: Option<Cache> = get_database_cache(json.clone()).await.unwrap_or(None);

    if cache.is_none() {
        let return_data: T = do_request(json.clone(), false).await?;
        return Ok(return_data);
    }

    let cache = cache.ok_or(AppError {
        message: "Cache not found".to_string(),
        error_type: ErrorType::Database,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    let updated_at = cache.last_updated;
    let duration_since_updated = Utc::now().timestamp() - updated_at;

    if duration_since_updated >= TIME_BETWEEN_CACHE_UPDATE as i64 {
        let return_data: T = do_request(json.clone(), false).await?;
        return Ok(return_data);
    }
    let json: &'a str = Box::leak(cache.resp.into_boxed_str());
    let return_data: T = serde_json::from_str(json).map_err(|e| AppError {
        message: format!("Failed to parse: {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    return Ok(return_data);
}

async fn add_cache(json: Value, resp: String) -> Result<bool, AppError> {
    let now = Utc::now().timestamp();
    let cache = Cache {
        json: json.to_string(),
        resp: resp.clone(),
        last_updated: now,
    };
    set_database_cache(cache).await?;
    Ok(true)
}

async fn do_request<'a, T: Deserialize<'a>>(json: Value, always_update: bool) -> Result<T, AppError> {
    let client = Client::new();
    let res = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.clone().to_string())
        .send()
        .await
        .map_err(|e| AppError {
            message: format!("Timeout: {}", e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Unknown,
        })?
        .text()
        .await;
    let resp = res.map_err(|e| AppError {
        message: format!("Failed to get the response: {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    if !always_update {
        add_cache(json.clone(), resp.clone()).await?;
    }
    let json: &'a str = Box::leak(resp.into_boxed_str());
    let return_data: T = serde_json::from_str(json).map_err(|e| AppError {
        message: format!("Failed to parse: {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;

    Ok(return_data)
}