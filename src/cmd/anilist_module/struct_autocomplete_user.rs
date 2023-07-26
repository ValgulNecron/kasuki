use serde::Deserialize;
use serde_json::json;
use serenity::json::Value;
use crate::cmd::anilist_module::struct_autocomplete::AutocompleteOption;
use crate::cmd::general_module::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct AutocompleteUser {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UserPage {
    pub users: Option<Vec<Option<AutocompleteUser>>>,
}

#[derive(Debug, Deserialize)]
pub struct UserPageData {
    #[serde(rename = "Page")]
    pub page: UserPage,
}

#[derive(Debug, Deserialize)]
pub struct UserPageWrapper {
    pub data: UserPageData,
}

impl UserPageWrapper{
    pub async fn new_autocomplete_user(search: &Value, count: i32) -> UserPageWrapper {
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
            "count": 8,
        }});

        let res = make_request_anilist(json, true).await;
        let data: UserPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub fn get_choice(&self) -> Value {
         if let Some(users) = &self.data.page.users {
             let suggestions: Vec<AutocompleteOption> = users
                 .iter()
                 .filter_map(|item| {
                     if let Some(item) = item {
                         Some(AutocompleteOption {
                             name: item.name.clone(),
                             value: item.id.to_string(),
                         })
                     } else {
                         None
                     }
                 })
                 .collect();
             let choices = json!(suggestions);
             choices
         } else {
             let choices = json!("Error");
             choices
         }
    }
}