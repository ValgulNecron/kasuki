use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_management::web_request_error::WebRequestError;
use crate::error_management::web_request_error::WebRequestError::NotFound;

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

impl PageWrapper {
    pub async fn new_anime_page(number: i64) -> Result<PageWrapper, WebRequestError> {
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
        let res = serde_json::from_str(&res)
            .map_err(|e| NotFound(format!("Error getting the media with id {}. {}", number, e)))?;
        Ok(res)
    }

    pub async fn new_manga_page(number: i64) -> Result<PageWrapper, WebRequestError> {
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

        let res = serde_json::from_str(&res)
            .map_err(|e| NotFound(format!("Error getting the media with id {}. {}", number, e)))?;
        Ok(res)
    }
}
