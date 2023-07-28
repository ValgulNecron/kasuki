use serde::Deserialize;

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

impl MediaWrapper {}
