use chrono::Utc;
use serde_json::Value;

use crate::constant::CACHE_SQLITE_DB;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::error_management::database_error::DatabaseError;
use crate::error_management::database_error::DatabaseError::Insert;

/// Retrieves the cache statistics for a given random type from a SQLite database using a connection pool.
/// The cache statistics include the response, last updated timestamp, and last page.
/// If the cache statistics for the given random type are found in the database, they are returned.
/// If no cache statistics are found, `None` is returned for each value.
///
/// # Arguments
///
/// * `random_type` - The random type to retrieve cache statistics for.
///
/// # Returns
///
/// A tuple containing the response, last updated timestamp, and last page of the cache statistics.
pub async fn get_database_random_cache_sqlite(
    random_type: &str,
) -> Result<(Option<String>, Option<i64>, Option<i64>), DatabaseError> {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    let row: (Option<String>, Option<i64>, Option<i64>) =
        sqlx::query_as("SELECT response, last_updated, last_page FROM cache_stats WHERE key = ?")
            .bind(random_type)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None, None));
    pool.close().await;
    Ok(row)
}

/// Sets the database random cache for SQLite.
///
/// This function inserts or replaces a record in the `cache_stats` table of the SQLite database with the given parameters.
///
/// # Arguments
///
/// * `random_type` - The key identifying the random type.
/// * `cached_response` - The cached response to be stored.
/// * `now` - The timestamp for the last update.
/// * `previous_page` - The value of the last page.
///
pub async fn set_database_random_cache_sqlite(
    random_type: &str,
    cached_response: &str,
    now: i64,
    previous_page: i64,
) -> Result<(), DatabaseError> {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
        .bind(random_type)
        .bind(cached_response)
        .bind(now)
        .bind(previous_page)
        .execute(&pool)
        .await
        .map_err(|e| Insert(format!("Failed to insert into the table. {}", e)))?;
    pool.close().await;
    Ok(())
}

/// Retrieves data from a SQLite database cache based on the provided JSON.
///
/// # Arguments
///
/// * `json` - The JSON data to search for in the cache.
///
/// # Returns
///
/// A tuple containing the JSON, response, and last_updated values from the cache.
/// If no matching JSON is found in the cache, the returned tuple will contain `None` values.
///
pub async fn get_database_cache_sqlite(
    json: Value,
) -> Result<(Option<String>, Option<String>, Option<i64>), DatabaseError> {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    let row: (Option<String>, Option<String>, Option<i64>) =
        sqlx::query_as("SELECT json, response, last_updated FROM request_cache WHERE json = ?")
            .bind(json.clone())
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None, None));
    pool.close().await;
    Ok(row)
}

/// Sets the database cache for SQLite.
///
/// # Arguments
///
/// * `json` - The JSON value to be stored in the cache.
/// * `resp` - The response string to be stored in the cache.
///
pub async fn set_database_cache_sqlite(json: Value, resp: String) -> Result<(), DatabaseError> {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT OR REPLACE INTO request_cache (json, response, last_updated) VALUES (?, ?, ?)",
    )
        .bind(json.clone())
        .bind(resp.clone())
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| Insert(format!("Failed to insert into the table. {}", e)))?;
    pool.close().await;
    Ok(())
}
