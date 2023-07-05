use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InfoLocalisedText {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangLocalisedText {
    pub title: String,
    pub description: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PingLocalisedText {
    pub title: String,
    pub description_part_1: String,
    pub description_part_2: String,
    pub description_part_3: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLocalisedText {
    pub title: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InProgressLocalisedText {
    pub title: String,
    pub description: String
}