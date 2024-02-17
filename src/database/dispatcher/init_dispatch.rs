use std::env;

use crate::database::postgresql::init::init_postgres;
use crate::database::sqlite::init::init_sqlite;
use crate::error_management::database_error::DatabaseError;

pub async fn init_sql_database() -> Result<(), DatabaseError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        init_sqlite().await
    } else if db_type == *"postgresql" {
        init_postgres().await
    } else {
        init_sqlite().await
    }
}
