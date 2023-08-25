use serde::Deserialize;
use serde_json::{json, Value};

use crate::cmd::anilist_module::structs::struct_autocomplete::AutocompleteOption;
use crate::cmd::general_module::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct AutocompleteName {
    pub full: String,
    #[serde(rename = "userPreferred")]
    pub user_preferred: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutocompleteStaff {
    pub id: u32,
    pub name: Option<AutocompleteName>,
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
    pub async fn new_autocomplete_staff(search: &Value, count: i32) -> StaffPageWrapper {
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
            "count": count,
        }});

        let res = make_request_anilist(json, true).await;
        let data: StaffPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub fn get_choice(&self) -> Vec<AutocompleteOption> {
        if let Some(users) = &self.data.page.staff {
            users
                .iter()
                .filter_map(|item| {
                    if let Some(item) = item {
                        Some(AutocompleteOption {
                            name: item.name.as_ref().unwrap().full.clone(),
                            value: item.id.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<AutocompleteOption>>()
        } else {
            vec![]
        }
    }
}
