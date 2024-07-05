use std::collections::HashMap;
use std::sync::Arc;
// Import necessary libraries and modules
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error};

// App is a struct that represents a Steam app
#[derive(Debug, Deserialize, Clone)]
pub struct App {
    // app_id is the id of the app
    #[serde(rename = "appid")]
    pub app_id: u128,
    // name is the name of the app
    pub name: String,
}

/// `get_game` is an asynchronous function that gets the list of Steam apps and stores them in the APPS constant.
/// It does not take any parameters and does not return a value.
///
/// # Errors
///
/// This function will log an error and return early if it encounters any issues while making the HTTP request, parsing the response body, or deserializing the JSON.
pub async fn get_game(apps_data: Arc<RwLock<HashMap<String, u128>>>) {
    // Log the start of the process
    debug!("Started the process");
    // Define the URL for the Steam API
    let url = "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?format=json";
    // Make the HTTP request
    let response = match reqwest::get(url).await {
        Err(e) => {
            // Log the error and return early if the HTTP request fails
            error!("Error: {}", e);
            return;
        }
        Ok(response) => response,
    };

    // Get the response body
    let body = match response.text().await {
        Err(e) => {
            // Log the error and return early if getting the response body fails
            error!("Error: {}", e);
            return;
        }
        Ok(body) => body,
    };

    // Parse the response body as JSON
    let json: Value = match serde_json::from_str(&body) {
        Err(e) => {
            // Log the error and return early if parsing the JSON fails
            error!("Error: {}", e);
            return;
        }
        Ok(json) => json,
    };

    // Deserialize the JSON into a vector of App structs
    let apps: Vec<App> = match serde_json::from_value(json["applist"]["apps"].clone()) {
        Err(e) => {
            // Log the error and return early if deserializing the JSON fails
            error!("Error: {}", e);
            return;
        }
        Ok(apps) => apps,
    };

    // Clear the APPS constant and insert the new apps
    let mut guard = apps_data.write().await;
    guard.clear();
    guard.shrink_to_fit();
    // Convert the vector of apps into a hashmap
    let app_map: HashMap<String, u128> = apps
        .iter()
        .map(|app| (app.name.clone(), app.app_id))
        .collect();
    *guard = app_map;
    guard.shrink_to_fit();
    debug!("Done getting game")
}
