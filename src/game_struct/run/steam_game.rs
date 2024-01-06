use crate::constant::{APPS, LANG_MAP};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{NotAValidGameError, NotAValidUrlError};
use crate::sqls::general::data::get_data_guild_langage;
use fuzzy_match::fuzzy_match;
use serde::{Deserialize, Serialize};
use serde_with::formats::PreferOne;
use serde_with::serde_as;
use serde_with::OneOrMany;
use std::collections::HashMap;
use tracing::trace;

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
    pub steam_appid: Option<u128>,
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
    pub async fn new_steam_game_by_id(
        appid: u128,
        guild_id: String,
    ) -> Result<SteamGameWrapper, AppError> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; WOW64; rv:44.0) Gecko/20100101 Firefox/44.0")
            .build()
            .map_err(|_| NotAValidUrlError(String::from("Bad url")))?;
        let lang = get_data_guild_langage(guild_id)
            .await?
            .0
            .unwrap_or(String::from("en"))
            .to_uppercase();
        let full_lang = LANG_MAP
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
            .map_err(|_| NotAValidUrlError(String::from("Bad url")))?;
        let text = response
            .text()
            .await
            .map_err(|_| NotAValidGameError(String::from("Bad game")))?;
        let game_wrapper: HashMap<String, SteamGameWrapper> =
            serde_json::from_str(text.as_str())
                .map_err(|_| NotAValidGameError(String::from("Bad game")))?;

        Ok(game_wrapper.get(&appid.to_string()).unwrap().clone())
    }

    pub async fn new_steam_game_by_search(
        search: &str,
        guild_id: String,
    ) -> Result<SteamGameWrapper, AppError> {
        let lang = get_data_guild_langage(guild_id)
            .await?
            .0
            .unwrap_or(String::from("en"));
        let full_lang = LANG_MAP.get(lang.as_str()).unwrap_or(&"english");
        let mut choices: Vec<(&String, &u128)> = Vec::new();
        unsafe {
            choices = APPS.iter().collect();
        }

        let choices: Vec<(&str, &u128)> = choices
            .into_iter()
            .map(|(s, id)| (s.as_str(), id))
            .collect();

        let appid: Option<u128>;
        match fuzzy_match(search, choices) {
            Some(app_id) => appid = Some(*app_id),
            None => appid = None,
        }

        let url = format!(
            "https://store.steampowered.com/api/appdetails/?cc={}&l={}&appids={}",
            lang,
            full_lang,
            appid.unwrap_or(1)
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|_| NotAValidUrlError(String::from("Bad url")))?;
        let game_wrapper: HashMap<String, SteamGameWrapper> = response
            .json()
            .await
            .map_err(|_| NotAValidGameError(String::from("Bad game")))?;

        Ok(game_wrapper
            .get(&appid.unwrap_or(1).to_string())
            .unwrap()
            .clone())
    }
}