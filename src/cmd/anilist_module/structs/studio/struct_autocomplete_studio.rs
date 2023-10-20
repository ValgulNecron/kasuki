use serde::Deserialize;
use serde_json::{json, Value};

use crate::cmd::anilist_module::structs::struct_autocomplete::AutocompleteOption;
use crate::cmd::general_module::function::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct AutocompleteStudio {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct StudioPage {
    pub studios: Option<Vec<Option<AutocompleteStudio>>>,
}

#[derive(Debug, Deserialize)]
pub struct StudioPageData {
    #[serde(rename = "Page")]
    pub page: StudioPage,
}

#[derive(Debug, Deserialize)]
pub struct StudioPageWrapper {
    pub data: StudioPageData,
}

impl StudioPageWrapper {
    pub async fn new_autocomplete_staff(search: &Value, count: i32) -> StudioPageWrapper {
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
            "count": count,
        }});

        let res = make_request_anilist(json, true).await;
        serde_json::from_str(&res).unwrap()
    }

    pub fn get_choice(&self) -> Vec<AutocompleteOption> {
        if let Some(studios) = &self.data.page.studios {
            studios
                .iter()
                .filter_map(|item| {
                    item.as_ref().map(|item| AutocompleteOption {
                        name: item.name.clone(),
                        value: item.id.to_string(),
                    })
                })
                .collect::<Vec<AutocompleteOption>>()
        } else {
            vec![]
        }
    }
}
