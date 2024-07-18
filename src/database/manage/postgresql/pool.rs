use std::error::Error;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

use crate::helper::error_management::error_enum::UnknownResponseError;

/// Retrieves a connection pool to the PostgreSQL database.
///
/// This function performs the following operations in order:
/// 1. Retrieves the `DATABASE_URL` environment variable.
/// 2. Creates a new `PgPoolOptions` instance and sets the maximum number of connections to 20.
/// 3. Connects to the PostgreSQL database using the `DATABASE_URL` and the `PgPoolOptions` instance.
///
/// # Returns
///
/// * A Result that is either a `Pool<Postgres>` if the operation was successful, or an Err variant with an `AppError` if the operation failed. The `AppError` contains a description of the error, the type of the error (`ErrorType::Database`), and the response type of the error (`ErrorResponseType::Unknown`).
pub async fn get_postgresql_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    let pool_url = std::env::var("DATABASE_URL").map_err(|e| {
        UnknownResponseError::Database(format!("Failed to get the postgres pool: {:#?}", e))
    })?;
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(pool_url.as_str())
        .map_err(|e| {
            UnknownResponseError::Database(format!(
                "Failed to connect to the postgres pool: {:#?}",
                e
            ))
        })?;

    Ok(pool)
}
