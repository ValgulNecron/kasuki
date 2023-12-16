use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;

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

impl StaffPageWrapper {
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
            "count": AUTOCOMPLETE_COUNT,
        }});

        let res = make_request_anilist(json, true).await;
        let data: StaffPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }
}
