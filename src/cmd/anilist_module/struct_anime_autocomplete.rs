use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Title {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub id: u32,
    pub title: Option<Title>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub media: Option<Vec<Option<Media>>>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "Page")]
    pub page: Page,
}

#[derive(Debug, Deserialize)]
pub struct Root {
    pub data: Data,
}
