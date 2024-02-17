use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use crate::error_management::database_error::DatabaseError;
use crate::error_management::error_enum::NotACommandError::CreatingPoolError;

pub async fn get_postgresql_pool() -> Result<Pool<Postgres>, DatabaseError> {
    let pool_url = std::env::var("DATABASE_URL").map_err(|e| {
        CreatingPoolError(format!("Failed to get the url from environment: {}", e))
    });
    PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pool_url.as_str())
        .map_err(|e| CreatingPoolError(format!("Failed to get the postgres pool: {}", e)))
}
