use serde_json::Value;

use reqwest::Client;

pub async fn make_request(json: Value) -> String{
    let client =  Client::new();
    let res = client.post("https://graphql.anilist.co/")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;
    return res.unwrap()
}