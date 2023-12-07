use crate::common::make_anilist_request::make_request_anilist;
use crate::error_enum::AppError;
use crate::error_enum::AppError::MediaGettingError;
use serde::Deserialize;
use serde_json::json;
use serenity::futures::TryFutureExt;

#[derive(Debug, Deserialize)]
pub struct MediaWrapper {
    pub data: MediaData,
}

#[derive(Debug, Deserialize)]
pub struct MediaData {
    #[serde(rename = "Media")]
    pub media: Media,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: i64,
    pub description: Option<String>,
    pub title: Title,
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "isAdult")]
    pub is_adult: bool,
    #[serde(rename = "startDate")]
    pub start_date: StartEndDate,
    #[serde(rename = "endDate")]
    pub end_date: StartEndDate,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
    pub status: Option<String>,
    pub season: Option<String>,
    #[serde(rename = "isLicensed")]
    pub is_licensed: bool,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    pub genres: Vec<Option<String>>,
    pub tags: Vec<Tag>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub favourites: Option<i32>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<String>,
    pub staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartEndDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    pub id: Option<u32>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: Option<u32>,
    pub name: Name,
}

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: Option<String>,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

impl MediaWrapper {
    pub async fn new_anime_by_id(search: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5) {
		Media (id: $search, type: ANIME){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_anime_by_search(search: &String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5) {
		Media (search: $search, type: ANIME){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_manga_by_id(search: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_manga_by_search(search: String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format_not: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";
        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_ln_by_id(search: String) -> Result<MediaWrapper, AppError> {
        let query_id: &str = "
    query ($search: Int, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (id: $search, type: MANGA, format: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_id, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }

    pub async fn new_ln_by_search(search: String) -> Result<MediaWrapper, AppError> {
        let query_string: &str = "
    query ($search: String, $limit: Int = 5, $format: MediaFormat = NOVEL) {
		Media (search: $search, type: MANGA, format: $format){
    id
      description
    title{
      romaji
      english
    }
    type
    format
    source
    isAdult
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    chapters
    volumes
    status
    season
    isLicensed
    coverImage {
      extraLarge
    }
    bannerImage
    genres
    tags {
      name
    }
    averageScore
    meanScore
    popularity
    favourites
    siteUrl
    staff(perPage: $limit) {
      edges {
        node {
          id
          name {
            full
            userPreferred
          }
        }
        id
        role
      }
    }
  }
}
";

        let json = json!({"query": query_string, "variables": {"search": search}});
        let resp = make_request_anilist(json, false).await;
        // Get json
        serde_json::from_str(&resp)
            .map_err(|_| MediaGettingError(String::from("Error getting this media.")))
    }
}
