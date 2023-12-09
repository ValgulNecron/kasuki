use crate::common::make_anilist_request::make_request_anilist;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

use crate::error_enum::AppError;
use crate::error_enum::AppError::NoMediaDifferedError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: Option<i64>,
    #[serde(rename = "timeUntilAiring")]
    pub time_until_airing: Option<i64>,
    pub episode: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MinimalAnime {
    pub id: i32,
    pub title: Option<Title>,
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<CoverImage>,
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
    pub async fn new_minimal_anime_by_id(search: String) -> Result<MinimalAnimeWrapper, AppError> {
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
        let data = serde_json::from_str(&resp)
            .map_err(|_| NoMediaDifferedError(String::from("No media")))?;
        Ok(data)
    }

    pub async fn new_minimal_anime_by_id_no_error(id: i32) -> MinimalAnimeWrapper {
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
        let json = json!({"query": query, "variables": {"name": id}});
        let resp = make_request_anilist(json, true).await;
        serde_json::from_str(&resp).unwrap()
    }

    pub async fn new_minimal_anime_by_search(
        search: String,
    ) -> Result<MinimalAnimeWrapper, AppError> {
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
        let data = serde_json::from_str(&resp)
            .map_err(|_| NoMediaDifferedError(String::from("No media")))?;
        Ok(data)
    }
}

#[derive(Debug, FromRow, Clone)]
pub struct ActivityData {
    pub anime_id: Option<String>,
    pub timestamp: Option<String>,
    pub server_id: Option<String>,
    pub webhook: Option<String>,
    pub episode: Option<String>,
    pub name: Option<String>,
    pub delays: Option<i32>,
}
