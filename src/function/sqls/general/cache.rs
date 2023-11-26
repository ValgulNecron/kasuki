use std::env;

use serde_json::Value;

use crate::function::sqls::sqlite::cache::{
    get_database_cache_sqlite, get_database_random_cache_sqlite, set_database_cache_sqlite,
    set_database_random_cache_sqlite,
};

pub async fn get_database_random_cache(
    random_type: &str,
) -> (Option<String>, Option<i64>, Option<i64>) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_database_random_cache_sqlite(random_type).await
    } else if db_type == *"postgresql" {
        (None, None, None)
    } else {
        get_database_random_cache_sqlite(random_type).await
    }
}

pub async fn set_database_random_cache(
    random_type: &str,
    cached_response: &str,
    now: i64,
    previous_page: i64,
) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_database_random_cache_sqlite(random_type, cached_response, now, previous_page).await
    } else if db_type == *"postgresql" {
    } else {
        set_database_random_cache_sqlite(random_type, cached_response, now, previous_page).await
    }
}

pub async fn get_database_cache(json: Value) -> (Option<String>, Option<String>, Option<i64>) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        get_database_cache_sqlite(json).await
    } else if db_type == *"postgresql" {
        (None, None, None)
    } else {
        get_database_cache_sqlite(json).await
    }
}

pub async fn set_database_cache(json: Value, resp: String) {
    let db_type = env::var("DB_TYPE").unwrap_or("sqlite".to_string());
    if db_type == *"sqlite" {
        set_database_cache_sqlite(json, resp).await
    } else if db_type == *"postgresql" {
    } else {
        set_database_cache_sqlite(json, resp).await
    }
}
