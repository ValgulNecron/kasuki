use crate::constant::{CACHE_SQLITE_DB, DATA_SQLITE_DB};
use crate::error_enum::AppError;
use crate::sqls::sqlite::pool::get_sqlite_pool;

pub async fn init_sqlite() -> Result<(), AppError> {
    let pool = get_sqlite_pool(CACHE_SQLITE_DB).await?;
    crate::sqls::sqlite::init::init_sqlite_cache(&pool).await?;
    pool.close().await;
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    crate::sqls::sqlite::init::init_sqlite_data(&pool).await?;
    pool.close().await;
    Ok(())
}
