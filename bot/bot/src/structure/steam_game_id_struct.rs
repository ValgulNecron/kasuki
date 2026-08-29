use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use anyhow::{Context as AnyhowContext, Result, anyhow};
use arc_swap::ArcSwap;
use reqwest::Client;
use serde::Deserialize;
use shared::cache::CacheInterface;
use tracing::{debug, info, warn};

use crate::structure::steam_game_index::SteamGameIndex;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

const STEAM_CACHE_KEY: &str = "steam_app_list";

/// Response shape of `IStoreService/GetAppList/v1`.
#[derive(Debug, Deserialize)]
struct AppListResponse {
	response: AppListPage,
}

#[derive(Debug, Deserialize, Default)]
struct AppListPage {
	#[serde(default)]
	apps: Vec<App>,
	/// Absent once the final page has been returned.
	#[serde(default)]
	have_more_results: bool,
	/// Cursor to pass as `last_appid` for the next page.
	#[serde(default)]
	last_appid: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct App {
	#[serde(rename = "appid")]
	pub app_id: u32,
	#[serde(default)]
	pub name: String,
}

/// Valve caps a single page at 50 000 entries.
const STEAM_PAGE_SIZE: u32 = 50_000;
/// Safety bound so a misbehaving cursor can never loop forever.
const STEAM_MAX_PAGES: u32 = 40;

pub async fn get_game(
	apps_data: Arc<ArcSwap<SteamGameIndex>>, steam_cache: Arc<CacheInterface>, api_key: &str,
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

	let mut app_map: HashMap<String, u32> = HashMap::new();
	let mut last_appid: Option<u32> = None;

	for page in 0..STEAM_MAX_PAGES {
		let mut request = HTTP_CLIENT
			.get("https://api.steampowered.com/IStoreService/GetAppList/v1/")
			.query(&[("key", api_key)])
			.query(&[("include_games", "true")])
			.query(&[("max_results", STEAM_PAGE_SIZE.to_string())]);

		if let Some(cursor) = last_appid {
			request = request.query(&[("last_appid", cursor.to_string())]);
		}

		let response: AppListResponse = request
			.send()
			.await
			.context("Failed to connect to Steam API")?
			.error_for_status()
			.context("Steam API returned an error status")?
			.json()
			.await
			.context("Failed to parse Steam API response")?;

		let page_data = response.response;
		let received = page_data.apps.len();
		debug!("Steam app list page {}: {} entries", page, received);

		app_map.extend(page_data.apps.into_iter().map(|app| (app.name, app.app_id)));

		// Stop when Valve says there is nothing left, or when it gives us no cursor to
		// advance with — continuing without a moving cursor would refetch the same page.
		if !page_data.have_more_results {
			break;
		}
		match page_data.last_appid {
			Some(cursor) if Some(cursor) != last_appid => last_appid = Some(cursor),
			_ => {
				warn!("Steam reported more results but returned no new cursor; stopping early");
				break;
			},
		}

		if page + 1 == STEAM_MAX_PAGES {
			warn!(
				"Reached the {}-page safety limit for the Steam app list; results may be partial",
				STEAM_MAX_PAGES
			);
		}
	}

	let new_size = app_map.len();
	debug!("Deserialized {} Steam apps from API", new_size);

	if new_size == 0 {
		return Err(anyhow!("Steam API returned an empty app list"));
	}

	if let Ok(json) = serde_json::to_string(&app_map) {
		if let Err(e) = steam_cache.write(STEAM_CACHE_KEY.to_string(), json).await {
			warn!("Failed to persist Steam app list to cache: {}", e);
		}
	}

	apps_data.store(Arc::new(SteamGameIndex::from_map(app_map)));

	debug!("Updated Steam game cache: {} entries", new_size);
	Ok(new_size)
}
