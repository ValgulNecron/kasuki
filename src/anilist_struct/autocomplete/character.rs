use crate::constant::AUTOCOMPLETE_COUNT;
use serde::Deserialize;
use serde_json::json;
use tracing::log::trace;

use crate::common::make_anilist_request::make_request_anilist;

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteName {
    pub full: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteCharacter {
    pub id: u32,
    pub name: Option<AutocompleteName>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterPage {
    pub characters: Option<Vec<Option<AutocompleteCharacter>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterPageData {
    #[serde(rename = "Page")]
    pub page: CharacterPage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterPageWrapper {
    pub data: CharacterPageData,
}

impl CharacterPageWrapper {
    pub async fn new_autocomplete_character(search: &String) -> CharacterPageWrapper {
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
            "count": AUTOCOMPLETE_COUNT,
        }});
        let res = make_request_anilist(json, true).await;
        trace!("{:#?}", res);
        let data: CharacterPageWrapper = serde_json::from_str(&res).unwrap();
        trace!("{:#?}", data);
        data
    }
}
