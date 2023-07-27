use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AutocompleteTitle {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutocompleteMedia {
    pub id: u32,
    pub title: Option<AutocompleteTitle>,
}

#[derive(Debug, Deserialize)]
pub struct MediaPage {
    pub media: Option<Vec<Option<AutocompleteMedia>>>,
}

#[derive(Debug, Deserialize)]
pub struct MediaPageData {
    #[serde(rename = "Page")]
    pub page: MediaPage,
}

#[derive(Debug, Deserialize)]
pub struct MediaPageWrapper {
    pub data: MediaPageData,
}

impl MediaPageWrapper {

}