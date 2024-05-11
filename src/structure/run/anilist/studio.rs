use serde::Deserialize;
use serde_json::json;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::make_anilist_cached_request::make_request_anilist;

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    pub romaji: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaNode {
    pub title: Title,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    pub nodes: Vec<MediaNode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Studio {
    pub id: u32,
    pub name: String,
    #[serde(rename = "isAnimationStudio")]
    pub is_animation_studio: bool,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub favourites: u32,
    pub media: Media,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioData {
    #[serde(rename = "Studio")]
    pub studio: Studio,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioWrapper {
    pub data: StudioData,
}

/// `StudioWrapper` is an implementation block for the `StudioWrapper` struct.
impl StudioWrapper {
    /// `new_studio_by_id` is an asynchronous function that creates a new studio by ID.
    /// It takes an `id` as a parameter.
    /// `id` is a 32-bit integer that represents the ID of the studio.
    /// It returns a `Result` that contains a `StudioWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes an `id` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `id` variable is set to the `id` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StudioWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `id` - A 32-bit integer that represents the ID of the studio.
    ///
    /// # Returns
    ///
    /// * `Result<StudioWrapper, AppError>` - A Result that contains a `StudioWrapper` or an `AppError`.
    pub async fn new_studio_by_id(id: i32) -> Result<StudioWrapper, AppError> {
        let query_id: &str = "\
        query ($name: Int, $limit: Int = 15) {
          Studio(id: $name) {
            id
            name
            isAnimationStudio
            siteUrl
            favourites
            media(perPage: $limit, sort: START_DATE_DESC) {
              nodes {
                title{
                  romaji
                  userPreferred
                }
                siteUrl
              }
            }
          }
        }
        ";
        let json = json!({"query": query_id, "variables": {"name": id}});
        let resp = make_request_anilist(json, false).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the studio with id {}. {}", id, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
    }

    /// `new_studio_by_search` is an asynchronous function that creates a new studio by search.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `Result` that contains a `StudioWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `search` variable is set to the `search` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `StudioWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `Result<StudioWrapper, AppError>` - A Result that contains a `StudioWrapper` or an `AppError`.
    pub async fn new_studio_by_search(search: &String) -> Result<StudioWrapper, AppError> {
        let query_string: &str = "
        query ($name: String, $limit: Int = 5) {
          Studio(search: $name) {
            id
            name
            isAnimationStudio
            siteUrl
            favourites
            media(perPage: $limit, sort: START_DATE_DESC) {
              nodes {
                title{
                  romaji
                  userPreferred
                }
                siteUrl
              }
            }
          }
        }
        ";
        let json = json!({"query": query_string, "variables": {"name": search}});
        let resp = make_request_anilist(json, false).await;
        serde_json::from_str(&resp).map_err(|e| {
            AppError::new(
                format!("Error getting the studio with name {}. {}", search, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })
    }
}
