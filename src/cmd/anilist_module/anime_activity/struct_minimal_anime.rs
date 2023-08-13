use crate::cmd::general_module::lang_struct::AddActivityLocalisedText;
use crate::cmd::general_module::request::make_request_anilist;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: i64,
    #[serde(rename = "timeUntilAiring")]
    pub time_until_airing: i64,
    pub episode: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Title {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnime {
    pub id: i32,
    pub title: Title,
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnimeWrapper {
    pub data: MinimalAnimeData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnimeData {
    #[serde(rename = "Media")]
    pub media: MinimalAnime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

impl MinimalAnimeWrapper {
    pub async fn new_minimal_anime_by_id(
        localised_text: AddActivityLocalisedText,
        search: String,
    ) -> Result<MinimalAnimeWrapper, String> {
        let query = "
            query ($name: Int) {
              Media(type: ANIME, id: $name) {
                id
               coverImage {
                  extraLarge
                }
                title {
                  romaji
                  english
                }
                nextAiringEpisode {
                  airingAt
                  timeUntilAiring
                  episode
                }
              }
            }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        // Get json
        let data: MinimalAnimeWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub async fn new_minimal_anime_by_id_no_error(search: String) -> MinimalAnimeWrapper {
        let query = "
            query ($name: Int) {
              Media(type: ANIME, search: $name) {
                id
                coverImage {
                  extraLarge
                }
                title {
                  romaji
                  english
                }
                nextAiringEpisode {
                  airingAt
                  timeUntilAiring
                  episode
                }
              }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        let data: MinimalAnimeWrapper = serde_json::from_str(&resp).unwrap();
        data
    }

    pub async fn new_minimal_anime_by_search(
        localised_text: AddActivityLocalisedText,
        search: String,
    ) -> Result<MinimalAnimeWrapper, String> {
        let query = "
            query ($name: String) {
              Media(type: ANIME, search: $name) {
                id
                coverImage {
                  extraLarge
                }
                title {
                  romaji
                  english
                }
                nextAiringEpisode {
                  airingAt
                  timeUntilAiring
                  episode
                }
              }
            }
        ";
        let json = json!({"query": query, "variables": {"name": search}});
        let resp = make_request_anilist(json, true).await;
        // Get json
        let data: MinimalAnimeWrapper = match serde_json::from_str(&resp) {
            Ok(data) => data,
            Err(error) => {
                println!("Error: {}", error);
                return Err(localised_text.error_no_media.clone());
            }
        };
        return Ok(data);
    }

    pub fn get_id(&self) -> i32 {
        self.data.media.id
    }

    pub fn get_timestamp(&self) -> i64 {
        let media = self.data.media.next_airing_episode.clone().unwrap();
        media.airing_at.clone()
    }

    pub fn get_name(&self) -> String {
        format!("{} / {}", self.get_en_title(), self.get_rj_title())
    }

    pub fn get_en_title(&self) -> String {
        self.data
            .media
            .title
            .english
            .clone()
            .unwrap_or_else(|| "NA".to_string())
    }

    pub fn get_rj_title(&self) -> String {
        self.data.media.title.romaji.clone()
    }

    pub fn get_episode(&self) -> i32 {
        self.data.media.next_airing_episode.clone().unwrap().episode
    }

    pub fn get_image(&self) -> String {
        self.data.media.cover_image.extra_large.clone().unwrap()
    }
}
