use std::collections::HashMap;
use std::sync::Arc;

use crate::config::DbConfig;
use crate::constant::LANG_MAP;
use crate::error_management::error_dispatch;
use crate::helper::get_guild_lang::get_guild_language;
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use rust_fuzzy_search::fuzzy_search_sorted;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::sync::RwLock;
use tracing::trace;

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

pub struct Webm {
    #[serde(rename = "480°")]
    pub _480: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct Mp4 {
    #[serde(rename = "480°")]
    pub _480: Option<String>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]

pub struct Recommendations {
    pub total: Option<u32>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]

pub struct ReleaseDate {
    pub coming_soon: bool,
    pub date: Option<String>,
}

impl SteamGameWrapper {
    pub async fn new_steam_game_by_id(
        appid: u128,
        guild_id: String,
        db_config: DbConfig,
    ) -> Result<SteamGameWrapper> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; rv:44.0) Gecko/20100101 Firefox/44.0")
            .build()
            .context("Failed to build reqwest client")?;

        let lang = get_guild_language(guild_id, db_config).await;

        let local_lang = LANG_MAP.clone();

        let full_lang = *local_lang
            .get(lang.to_lowercase().as_str())
            .unwrap_or(&"english");

        let url = format!(
            "https://store.steampowered.com/api/appdetails/?cc={}&l={}&appids={}",
            lang, full_lang, appid
        );

        trace!("{}", url);

        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to send request")?;

        let mut text = response
            .text()
            .await
            .context("Failed to get response text")?;

        let re = Regex::new(r#""required_age":"(\d+)""#).expect("Failed to create regex");

        if let Some(cap) = re.captures(&text) {
            if let Some(number) = cap.get(1) {
                let number_str = number.as_str();

                let number: u32 = number_str.parse().expect("Not a number!");

                let base = format!("\"required_age\":\"{}\"", number);

                let new = format!("\"required_age\":{}", number);

                text = text.replace(&base, &new);

                trace!("{}", number)
            }
        }

        let game_wrapper: HashMap<String, SteamGameWrapper> =
            serde_json::from_str(text.as_str()).context("Failed to parse response text")?;

        let game = game_wrapper
            .get(&appid.to_string())
            .context("Failed to get game")?;

        Ok(game.clone())
    }

    pub async fn new_steam_game_by_search(
        search: &str,
        guild_id: String,
        apps: Arc<RwLock<HashMap<String, u128>>>,
        db_config: DbConfig,
    ) -> Result<SteamGameWrapper> {
        let guard = apps.read().await;

        let choices: Vec<(&String, &u128)> = guard.iter().collect();

        let choices: Vec<&str> = choices.into_iter().map(|(s, _)| s.as_str()).collect();

        let results: Vec<(&str, f32)> = fuzzy_search_sorted(search, &choices);

        let mut appid = &0u128;

        if results.is_empty() {
            return Err(anyhow!("No game found".to_string()));
        }

        for (name, _) in results {
            if appid == &0u128 {
                appid = match guard.get(name) {
                    Some(appid) => appid,
                    None => {
                        return Err(anyhow!("No game found".to_string()));
                    }
                }
            }

            if search.to_lowercase() == name.to_lowercase() {
                appid = match guard.get(name) {
                    Some(appid) => appid,
                    None => {
                        return Err(anyhow!("No game found".to_string()));
                    }
                };

                break;
            }
        }

        SteamGameWrapper::new_steam_game_by_id(*appid, guild_id, db_config).await
    }
}
