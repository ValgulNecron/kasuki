use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::{StaffGettingError, StudioGettingError};

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

impl StudioWrapper {
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
        serde_json::from_str(&resp)
            .map_err(|e| {
            Error(StudioGettingError(format!(
                "Error getting the studio with id {}. {}",
                id, e
            )))
        })
    }

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
        serde_json::from_str(&resp)
            .map_err(|e| {
            Error(StaffGettingError(format!(
                "Error getting the studio with name {}. {}",
                search, e
            )))
        })
    }
}
