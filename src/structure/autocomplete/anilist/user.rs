use serde::Deserialize;
use serde_json::json;
use tracing::log::trace;

use crate::helper::make_anilist_cached_request::make_request_anilist;
use crate::constant::AUTOCOMPLETE_COUNT_LIMIT;

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteUser {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserPage {
    pub users: Option<Vec<Option<AutocompleteUser>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserPageData {
    #[serde(rename = "Page")]
    pub page: UserPage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserPageWrapper {
    pub data: UserPageData,
}

/// `UserPageWrapper` is an implementation block for the `UserPageWrapper` struct.
impl UserPageWrapper {
    /// `new_autocomplete_user` is an asynchronous function that creates a new autocomplete user.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `UserPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` and `count` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter and the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then traces the response and deserializes it into a `UserPageWrapper`.
    /// It traces the deserialized data and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `UserPageWrapper` - A `UserPageWrapper` that represents the page wrapper of the user.
    pub async fn new_autocomplete_user(search: &String) -> UserPageWrapper {
        let query_str = "query ($search: String, $count: Int) {
          Page(perPage: $count) {
            users(search: $search) {
              id
              name
            }
          }
        }";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "count": AUTOCOMPLETE_COUNT_LIMIT,
        }});

        let res = make_request_anilist(json, true).await;
        trace!("{:#?}", res);
        let data: UserPageWrapper = serde_json::from_str(&res).unwrap();
        trace!("{:#?}", data);
        data
    }
}
