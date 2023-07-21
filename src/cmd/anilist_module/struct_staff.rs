use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub large: String,
}

#[derive(Debug, Deserialize)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub title: Title,
}

#[derive(Debug, Deserialize)]
pub struct StaffMedia {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    #[serde(rename = "roleNotes")]
    pub role_notes: Option<String>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    #[serde(rename = "staffRole")]
    pub staff_role: String,
}

#[derive(Debug, Deserialize)]
pub struct Character {
    pub name: Name,
    pub image: Image,
}

#[derive(Debug, Deserialize)]
pub struct Characters {
    pub nodes: Vec<Character>,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    pub name: Name,
    pub id: i32,
    #[serde(rename = "languageV2")]
    pub language_v2: String,
    pub image: Image,
    pub description: String,
    #[serde(rename = "primaryOccupations")]
    pub primary_occupations: Vec<String>,
    pub gender: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Date,
    #[serde(rename = "dateOfDeath")]
    pub date_of_death: Date,
    pub age: Option<i32>,
    #[serde(rename = "yearsActive")]
    pub years_active: Vec<i32>,
    #[serde(rename = "homeTown")]
    pub home_town: Option<String>,
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "staffMedia")]
    pub staff_media: StaffMedia,
    pub characters: Characters,
}

#[derive(Debug, Deserialize)]
pub struct StaffWrapper {
    #[serde(rename = "Staff")]
    pub staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct StaffData {
    pub data: StaffWrapper,
}
