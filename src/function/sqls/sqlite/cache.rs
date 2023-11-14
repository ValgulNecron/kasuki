use crate::constant::CACHE_SQLITE_DB;
use crate::function::sqls::sqlite::pool::get_sqlite_pool;

pub async fn get_random_cache_sqlite(
    random_type: &str,
) -> (Option<String>, Option<i64>, Option<i64>) {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await;
    let row: (Option<String>, Option<i64>, Option<i64>) =
        sqlx::query_as("SELECT response, last_updated, last_page FROM cache_stats WHERE key = ?")
            .bind(random_type)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None, None));

    row
}
