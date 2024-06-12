use crate::cache::manage::cache_dispatch::set_cache;
use cynic::{Operation, QueryFragment, QueryVariables};
use reqwest::{Client, Response};
use serde::Serialize;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn make_request_anilist<T: QueryFragment, A: QueryVariables + Serialize>(
    operation: Operation<T, A>,
    always_update: bool,
) -> Result<String, AppError> {
    do_request(operation).await
    /*else {
        let return_data: T = get_cache(operation).await?;
        Ok(return_data)
    };*/
}
/*
async fn get_cache<'a, T: Deserialize<'a>, A: QueryVariables>(operation: Operation<T, A>,) -> Result<T, AppError> {
    let cache: Option<Cache> = get_database_cache(operation).await.unwrap_or(None);

    if cache.is_none() {
        let return_data: T = do_request(operation), false).await?;
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
        let return_data: T = do_request(operation.clone(), false).await?;
        return Ok(return_data);
    }
    let json: &'a str = Box::leak(cache.resp.into_boxed_str());
    trace!(?json);
    let return_data: T = serde_json::from_str(json).map_err(|e| AppError {
        message: format!("Failed to parse: {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    return Ok(return_data);
}

async fn add_cache<'a, T: Deserialize<'a>, A: QueryVariables>(operation: Operation<T, A>, resp: String) -> Result<bool, AppError> {
    let now = Utc::now().timestamp();
    let cache = Cache {
        json: operation.query,
        resp: resp.clone(),
        last_updated: now,
    };
    set_database_cache(cache).await?;
    Ok(true)
}
*/
async fn do_request<T: QueryFragment, A: QueryVariables + Serialize>(
    operation: Operation<T, A>,
) -> Result<String, AppError> {
    let client = Client::new();
    let resp = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&operation)
        .send()
        .await
        .map_err(|e| AppError {
            message: format!("Timeout: {}", e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Unknown,
        })?;

    let response_text = resp.text().await.map_err(|e| AppError {
        message: format!("Error: {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Unknown,
    })?;
    set_cache(operation.query, response_text.clone()).await;

    Ok(response_text)
}
