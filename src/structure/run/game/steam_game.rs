use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use regex::Regex;
use rust_fuzzy_search::fuzzy_search_sorted;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::sync::RwLock;
use tracing::trace;

use crate::constant::LANG_MAP;
use crate::helper::error_management::error_enum::UnknownResponseError;
use crate::helper::get_guild_lang::get_guild_language;

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct SteamGameWrapper {
    pub success: bool,
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
    /// `new_steam_game_by_id` is an asynchronous function that creates a new `SteamGameWrapper` by the given app id.
    /// It takes an `appid` and `guild_id` as parameters.
    /// `appid` is a u128, and `guild_id` is a String.
    /// It returns a Result which is either a `SteamGameWrapper` or an `AppError`.
    ///
    /// # Arguments
    ///
    /// * `appid` - A u128 that represents the app id.
    /// * `guild_id` - A String that represents the guild id.
    ///
    /// # Returns
    ///
    /// * `Result<SteamGameWrapper, AppError>` - A Result type which is either a `SteamGameWrapper` or an `AppError`.
    ///
    /// # Errors
    ///
    /// This function will return an `AppError` if it encounters any issues while building the client, making the HTTP request, getting the text data, or parsing the JSON.
    pub async fn new_steam_game_by_id(
        appid: u128,
        guild_id: String,
        db_type: String,
    ) -> Result<SteamGameWrapper, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; rv:44.0) Gecko/20100101 Firefox/44.0")
            .build()
            .map_err(|e| {
                UnknownResponseError::WebRequest(format!(
                    "Error when building the client. {:#?}",
                    e.to_string()
                ))
            })?;
        let lang = get_guild_language(guild_id, db_type).await;
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
            .map_err(|e| UnknownResponseError::WebRequest(format!("{:#?}", e)))?;
        let mut text = response
            .text()
            .await
            .map_err(|e| UnknownResponseError::WebRequest(format!("{:#?}", e)))?;

        let re = match Regex::new(r#""required_age":"(\d+)""#) {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(UnknownResponseError::WebRequest(format!(
                    "{:#?}",
                    e
                ))));
            }
        };

        if let Some(cap) = re.captures(&text) {
            if let Some(number) = cap.get(1) {
                let number_str = number.as_str();
                let number: u32 = number_str.parse().expect("Not a number!");
                let base = format!("\"required_age\":\"{}\"", number);
                let new = format!("\"required_age\":{}", number);
                text = text.replace(&base, &new);
                trace!("{}", number) // Output: 18
            }
        }

        let game_wrapper: HashMap<String, SteamGameWrapper> =
            serde_json::from_str(text.as_str())
                .map_err(|e| UnknownResponseError::Json(format!("{:#?}", e)))?;
        match game_wrapper.get(&appid.to_string()) {
            Some(game) => Ok(game.clone()),
            None => Err(Box::new(UnknownResponseError::Json(
                "Game not found".to_string(),
            ))),
        }
    }

    /// `new_steam_game_by_search` is an asynchronous function that creates a new `SteamGameWrapper` by searching for the given string.
    /// It takes a `search` and `guild_id` as parameters.
    /// `search` is a reference to a str, and `guild_id` is a String.
    /// It returns a Result which is either a `SteamGameWrapper` or an `AppError`.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a str that represents the search string.
    /// * `guild_id` - A String that represents the guild id.
    ///
    /// # Returns
    ///
    /// * `Result<SteamGameWrapper, AppError>` - A Result type which is either a `SteamGameWrapper` or an `AppError`.
    ///
    /// # Errors
    ///
    /// This function will return an `AppError` if it encounters any issues while searching for the game.
    pub async fn new_steam_game_by_search(
        search: &str,
        guild_id: String,
        db_type: String,
        apps: Arc<RwLock<HashMap<String, u128>>>,
    ) -> Result<SteamGameWrapper, Box<dyn Error>> {
        let guard = apps.read().await;
        let choices: Vec<(&String, &u128)> = guard.iter().collect();

        let choices: Vec<&str> = choices.into_iter().map(|(s, _)| s.as_str()).collect();
        let results: Vec<(&str, f32)> = fuzzy_search_sorted(search, &choices);

        let mut appid = &0u128;
        if results.is_empty() {
            return Err(Box::new(UnknownResponseError::Option(
                "No game found".to_string(),
            )));
        }
        for (name, _) in results {
            if appid == &0u128 {
                appid = match guard.get(name) {
                    Some(appid) => appid,
                    None => {
                        return Err(Box::new(UnknownResponseError::Option(
                            "No game found".to_string(),
                        )));
                    }
                }
            }

            if search.to_lowercase() == name.to_lowercase() {
                appid = match guard.get(name) {
                    Some(appid) => appid,
                    None => {
                        return Err(Box::new(UnknownResponseError::Option(
                            "No game found".to_string(),
                        )));
                    }
                };
                break;
            }
        }

        SteamGameWrapper::new_steam_game_by_id(*appid, guild_id, db_type).await
    }
}
