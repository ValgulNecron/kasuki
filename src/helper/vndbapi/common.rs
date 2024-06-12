use crate::constant::VNDB_CACHE;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use tracing::trace;

pub async fn do_request_cached(path: String) -> Result<String, AppError> {
    let cache = unsafe { VNDB_CACHE.get(&path).await };
    if let Some(cached) = cache {
        return Ok(cached);
    }
    do_request(path).await
}

pub async fn do_request(path: String) -> Result<String, AppError> {
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
    unsafe {
        VNDB_CACHE.insert(path.clone(), response_text.clone()).await;
    }
    trace!("Response: {}", response_text);
    Ok(response_text)
}

pub async fn do_request_cached_with_json(path: String, json: String) -> Result<String, AppError> {
    let cache = unsafe { VNDB_CACHE.get(&path).await };
    if let Some(cached) = cache {
        return Ok(cached);
    }
    do_request_with_json(path, json).await
}

pub async fn do_request_with_json(path: String, json: String) -> Result<String, AppError> {
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
    unsafe {
        VNDB_CACHE.insert(path.clone(), response_text.clone()).await;
    }
    trace!("Response: {}", response_text);
    Ok(response_text)
}
