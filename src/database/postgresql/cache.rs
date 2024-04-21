use chrono::Utc;
use serde_json::Value;

use crate::database::postgresql::pool::get_postgresql_pool;
use crate::database_struct::cache_stats::CacheStats;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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
