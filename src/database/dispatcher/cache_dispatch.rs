use std::env;

use serde_json::Value;

use crate::database::postgresql::cache::{
    get_database_cache_postgresql, get_database_random_cache_postgresql,
    set_database_cache_postgresql, set_database_random_cache_postgres,
};
use crate::database::sqlite::cache::{
    get_database_cache_sqlite, get_database_random_cache_sqlite, set_database_cache_sqlite,
    set_database_random_cache_sqlite,
};
use crate::database_struct::cache_stats::CacheStats;
use crate::error_management::error_enum::AppError;

/// Retrieves a random cache entry from the database.
///
/// This function takes a string representing the type of the random cache entry as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to retrieve the random cache entry.
///
/// # Arguments
///
/// * `random_type` - A string that represents the type of the random cache entry.
///
/// # Returns
///
/// * A Result that is either an Option variant containing the CacheStats if the operation was successful, or an Err variant with an AppError.
pub async fn get_database_random_cache(random_type: &str) -> Result<Option<CacheStats>, AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_database_random_cache_sqlite(random_type).await
    } else if db_type == *"postgresql" {
        get_database_random_cache_postgresql(random_type).await
    } else {
        get_database_random_cache_sqlite(random_type).await
    }
}

/// Sets a random cache entry in the database.
///
/// This function takes a string representing the type of the random cache entry, a string representing the cached response,
/// an i64 representing the current time, and an i64 representing the previous page as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the random cache entry.
///
/// # Arguments
///
/// * `random_type` - A string that represents the type of the random cache entry.
/// * `cached_response` - A string that represents the cached response.
/// * `now` - An i64 that represents the current time.
/// * `previous_page` - An i64 that represents the previous page.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_database_random_cache(
    random_type: &str,
    cached_response: &str,
    now: i64,
    previous_page: i64,
) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_database_random_cache_sqlite(random_type, cached_response, now, previous_page).await
    } else if db_type == *"postgresql" {
        set_database_random_cache_postgres(random_type, cached_response, now, previous_page).await
    } else {
        set_database_random_cache_sqlite(random_type, cached_response, now, previous_page).await
    }
}

/// Retrieves a cache entry from the database.
///
/// This function takes a Value representing the JSON of the cache entry as a parameter.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to retrieve the cache entry.
///
/// # Arguments
///
/// * `json` - A Value that represents the JSON of the cache entry.
///
/// # Returns
///
/// * A Result that is either a tuple containing the Option variants of the cache entry if the operation was successful, or an Err variant with an AppError.
pub async fn get_database_cache(
    json: Value,
) -> Result<(Option<String>, Option<String>, Option<i64>), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_database_cache_sqlite(json).await
    } else if db_type == *"postgresql" {
        get_database_cache_postgresql(json).await
    } else {
        get_database_cache_sqlite(json).await
    }
}

/// Sets a cache entry in the database.
///
/// This function takes a Value representing the JSON of the cache entry and a string representing the response as parameters.
/// It retrieves the type of the database from the environment variables and defaults to "sqlite" if it is not set.
/// Depending on the type of the database, it calls the corresponding function to set the cache entry.
///
/// # Arguments
///
/// * `json` - A Value that represents the JSON of the cache entry.
/// * `resp` - A string that represents the response.
///
/// # Returns
///
/// * A Result that is either an empty Ok variant if the operation was successful, or an Err variant with an AppError.
pub async fn set_database_cache(json: Value, resp: String) -> Result<(), AppError> {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_database_cache_sqlite(json, resp).await
    } else if db_type == *"postgresql" {
        set_database_cache_postgresql(json, resp).await
    } else {
        set_database_cache_sqlite(json, resp).await
    }
}
