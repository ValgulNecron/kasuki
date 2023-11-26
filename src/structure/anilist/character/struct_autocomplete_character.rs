use serde::Deserialize;
use serde_json::{json, Value};

use crate::function::requests::request::make_request_anilist;
use crate::structure::anilist::struct_autocomplete::AutocompleteOption;

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

impl CharacterPageWrapper {
    pub async fn new_autocomplete_character(search: &Value, count: i32) -> CharacterPageWrapper {
        let query_str = "query ($search: String, $count: Int) {
          Page(perPage: $count) {
            characters(search: $search) {
              id
              name {
                full
              }
            }
          }
        }
        ";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "count": count,
        }});
        let res = make_request_anilist(json, true).await;
        let data: CharacterPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub fn get_choices(&self) -> Value {
        if let Some(character) = &self.data.page.characters {
            let suggestions: Vec<AutocompleteOption> = character
                .iter()
                .filter_map(|item| {
                    item.as_ref().map(|item| AutocompleteOption {
                        name: match &item.name {
                            Some(name) => {
                                let english = name.user_preferred.clone();
                                let romaji = name.full.clone();
                                english.unwrap_or(romaji)
                            }
                            None => String::default(),
                        },
                        value: item.id.to_string(),
                    })
                })
                .collect();
            let choices = json!(suggestions);
            choices
        } else {
            let choices = json!("Error");
            choices
        }
    }
}
