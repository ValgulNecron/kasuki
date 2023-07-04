use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InfoLocalisedText{
    pub title: String,
    pub description: String,
    pub footer:String,
    pub button_see_on_github: String,
    pub button_official_website : String,
    pub button_official_discord : String,
    pub button_add_the_bot : String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LangLocalisedText{
    pub title: String,
    pub description: String,
    pub footer:String
}