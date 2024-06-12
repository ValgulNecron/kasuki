use crate::constant::VNDB_CACHE;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn do_request_cached(path: String, value: String) -> Result<String, AppError> {
    let key = format!("{}_{}", path, value);
    let cache = unsafe { VNDB_CACHE.get(&key).await };
    if let Some(cached) = cache {
        return Ok(cached);
    }
    do_request(path, key).await
}

pub async fn do_request(path: String, key: String) -> Result<String, AppError> {
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
        VNDB_CACHE.insert(key.clone(), response_text.clone()).await;
    }
    Ok(response_text)
}
