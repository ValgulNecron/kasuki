use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn get_postgresql_pool() -> Result<Pool<Postgres>, AppError> {
    let pool_url = std::env::var("DATABASE_URL").map_err(|e| {
        AppError::new(
            format!("Failed to get the url from environment: {}", e),
            ErrorType::Database,
            ErrorResponseType::Unknown,
        )
    })?;
    PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pool_url.as_str())
        .map_err(|e| {
            AppError::new(
                format!("Failed to get the postgres pool: {}", e),
                ErrorType::Database,
                ErrorResponseType::Unknown,
            )
        })
}
