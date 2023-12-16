use serde::Deserialize;
use serde_json::json;

use crate::common::make_anilist_request::make_request_anilist;

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteStudio {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioPage {
    pub studios: Option<Vec<Option<AutocompleteStudio>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioPageData {
    #[serde(rename = "Page")]
    pub page: StudioPage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StudioPageWrapper {
    pub data: StudioPageData,
}

impl StudioPageWrapper {
    pub async fn new_autocomplete_staff(search: &String) -> StudioPageWrapper {
        let query_str = "query ($search: String, $count: Int) {
          Page(perPage: $count) {
            studios(search: $search) {
              id
              name
            }
          }
        }";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "count": AUTOCOMPLETE_COUNT,
        }});

        let res = make_request_anilist(json, true).await;
        serde_json::from_str(&res).unwrap()
    }
}
