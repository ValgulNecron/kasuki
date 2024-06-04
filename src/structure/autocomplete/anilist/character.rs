use serde::Deserialize;
use serde_json::json;
use tracing::log::trace;

use crate::constant::AUTOCOMPLETE_COUNT_LIMIT;
use crate::helper::make_anilist_cached_request::make_request_anilist;

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct CharacterAutocompleteVariables<'a> {
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "CharacterAutocompleteVariables")]
pub struct CharacterAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "CharacterAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $search)]
    pub characters: Option<Vec<Option<Character>>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Character {
    pub id: i32,
    pub name: Option<CharacterName>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CharacterName {
    pub full: Option<String>,
    pub user_preferred: Option<String>,
    pub native: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
    pub first: Option<String>,
}
