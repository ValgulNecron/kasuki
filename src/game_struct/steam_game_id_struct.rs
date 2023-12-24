use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use tracing::debug;

use crate::constant::{APPS, GAME_UPDATE};

#[derive(Debug, Deserialize, Clone)]
pub struct App {
    #[serde(rename = "appid")]
    pub app_id: u128,
    pub name: String,
}

pub async fn get_game() {
    loop {
        debug!("Started the process");
        let url =
            "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?key=STEAMKEY&format=json";
        let response = reqwest::get(url).await.unwrap();

        let body = response.text().await.unwrap();
        let json: Value = serde_json::from_str(&body).unwrap();

        let mut apps: Vec<App> = serde_json::from_value(json["applist"]["apps"].clone()).unwrap();
        unsafe {
            APPS.clear();
            APPS.append(&mut apps);
        }

        debug!("waiting for {} day(s)", GAME_UPDATE);
        tokio::time::sleep(Duration::from_secs((GAME_UPDATE * 24 * 60 * 60) as u64)).await;
    }
}
