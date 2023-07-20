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