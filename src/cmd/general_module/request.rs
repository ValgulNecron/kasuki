use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

use crate::cmd::general_module::pool::get_pool;

pub async fn make_request_anilist(json: Value, always_update: bool) -> String {
    if always_update {
        do_request(json, always_update).await
    } else {
        get_cache(json.clone()).await
    }
}

async fn get_cache(json: Value) -> String {
    let database_url = "./cache.db";
    let pool = get_pool(database_url).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_cache (
            json TEXT PRIMARY KEY,
            response TEXT NOT NULL,
            last_updated INTEGER NOT NULL
        )",
    )
        .execute(&pool)
        .await
        .unwrap();

    let row: (Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT json, response, last_updated FROM request_cache WHERE json = ?",
    )
        .bind(json.clone())
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None));
    let (json_resp, response, last_updated): (Option<String>, Option<String>, Option<i64>) = row;

    return if json_resp.is_none() || response.is_none() || last_updated.is_none() {
        do_request(json.clone(), false).await
    } else {
        let updated_at = last_updated.unwrap();
        let duration_since_updated = Utc::now().timestamp() - updated_at;
        if duration_since_updated < 3 * (24 * 60 * 60) {
            response.unwrap()
        } else {
            do_request(json.clone(), false).await
        }
    };
}

async fn add_cache(json: Value, resp: String) -> bool {
    let database_url = "./cache.db";
    let pool = get_pool(database_url).await;
    let now = Utc::now().timestamp();
    sqlx::query("INSERT OR REPLACE INTO request_cache (json, response, last_updated) VALUES (?, ?, ?)")
        .bind(json.clone())
        .bind(resp.clone())
        .bind(now)
        .execute(&pool)
        .await.unwrap();

    return true;
}

async fn do_request(json: Value, always_update: bool) -> String {
    let client = Client::new();
    let res = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.clone().to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    let resp = res.unwrap();
    if !always_update {
        add_cache(json.clone(), resp.clone()).await;
    }
    resp
}