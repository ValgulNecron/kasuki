use sqlx::{Pool, Postgres};
use crate::error_enum::AppError;
use crate::error_enum::AppError::CreatingPoolError;

pub async fn get_postgresql_pool() -> Result<Pool<Postgres>, AppError> {
    let pool_url = std::env::var("DATABASE_URL").expect("database url");
    Postgres::connect(pool_url)
        .await
        .map_err(|_| CreatingPoolError(String::from("Failed to create the pool.")))
}