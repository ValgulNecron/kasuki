use std::env;

use crate::database::postgresql::init::init_postgres;
use crate::database::sqlite::init::init_sqlite;
use crate::error_management::error_enum::AppError;

pub async fn init_sql_database() -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        init_sqlite().await
    } else if db_type == *"postgresql" {
        init_postgres().await
    } else {
        init_sqlite().await
    }
}
