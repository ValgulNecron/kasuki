use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InfoLocalisedText {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangLocalisedText {
    pub title: String,
    pub description: String,
    pub error_perm: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PingLocalisedText {
    pub title: String,
    pub description_part_1: String,
    pub description_part_2: String,
    pub description_part_3: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLocalisedText {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InProgressLocalisedText {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranscriptLocalisedText {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TranslationLocalisedText {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnimeLocalisedText {
    pub desc_title: String,
    pub desc_part_1: String,
    pub desc_part_2: String,
    pub desc_part_3: String,
    pub desc_part_4: String,
    pub desc_part_5: String,
    pub desc_part_6: String,
    pub desc_part_7: String,
    pub fields_name_1: String,
    pub fields_name_2: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CharacterLocalisedText {
    pub age: String,
    pub gender: String,
    pub date_of_birth: String,
    pub favourite: String,
    pub desc: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalisedText {
    pub full_name: String,
    pub user_pref: String,
    pub role: String,
    pub format: String,
    pub source: String,
    pub start_date: String,
    pub end_date: String,
    pub fields_name_1: String,
    pub fields_name_2: String,
    pub desc_title: String,
}