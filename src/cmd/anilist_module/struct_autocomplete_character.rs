use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AutocompleteName {
    pub full: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutocompleteCharacter {
    pub id: u32,
    pub name: Option<AutocompleteName>,
}

#[derive(Debug, Deserialize)]
pub struct CharacterPage {
    pub characters: Option<Vec<Option<AutocompleteCharacter>>>,
}

#[derive(Debug, Deserialize)]
pub struct CharacterPageData {
    #[serde(rename = "Page")]
    pub page: CharacterPage,
}

#[derive(Debug, Deserialize)]
pub struct CharacterPageWrapper {
    pub data: CharacterPageData,
}
