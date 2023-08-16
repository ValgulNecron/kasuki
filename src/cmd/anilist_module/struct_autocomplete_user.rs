use serde::Deserialize;
use serde_json::json;
use serenity::client::Context;
use serenity::json::Value;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;

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

impl UserPageWrapper {
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
            "count": count,
        }});

        let res = make_request_anilist(json, true).await;
        let data: UserPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub fn get_choice(&self) -> Vec<AutocompleteOption> {
        if let Some(users) = &self.data.page.users {
            users
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
                .collect::<Vec<AutocompleteOption>>()
        } else {
            vec![]
        }
    }
}

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = UserPageWrapper::new_autocomplete_user(search, 8).await;
        let choices = data.get_choice();
        // doesn't matter if it errors
        let choices_json = json!(choices);
        _ = command
            .create_autocomplete_response(ctx.http.clone(), |response| {
                response.set_choices(choices_json)
            })
            .await;
    }
}