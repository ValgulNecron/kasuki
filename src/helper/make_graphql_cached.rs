use cynic::{GraphQlResponse, Operation, QueryFragment, QueryVariables};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::cache::manage::cache_dispatch::{get_anilist_cache, set_anilist_cache};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn make_request_anilist<
    'a,
    T: QueryFragment,
    S: QueryVariables + Serialize,
    U: for<'de> Deserialize<'de>,
>(
    operation: Operation<T, S>,
    always_update: bool,
    cache_type: String,
) -> Result<GraphQlResponse<U>, AppError> {
    if !always_update {
        do_request(operation, cache_type).await
    } else {
        let return_data: GraphQlResponse<U> = check_cache(operation, cache_type).await?;
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
    cache_type: String,
) -> Result<GraphQlResponse<U>, AppError> {
    let cache = get_anilist_cache(operation.query.clone(), cache_type.clone()).await;
    match cache {
        Some(data) => get_type(data),
        None => do_request(operation, cache_type).await,
    }
}

async fn do_request<
    T: QueryFragment,
    S: QueryVariables + Serialize,
    U: for<'de> Deserialize<'de>,
>(
    operation: Operation<T, S>,
    cache_type: String,
) -> Result<GraphQlResponse<U>, AppError> {
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
    set_anilist_cache(operation.query, response_text.clone(), cache_type).await;
    get_type(response_text)
}

fn get_type<U: for<'de> Deserialize<'de>>(value: String) -> Result<GraphQlResponse<U>, AppError> {
    serde_json::from_str::<GraphQlResponse<U>>(&value).map_err(|e| AppError {
        message: format!("Error deserializing studio data {}", e),
        error_type: ErrorType::WebRequest,
        error_response_type: ErrorResponseType::Message,
    })
}
