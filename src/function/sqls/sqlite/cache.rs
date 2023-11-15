use crate::constant::CACHE_SQLITE_DB;
use crate::function::sqls::sqlite::pool::get_sqlite_pool;
use chrono::Utc;
use log::error;
use serde_json::Value;

pub async fn get_database_random_cache_sqlite(
    random_type: &str,
) -> (Option<String>, Option<i64>, Option<i64>) {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    let row: (Option<String>, Option<i64>, Option<i64>) =
        sqlx::query_as("SELECT response, last_updated, last_page FROM cache_stats WHERE key = ?")
            .bind(random_type)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None, None));
    pool.close().await;
    row
}

pub async fn set_database_random_cache_sqlite(
    random_type: &str,
    cached_response: &str,
    now: i64,
    previous_page: i64,
) {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    match sqlx::query("INSERT OR REPLACE INTO cache_stats (key, response, last_updated, last_page) VALUES (?, ?, ?, ?)")
        .bind(random_type)
        .bind(cached_response)
        .bind(now)
        .bind(previous_page)
        .execute(&pool)
        .await {
        Ok(_) => {},
        Err(e) => error!("{}", e)
    }
    pool.close().await;
}

pub async fn get_database_cache_sqlite(
    json: Value,
) -> (Option<String>, Option<String>, Option<i64>) {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    let row: (Option<String>, Option<String>, Option<i64>) =
        sqlx::query_as("SELECT json, response, last_updated FROM request_cache WHERE json = ?")
            .bind(json.clone())
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None, None));
    pool.close().await;
    row
}

pub async fn set_database_cache_sqlite(json: Value, resp: String) {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    let now = Utc::now().timestamp();
    match sqlx::query(
        "INSERT OR REPLACE INTO request_cache (json, response, last_updated) VALUES (?, ?, ?)",
    )
    .bind(json.clone())
    .bind(resp.clone())
    .bind(now)
    .execute(&pool)
    .await
    {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
    pool.close().await;
}
