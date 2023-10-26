use crate::function::request::request::make_request_anilist;
use crate::structure::anilist::struct_autocomplete::AutocompleteOption;
use serde::Deserialize;
use serde_json::{json, Value};

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
                    item.as_ref().map(|item| AutocompleteOption {
                        name: item
                            .name
                            .user_preferred
                            .as_ref()
                            .unwrap_or(&item.name.full)
                            .to_string(),
                        value: item.id.to_string(),
                    })
                })
                .collect::<Vec<AutocompleteOption>>()
        } else {
            vec![]
        }
    }
}
