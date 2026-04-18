use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use anyhow::{Context as AnyhowContext, Result};
use arc_swap::ArcSwap;
use reqwest::Client;
use serde::Deserialize;
use shared::cache::CacheInterface;
use tracing::{debug, info, warn};

use crate::structure::steam_game_index::SteamGameIndex;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

const STEAM_CACHE_KEY: &str = "steam_app_list";

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

pub async fn get_game(
	apps_data: Arc<ArcSwap<SteamGameIndex>>, steam_cache: Arc<CacheInterface>,
) -> Result<usize> {
	debug!("Started Steam game data update process");

	let is_cold_start = apps_data.load().is_empty();
	if is_cold_start {
		if let Ok(Some(cached_json)) = steam_cache.read(STEAM_CACHE_KEY).await {
			let app_map: HashMap<String, u32> = serde_json::from_str(&cached_json)
				.context("Failed to deserialize cached Steam app list")?;
			let size = app_map.len();

			info!(
				"Loaded {} Steam apps from cache (skipping HTTP fetch)",
				size
			);
			apps_data.store(Arc::new(SteamGameIndex::from_map(app_map)));
			return Ok(size);
		}
	}

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

	let app_map: HashMap<String, u32> =
		apps.into_iter().map(|app| (app.name, app.app_id)).collect();

	let new_size = app_map.len();

	if let Ok(json) = serde_json::to_string(&app_map) {
		if let Err(e) = steam_cache.write(STEAM_CACHE_KEY.to_string(), json).await {
			warn!("Failed to persist Steam app list to cache: {}", e);
		}
	}

	apps_data.store(Arc::new(SteamGameIndex::from_map(app_map)));

	debug!("Updated Steam game cache: {} entries", new_size);
	Ok(new_size)
}
