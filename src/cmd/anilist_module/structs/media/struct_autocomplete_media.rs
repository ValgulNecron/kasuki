use serde::Deserialize;
use serde_json::{json, Value};
use serenity::client::Context;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;

use crate::cmd::anilist_module::structs::struct_autocomplete::AutocompleteOption;
use crate::cmd::general_module::function::request::make_request_anilist;

#[derive(Debug, Deserialize)]
pub struct AutocompleteTitle {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AutocompleteMedia {
    pub id: u32,
    pub title: Option<AutocompleteTitle>,
}

#[derive(Debug, Deserialize)]
pub struct MediaPage {
    pub media: Option<Vec<Option<AutocompleteMedia>>>,
}

#[derive(Debug, Deserialize)]
pub struct MediaPageData {
    #[serde(rename = "Page")]
    pub page: MediaPage,
}

#[derive(Debug, Deserialize)]
pub struct MediaPageWrapper {
    pub data: MediaPageData,
}

impl MediaPageWrapper {
    pub async fn new_autocomplete_anime(
        search: &Value,
        count: u32,
        search_type: &str,
    ) -> MediaPageWrapper {
        let query_str = "query ($search: String, $type: MediaType, $count: Int) {
  Page(perPage: $count) {
    media(search: $search, type: $type) {
      id
      title {
        romaji
        english
      }
    }
  }
}";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "type": search_type,
            "count": count,
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub async fn new_autocomplete_manga(
        search: &Value,
        count: u32,
        search_type: &str,
        format: &str,
    ) -> MediaPageWrapper {
        let query_str =
            "query($search: String, $type: MediaType, $count: Int, $format: MediaFormat) {
          Page(perPage: $count) {
		    media(search: $search, type: $type, format_not: $format) {
		      id
		      title {
		        romaji
		        english
		      }
			}
		  }
		}";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "type": search_type,
            "count": count,
            "format": format
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub async fn new_autocomplete_ln(
        search: &Value,
        count: u32,
        search_type: &str,
        format: &str,
    ) -> MediaPageWrapper {
        let query_str =
            "query($search: String, $type: MediaType, $count: Int, $format: MediaFormat) {
          Page(perPage: $count) {
		    media(search: $search, type: $type, format: $format) {
		      id
		      title {
		        romaji
		        english
		      }
			}
		  }
		}";
        let json = json!({"query": query_str, "variables": {
            "search": search,
            "type": search_type,
            "count": count,
            "format": format
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub fn get_choices(&self) -> Value {
        if let Some(media) = &self.data.page.media {
            let suggestions: Vec<AutocompleteOption> = media
                .iter()
                .filter_map(|item| {
                    if let Some(item) = item {
                        Some(AutocompleteOption {
                            name: match &item.title {
                                Some(name) => {
                                    let english = name.english.clone();
                                    let romaji = name.romaji.clone();
                                    english.unwrap_or(romaji)
                                }
                                None => String::default(),
                            },
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

pub async fn autocomplete(ctx: Context, command: AutocompleteInteraction) {
    let search = &command.data.options.first().unwrap().value;
    if let Some(search) = search {
        let data = MediaPageWrapper::new_autocomplete_anime(search, 8, "ANIME").await;
        let choices = data.get_choices();
        // doesn't matter if it errors
        _ = command
            .create_autocomplete_response(ctx.http, |response| {
                response.set_choices(choices.clone())
            })
            .await;
    }
}
