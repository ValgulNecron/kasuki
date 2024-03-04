use serde::Deserialize;
use serde_json::json;
use tracing::log::trace;

use crate::common::make_anilist_request::make_request_anilist;
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

impl UserPageWrapper {
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
