use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::error_enum::AppError;
use crate::sqls::postgresql::pool::get_postgresql_pool;
use crate::sqls::sqlite::pool::get_sqlite_pool;

pub async fn init_sqlite() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    init_postgres_cache(&pool).await?;
    pool.close().await;
    let pool = get_postgresql_pool().await?;
    init_postgres_data(&pool).await?;
    pool.close().await;
    Ok(())
}

