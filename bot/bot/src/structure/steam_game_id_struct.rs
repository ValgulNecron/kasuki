use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use anyhow::{Context as AnyhowContext, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, trace};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Debug, Deserialize)]
struct AppListResponse {
	applist: AppList,
}

#[derive(Debug, Deserialize)]
struct AppList {
	apps: Vec<App>,
}

#[derive(Debug, Deserialize)]
pub struct App {
	#[serde(rename = "appid")]
	pub app_id: u32,
	pub name: String,
}

pub async fn get_game(apps_data: Arc<RwLock<HashMap<String, u32>>>) -> Result<usize> {
	debug!("Started Steam game data update process");

	let url = "https://api.steampowered.com/ISteamApps/GetAppList/v0002/?format=json";

	let response: AppListResponse = HTTP_CLIENT
		.get(url)
		.send()
		.await
		.context("Failed to connect to Steam API")?
		.json()
		.await
		.context("Failed to parse Steam API response")?;

	let apps = response.applist.apps;
	debug!("Deserialized {} Steam apps from API", apps.len());

	let app_map: HashMap<String, u32> = apps
		.into_iter()
		.map(|app| (app.name, app.app_id))
		.collect();

	let new_size = app_map.len();

	let mut write_guard = apps_data.write().await;
	trace!("Acquired write lock on apps cache");
	*write_guard = app_map;
	write_guard.shrink_to_fit();
	drop(write_guard);

	debug!("Updated Steam game cache: {} entries", new_size);
	Ok(new_size)
}
