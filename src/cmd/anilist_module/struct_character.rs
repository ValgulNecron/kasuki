use serde::Deserialize;

#[derive(Deserialize)]
pub struct CharacterData {
    pub data: CharacterWrapper,
}

#[derive(Deserialize)]
pub struct CharacterWrapper {
    #[serde(rename = "Character")]
    pub character: Character,
}

#[derive(Deserialize)]
pub struct Character {
    pub id: u32,
    pub name: Name,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    pub description: String,
    pub gender: String,
    pub age: String,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: DateOfBirth,
    pub image: Image,
    pub favourites: u32,
    #[serde(rename = "modNotes")]
    pub mod_notes: Option<String>,
}

#[derive(Deserialize)]
pub struct Name {
    pub full: String,
    pub native: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: String,
}

#[derive(Deserialize)]
pub struct DateOfBirth {
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Deserialize)]
pub struct Image {
    pub large: String,
}
