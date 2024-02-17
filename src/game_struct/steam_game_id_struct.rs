use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error};

use crate::constant::{APPS, TIME_BETWEEN_GAME_UPDATE};

#[derive(Debug, Deserialize, Clone)]
pub struct App {
    #[serde(rename = "appid")]
    pub app_id: u128,
    pub name: String,
}

pub async fn get_game() {
    loop {
        debug!("Started the process");
        let url = "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?format=json";
        let response = match reqwest::get(url).await {
            Err(e) => {
                error!("Error: {}", e);
                continue
            }
            Ok(response) => response,
        };

        let body = match response.text().await {
            Err(e) => {
                error!("Error: {}", e);
                continue
            }
            Ok(body) => body,
        };
        let json: Value = if let Err(e) = serde_json::from_str(&body) {
            error!("Error: {}", e);
            continue
        } else if let Ok(json) = serde_json::from_str(&body) {
            json
        } else {
            unreachable!()
        };

        let apps: Vec<App> = if let Err(e) = serde_json::from_value(json["applist"]["apps"].clone()) {
            error!("Error: {}", e);
            continue
        } else if let Ok(apps) = serde_json::from_value(json["applist"]["apps"].clone()) {
            apps
        } else {
            unreachable!()
        };

        unsafe {
            APPS.clear();
            for app in apps {
                APPS.insert(app.name, app.app_id);
            }
        }

        tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_GAME_UPDATE)).await;
    }
}
