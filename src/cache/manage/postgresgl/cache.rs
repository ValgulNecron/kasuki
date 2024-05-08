use chrono::Utc;
use serde_json::Value;

use crate::cache::cache_struct::cache_stats::CacheStats;
use crate::database::postgresql::pool::get_postgresql_pool;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Retrieves a random cache entry from the PostgreSQL database.
///
/// This function takes a string parameter `random_type` which is used to query the database.
/// It fetches the `response`, `last_updated`, and `last_page` fields from the `CACHE.cache_stats` table where the `key` matches `random_type`.
///
/// # Parameters
///
/// * `random_type`: A string slice that represents the key of the cache entry.
///
/// # Returns
///
/// * A Result that is either an Option containing CacheStats if the operation was successful and the cache entry exists, or None if the cache entry does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_database_random_cache_postgresql(
    random_type: &str,
) -> Result<Option<CacheStats>, AppError> {
    let pool = get_postgresql_pool().await?;

    let row: Option<CacheStats> = sqlx::query_as(
        "SELECT response, last_updated, last_page FROM CACHE.cache_stats WHERE key = $1",
    )
    .bind(random_type)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    pool.close().await;
    Ok(row)
}

/// Sets a random cache entry in the PostgreSQL database.
///
/// This function takes four parameters: `random_type`, `cached_response`, `now`, and `previous_page`. It inserts these values into the `CACHE.cache_stats` table. If a cache entry with the same `key` already exists, it updates the existing entry with the new values.
///
/// # Parameters
///
/// * `random_type`: A string slice that represents the key of the cache entry.
/// * `cached_response`: A string slice that represents the response to be cached.
/// * `now`: An i64 that represents the current timestamp.
/// * `previous_page`: An i64 that represents the previous page number.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_database_random_cache_postgres(
    random_type: &str,
    cached_response: &str,
    now: i64,
    previous_page: i64,
) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    sqlx::query("INSERT INTO CACHE.cache_stats (key, response, last_updated, last_page) VALUES ($1, $2, $3, $4)\
     ON CONFLICT (key) DO UPDATE SET response = EXCLUDED.response, last_updated = EXCLUDED.last_updated, last_page = EXCLUDED.last_page")
        .bind(random_type)
        .bind(cached_response)
        .bind(now)
        .bind(previous_page)
        .execute(&pool)
        .await
        .map_err(|e|
            AppError::new(
                format!("Failed to insert into the table. {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            ))?;
    pool.close().await;
    Ok(())
}

/// Retrieves a cache entry from the PostgreSQL database.
///
/// This function takes a `json` parameter of type `Value` which is used to query the database.
/// It fetches the `json`, `response`, and `last_updated` fields from the `CACHE.request_cache` table where the `json` matches the input `json`.
///
/// # Parameters
///
/// * `json`: A `Value` that represents the json of the cache entry.
///
/// # Returns
///
/// * A Result that is either a tuple containing Option<String>, Option<String>, Option<i64> if the operation was successful and the cache entry exists, or (None, None, None) if the cache entry does not exist. Returns an Err variant with an AppError if the operation failed.
pub async fn get_database_cache_postgresql(
    json: Value,
) -> Result<(Option<String>, Option<String>, Option<i64>), AppError> {
    let pool = get_postgresql_pool().await?;

    let row: (Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT json, response, last_updated FROM CACHE.request_cache WHERE json = $1",
    )
    .bind(json.clone())
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None, None));

    pool.close().await;
    Ok(row)
}

/// Sets a cache entry in the PostgreSQL database.
///
/// This function takes two parameters: `json` and `resp`. It inserts these values into the `CACHE.request_cache` table. If a cache entry with the same `json` already exists, it updates the existing entry with the new values.
///
/// # Parameters
///
/// * `json`: A `Value` that represents the json of the cache entry.
/// * `resp`: A `String` that represents the response to be cached.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError if the operation failed.
pub async fn set_database_cache_postgresql(json: Value, resp: String) -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO CACHE.request_cache (json, response, last_updated) VALUES ($1, $2, $3) ON CONFLICT (json) DO UPDATE SET response = EXCLUDED.response, last_updated = EXCLUDED.last_updated",
    )
        .bind(json.clone())
        .bind(resp.clone())
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e|
            AppError::new(
                format!("Failed to insert into the table. {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            ))?;
    pool.close().await;
    Ok(())
}
