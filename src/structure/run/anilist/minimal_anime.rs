use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use tracing::log::trace;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::make_anilist_cached_request::make_request_anilist;

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

/// `MinimalAnimeWrapper` is an implementation block for the `MinimalAnimeWrapper` struct.
impl MinimalAnimeWrapper {
    /// `get_date` is a function that gets the date.
    /// It takes a `date` as a parameter.
    /// `date` is a reference to a `StartEndDate` that represents the start or end date.
    /// It returns a String that represents the date.
    ///
    /// This function first gets the year, the day, and the month from the `date`.
    /// If the year, the day, and the month are all 0, it returns a default date.
    /// Otherwise, it formats the year, the day, and the month into a date string.
    ///
    /// # Arguments
    ///
    /// * `date` - A reference to a `StartEndDate` that represents the start or end date.
    ///
    /// # Returns
    ///
    /// * `String` - A String that represents the date.
    pub async fn new_minimal_anime_by_id(id: String) -> Result<MinimalAnimeWrapper, AppError> {
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
        let json = json!({"query": query, "variables": {"name": id}});
        let resp = make_request_anilist(json, true).await;
        trace!("{:?}", resp);
        // Get json
        let data = serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the media with id {}. {}", id, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })?;
        Ok(data)
    }

    /// `new_minimal_anime_by_search` is an asynchronous function that creates a new minimal anime by search.
    /// It takes a `search` as a parameter.
    /// `search` is a String that represents the search query.
    /// It returns a `Result` that contains a `MinimalAnimeWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MinimalAnimeWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<MinimalAnimeWrapper, AppError>` - A Result that contains a `MinimalAnimeWrapper` or an `AppError`.
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
        let data = serde_json::from_str(&resp).map_err(|e| AppError {
            message: format!("Error getting the media with name {}. {}", search, e),
            error_type: ErrorType::WebRequest,
            error_response_type: ErrorResponseType::Message,
        })?;
        Ok(data)
    }
}


