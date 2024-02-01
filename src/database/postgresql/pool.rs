use crate::error_enum::AppError;
use crate::error_enum::AppError::NotACommandError;
use crate::error_enum::NotACommandError::NotACommandCreatingPoolError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn get_postgresql_pool() -> Result<Pool<Postgres>, AppError> {
    let pool_url = std::env::var("DATABASE_URL").expect("database url");
    PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pool_url.as_str())
        .map_err(|e| {
            NotACommandError(NotACommandCreatingPoolError(format!(
                "Failed to create the pool. {}",
                e
            )))
        })
}
