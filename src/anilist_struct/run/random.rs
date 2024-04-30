use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    pub id: i32,
    pub title: Title,
    #[serde(rename = "meanScore")]
    pub mean_score: i32,
    pub description: String,
    pub tags: Vec<Tag>,
    pub genres: Vec<String>,
    pub format: String,
    pub status: String,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Title {
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageData {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PageWrapper {
    pub data: PageData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: String,
}

/// `PageWrapper` is an implementation block for the `PageWrapper` struct.
impl PageWrapper {
    /// `new_anime_page` is an asynchronous function that creates a new anime page.
    /// It takes a `number` as a parameter.
    /// `number` is a 64-bit integer that represents the page number.
    /// It returns a `Result` that contains a `PageWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `number` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `number` variable is set to the `number` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `PageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `number` - A 64-bit integer that represents the page number.
    ///
    /// # Returns
    ///
    /// * `Result<PageWrapper, AppError>` - A Result that contains a `PageWrapper` or an `AppError`.
    pub async fn new_anime_page(number: i64) -> Result<PageWrapper, AppError> {
        let query = "
                    query($anime_page: Int){
                        Page(page: $anime_page, perPage: 1){
                            media(type: ANIME){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"anime_page": number}});
        let res = make_request_anilist(json, false).await;
        let res = serde_json::from_str(&res).map_err(|e| {
            AppError::new(
                format!("Error getting the media with id {}. {}", number, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })?;
        Ok(res)
    }

    /// `new_manga_page` is an asynchronous function that creates a new manga page.
    /// It takes a `number` as a parameter.
    /// `number` is a 64-bit integer that represents the page number.
    /// It returns a `Result` that contains a `PageWrapper` or an `AppError`.
    ///
    /// This function first defines a GraphQL query string that takes a `number` as a variable.
    /// It then creates a JSON object with the query string and the variable.
    /// The `number` variable is set to the `number` parameter.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `PageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `number` - A 64-bit integer that represents the page number.
    ///
    /// # Returns
    ///
    /// * `Result<PageWrapper, AppError>` - A Result that contains a `PageWrapper` or an `AppError`.
    pub async fn new_manga_page(number: i64) -> Result<PageWrapper, AppError> {
        let query = "
                    query($manga_page: Int){
                        Page(page: $manga_page, perPage: 1){
                            media(type: MANGA){
                            id
                            title {
                                native
                                userPreferred
                            }
                            meanScore
                            description
                            tags {
                                name
                            }
                            genres
                            format
                            status
                            coverImage {
                                extraLarge
                            }
                        }
                    }
                }";

        let json = json!({"query": query, "variables": {"manga_page": number}});
        let res = make_request_anilist(json, false).await;

        let res = serde_json::from_str(&res).map_err(|e| {
            AppError::new(
                format!("Error getting the media with id {}. {}", number, e),
                ErrorType::WebRequest,
                ErrorResponseType::Message,
            )
        })?;
        Ok(res)
    }
}
