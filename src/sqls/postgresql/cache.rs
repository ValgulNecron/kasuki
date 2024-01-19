use crate::error_enum::AppError;
use crate::error_enum::AppError::SqlInsertError;
use crate::sqls::postgresql::pool::get_postgresql_pool;

pub async fn get_database_random_cache_postgresql(
    random_type: &str,
) -> Result<(Option<String>, Option<i64>, Option<i64>), AppError> {
    let pool = get_postgresql_pool().await?;

    let row: (Option<String>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT response, last_updated, last_page FROM CACHE.cache_stats WHERE key = $1",
    )
    .bind(random_type)
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None, None));

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
   sqlx::query("INSERT INTO cache_stats (key, response, last_updated, last_page) VALUES ($1, $2, $3, $4) ON CONFLICT (key) DO UPDATE SET response = EXCLUDED.response, last_updated = EXCLUDED.last_updated, last_page = EXCLUDED.last_page")
       .bind(random_type)
       .bind(cached_response)
       .bind(now)
       .bind(previous_page)
       .execute(&pool)
       .await
       .map_err(|_| SqlInsertError(String::from("Failed to insert data.")))?;
   pool.close().await;
   Ok(())
}
