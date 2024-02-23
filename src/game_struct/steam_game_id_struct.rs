use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error};

use crate::constant::APPS;

#[derive(Debug, Deserialize, Clone)]
pub struct App {
    #[serde(rename = "appid")]
    pub app_id: u128,
    pub name: String,
}

pub async fn get_game() {
    debug!("Started the process");
    let url = "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?format=json";
    let response = match reqwest::get(url).await {
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
        Ok(response) => response,
    };

    let body = match response.text().await {
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
        Ok(body) => body,
    };

    let json: Value = match serde_json::from_str(&body) {
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
        Ok(json) => json,
    };

    let apps: Vec<App> = match serde_json::from_value(json["applist"]["apps"].clone()) {
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
        Ok(apps) => apps,
    };

    unsafe {
        APPS.clear();
        for app in apps {
            APPS.insert(app.name, app.app_id);
        }
    }
}
