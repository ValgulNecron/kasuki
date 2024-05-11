use crate::cache::manage::postgresgl::init::init_postgres;
use crate::cache::manage::sqlite::init::init_sqlite;
use crate::constant::CACHE_TYPE;
use crate::helper::error_management::error_enum::AppError;

/// Initializes the Cache database.
///
/// This function does not take any parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to initialize the database.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn init_sql_database() -> Result<(), AppError> {
    let cache_type = CACHE_TYPE.clone();
    let cache_type = cache_type.as_str();
    if cache_type == "sqlite" {
        init_sqlite().await
    } else if cache_type == "postgresql" {
        init_postgres().await
    } else {
        init_sqlite().await
    }
}
