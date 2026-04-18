use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::constant::LANG_MAP;
use crate::structure::steam_game_index::SteamGameIndex;
use anyhow::{anyhow, Context, Result};
use arc_swap::ArcSwap;
use regex::Regex;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_with::serde_as;
use shared::cache::CacheInterface;
use shared::helper::get_guild_lang::get_guild_language;
use tracing::trace;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	reqwest::Client::builder()
		.user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; rv:44.0) Gecko/20100101 Firefox/44.0")
		.build()
		.expect("Failed to build static reqwest client")
});

static REQUIRED_AGE_RE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r#""required_age":"(\d+)""#).expect("Failed to create regex"));

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct SteamGameWrapper {
	pub data: Data,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct Data {
	#[serde(rename = "type")]
	pub app_type: Option<String>,
	pub name: Option<String>,
	pub steam_appid: Option<u32>,
	pub required_age: Option<u32>,
	pub is_free: Option<bool>,
	pub short_description: Option<String>,
	pub supported_languages: Option<String>,
	pub header_image: Option<String>,
	pub website: Option<String>,
	pub developers: Option<Vec<String>>,
	pub publishers: Option<Vec<String>>,
	pub price_overview: Option<PriceOverview>,
	pub platforms: Option<Platforms>,
	pub categories: Option<Vec<Category>>,
	pub release_date: Option<ReleaseDate>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct PriceOverview {
	pub discount_percent: Option<u32>,
	pub final_formatted: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct Platforms {
	pub windows: Option<bool>,
	pub mac: Option<bool>,
	pub linux: Option<bool>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct Category {
	pub description: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct ReleaseDate {
	pub coming_soon: bool,
	pub date: Option<String>,
}

impl SteamGameWrapper {
	pub async fn new_steam_game_by_id(
		appid: u32, guild_id: String, db_connection: Arc<DatabaseConnection>,
		steam_cache: Arc<CacheInterface>,
	) -> Result<SteamGameWrapper> {
		let lang = get_guild_language(guild_id, db_connection).await;

		let full_lang = *LANG_MAP
			.get(lang.to_lowercase().as_str())
			.unwrap_or(&"english");

		let cache_key = format!("steam_game_{}_{}", appid, lang);

		if let Some(cached) = steam_cache.read(&cache_key).await? {
			let game_wrapper: HashMap<String, SteamGameWrapper> =
				serde_json::from_str(&cached).context("Failed to parse cached response")?;
			if let Some(game) = game_wrapper.get(&appid.to_string()) {
				return Ok(game.clone());
			}
		}

		let url = format!(
			"https://store.steampowered.com/api/appdetails/?cc={}&l={}&appids={}",
			lang, full_lang, appid
		);

		trace!("{}", url);

		let response = HTTP_CLIENT
			.get(&url)
			.send()
			.await
			.context("Failed to send request")?;

		let mut text = response
			.text()
			.await
			.context("Failed to get response text")?;

		if let Some(cap) = REQUIRED_AGE_RE.captures(&text) {
			if let Some(number) = cap.get(1) {
				let number_str = number.as_str();

				if let Ok(number) = number_str.parse::<u32>() {
					let base = format!("\"required_age\":\"{}\"", number);
					let new = format!("\"required_age\":{}", number);
					text = text.replace(&base, &new);
				}
			}
		}

		steam_cache
			.write(cache_key, text.clone())
			.await
			.context("Failed to write to steam cache")?;

		let game_wrapper: HashMap<String, SteamGameWrapper> =
			serde_json::from_str(text.as_str()).context("Failed to parse response text")?;

		let game = game_wrapper
			.get(&appid.to_string())
			.context("Failed to get game")?;

		Ok(game.clone())
	}

	pub async fn new_steam_game_by_search(
		search: &str, guild_id: String, apps: Arc<ArcSwap<SteamGameIndex>>,
		db_connection: Arc<DatabaseConnection>, steam_cache: Arc<CacheInterface>,
	) -> Result<SteamGameWrapper> {
		let index = apps.load();

		if index.is_empty() {
			return Err(anyhow!(
				"Steam game database is still loading, please try again shortly"
			));
		}

		let results = index.search(search, 1);

		let (_, app_id) = results
			.first()
			.ok_or_else(|| anyhow!("No game found matching '{}'", search))?;

		let app_id = *app_id;
		drop(index);

		SteamGameWrapper::new_steam_game_by_id(app_id, guild_id, db_connection, steam_cache).await
	}
}
