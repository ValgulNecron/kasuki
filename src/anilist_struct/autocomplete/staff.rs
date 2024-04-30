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
pub struct AutocompleteStaff {
    pub id: u32,
    pub name: AutocompleteName,
}

#[derive(Debug, Deserialize)]
pub struct StaffPage {
    pub staff: Option<Vec<Option<AutocompleteStaff>>>,
}

#[derive(Debug, Deserialize)]
pub struct StaffPageData {
    #[serde(rename = "Page")]
    pub page: StaffPage,
}

#[derive(Debug, Deserialize)]
pub struct StaffPageWrapper {
    pub data: StaffPageData,
}

/// `StaffPageWrapper` is an implementation block for the `StaffPageWrapper` struct.
impl StaffPageWrapper {
    /// `new_autocomplete_staff` is an asynchronous function that creates a new autocomplete staff.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `StaffPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search` and `count` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter and the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then traces the response and deserializes it into a `StaffPageWrapper`.
    /// It traces the deserialized data and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `StaffPageWrapper` - A `StaffPageWrapper` that represents the page wrapper of the staff.
    pub async fn new_autocomplete_staff(search: &String) -> StaffPageWrapper {
        let query_str = "query ($search: String, $count: Int) {
          Page(perPage: $count) {
            staff(search: $search) {
              id
              name {
                full
                userPreferred
              }
            }
          }
        }";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "count": AUTOCOMPLETE_COUNT_LIMIT,
        }});

        let res = make_request_anilist(json, true).await;
        trace!("{:#?}", res);
        let data: StaffPageWrapper = serde_json::from_str(&res).unwrap();
        trace!("{:#?}", data);
        data
    }
}