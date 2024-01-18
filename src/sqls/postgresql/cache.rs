use crate::error_enum::AppError;
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
