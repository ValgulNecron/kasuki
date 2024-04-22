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
