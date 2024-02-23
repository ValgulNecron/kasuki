use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

use crate::constant::TIME_BETWEEN_CACHE_UPDATE;
use crate::database::dispatcher::cache_dispatch::{get_database_cache, set_database_cache};

pub async fn make_request_anilist(json: Value, always_update: bool) -> String {
    if always_update {
        do_request(json, always_update).await
    } else {
        get_cache(json.clone()).await
    }
}

async fn get_cache(json: Value) -> String {
    let (json_resp, response, last_updated): (Option<String>, Option<String>, Option<i64>) =
        get_database_cache(json.clone())
            .await
            .unwrap_or((None, None, None));

    if json_resp.is_none() || response.is_none() || last_updated.is_none() {
        do_request(json.clone(), false).await
    } else {
        let updated_at = last_updated.unwrap();
        let duration_since_updated = Utc::now().timestamp() - updated_at;
        if duration_since_updated < (TIME_BETWEEN_CACHE_UPDATE) as i64 {
            response.unwrap()
        } else {
            do_request(json.clone(), false).await
        }
    }
}

async fn add_cache(json: Value, resp: String) -> bool {
    set_database_cache(json, resp).await.unwrap_or(());

    true
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
