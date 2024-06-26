use std::collections::HashMap;

use regex::Regex;
use rust_fuzzy_search::fuzzy_search_sorted;
use serde::{Deserialize, Serialize};
use serde_with::formats::PreferOne;
use serde_with::serde_as;
use serde_with::OneOrMany;
use tracing::trace;

use crate::constant::{APPS, LANG_MAP};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
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
    pub detailed_description: Option<String>,
    pub about_the_game: Option<String>,
    pub short_description: Option<String>,
    pub supported_languages: Option<String>,
    pub reviews: Option<String>,
    pub header_image: Option<String>,
    pub capsule_image: Option<String>,
    pub capsule_imagev5: Option<String>,
    pub website: Option<String>,
    #[serde_as(deserialize_as = "OneOrMany<_, PreferOne>")]
    pub pc_requirements: Vec<Requirements>,
    #[serde_as(deserialize_as = "OneOrMany<_, PreferOne>")]
    pub mac_requirements: Vec<Requirements>,
    #[serde_as(deserialize_as = "OneOrMany<_, PreferOne>")]
    pub linux_requirements: Vec<Requirements>,
    pub developers: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub price_overview: Option<PriceOverview>,
    pub packages: Option<Vec<u32>>,
    pub package_groups: Option<Vec<PackageGroup>>,
    pub platforms: Option<Platforms>,
    pub categories: Option<Vec<Category>>,
    pub screenshots: Option<Vec<Screenshot>>,
    pub movies: Option<Vec<Movie>>,
    pub recommendations: Option<Recommendations>,
    pub release_date: Option<ReleaseDate>,
    pub support_info: Option<SupportInfo>,
    pub background: Option<String>,
    pub background_raw: Option<String>,
    pub content_descriptors: Option<ContentDescriptors>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Requirements {
    pub minimum: Option<String>,
    pub recommended: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct PriceOverview {
    pub currency: Option<String>,
    pub initial: Option<u32>,
    #[serde(rename = "final")]
    pub final_price: Option<u32>,
    pub discount_percent: Option<u32>,
    pub initial_formatted: Option<String>,
    pub final_formatted: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct PackageGroup {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub selection_text: Option<String>,
    pub save_text: Option<String>,
    pub display_type: Option<u32>,
    pub is_recurring_subscription: Option<String>,
    pub subs: Option<Vec<Sub>>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Sub {
    pub packageid: Option<u32>,
    pub percent_savings_text: Option<String>,
    pub percent_savings: Option<u32>,
    pub option_text: Option<String>,
    pub option_description: Option<String>,
    pub can_get_free_license: Option<String>,
    pub is_free_license: Option<bool>,
    pub price_in_cents_with_discount: Option<u32>,
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
    pub id: Option<u32>,
    pub description: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Screenshot {
    pub id: Option<u32>,
    pub path_thumbnail: Option<String>,
    pub path_full: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Movie {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub thumbnail: Option<String>,
    pub webm: Option<Webm>,
    pub mp4: Option<Mp4>,
    pub highlight: Option<bool>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Webm {
    #[serde(rename = "480°")]
    pub _480: Option<String>,
    pub max: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Mp4 {
    #[serde(rename = "480°")]
    pub _480: Option<String>,
    pub max: Option<String>,
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

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct SupportInfo {
    pub url: Option<String>,
    pub email: Option<String>,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct ContentDescriptors {
    pub ids: Vec<u32>,
    pub notes: Option<String>,
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
    ) -> Result<SteamGameWrapper, AppError> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; rv:44.0) Gecko/20100101 Firefox/44.0")
            .build()
            .map_err(|e| {
                AppError::new(
                    format!("Failed to build the client. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Unknown,
                )
            })?;
        let lang = get_guild_language(guild_id, db_type).await;
        let local_lang = LANG_MAP;
        let full_lang = *local_lang
            .get(lang.to_lowercase().as_str())
            .unwrap_or(&"english");
        let url = format!(
            "https://store.steampowered.com/api/appdetails/?cc={}&l={}&appids={}",
            lang, full_lang, appid
        );

        trace!("{}", url);

        let response = client.get(&url).send().await.map_err(|e| {
            AppError::new(
                format!("Error when making the request. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Unknown,
            )
        })?;
        let mut text = response.text().await.map_err(|e| {
            AppError::new(
                format!("Failed to get the text data. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Unknown,
            )
        })?;

        let re = Regex::new(r#""required_age":"(\d+)""#).unwrap();

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

        let game_wrapper: HashMap<String, SteamGameWrapper> = serde_json::from_str(text.as_str())
            .map_err(|e| {
            AppError::new(
                format!("Failed to parse as json. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Unknown,
            )
        })?;

        Ok(game_wrapper.get(&appid.to_string()).unwrap().clone())
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
    ) -> Result<SteamGameWrapper, AppError> {
        let choices: Vec<(&String, &u128)>;
        unsafe { choices = APPS.iter().collect() }

        let choices: Vec<&str> = choices.into_iter().map(|(s, _)| s.as_str()).collect();
        let results: Vec<(&str, f32)> = fuzzy_search_sorted(search, &choices);

        let mut appid = &0u128;
        unsafe {
            if results.is_empty() {
                return Err(AppError::new(
                    "Game not found.".to_string(),
                    ErrorType::WebRequest,
                    ErrorResponseType::Unknown,
                ));
            }
            for (name, _) in results {
                if appid == &0u128 {
                    appid = APPS.get(name).unwrap()
                }

                if search.to_lowercase() == name.to_lowercase() {
                    appid = APPS.get(name).unwrap();
                    break;
                }
            }
        }

        SteamGameWrapper::new_steam_game_by_id(*appid, guild_id, db_type).await
    }
}
