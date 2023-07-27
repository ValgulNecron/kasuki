use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Title {
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub media: Vec<Media>,
}

#[derive(Debug, Deserialize)]
pub struct PageWrapper {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Deserialize)]
pub struct PageData {
    pub data: PageWrapper,
}

#[derive(Debug, Deserialize)]
pub struct CoverImage {
    #[serde(rename = "extraLarge")]
    pub extra_large: String,
}

impl PageWrapper {

}