use serde::Deserialize;
use serde_json::json;
use tracing::log::trace;

use crate::common::make_anilist_request::make_request_anilist;
use crate::constant::AUTOCOMPLETE_COUNT_LIMIT;

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

/// `CharacterPageWrapper` is an implementation block for the `CharacterPageWrapper` struct.
impl CharacterPageWrapper {
    /// `new_autocomplete_character` is an asynchronous function that creates a new autocomplete character.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `CharacterPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` and `count` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter and the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then traces the response and deserializes it into a `CharacterPageWrapper`.
    /// It traces the deserialized data and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `CharacterPageWrapper` - A `CharacterPageWrapper` that represents the page wrapper of the character.
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
            "count": AUTOCOMPLETE_COUNT_LIMIT,
        }});
        let res = make_request_anilist(json, true).await;
        trace!("{:#?}", res);
        let data: CharacterPageWrapper = serde_json::from_str(&res).unwrap();
        trace!("{:#?}", data);
        data
    }
}
