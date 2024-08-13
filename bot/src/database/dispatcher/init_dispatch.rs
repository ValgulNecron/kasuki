use crate::config::BotConfigDetails;
use crate::database::postgresql::init::init_postgres;
use crate::database::sqlite::init::init_sqlite;
use std::error::Error;

/// Initializes the SQL database.
///
/// This function does not take any parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to initialize the database.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn init_sql_database(
    db_type: &str,
    db_config: BotConfigDetails,
) -> Result<(), Box<dyn Error>> {
    if db_type == "sqlite" {
        init_sqlite().await
    } else if db_type == "postgresql" {
        init_postgres(db_config).await
    } else {
        init_sqlite().await
    }
}
