use crate::common::make_anilist_request::make_request_anilist;
use crate::constant::AUTOCOMPLETE_COUNT;
use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteTitle {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteMedia {
    pub id: u32,
    pub title: Option<AutocompleteTitle>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaPage {
    pub media: Option<Vec<Option<AutocompleteMedia>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaPageData {
    #[serde(rename = "Page")]
    pub page: MediaPage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaPageWrapper {
    pub data: MediaPageData,
}

impl MediaPageWrapper {
    pub async fn new_autocomplete_anime(search: &String) -> MediaPageWrapper {
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
            "type": "ANIME",
            "count": AUTOCOMPLETE_COUNT,
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub async fn new_autocomplete_manga(search: &String) -> MediaPageWrapper {
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
            "type": "MANGA",
            "count": AUTOCOMPLETE_COUNT,
            "format": "NOVEL"
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    pub async fn new_autocomplete_ln(search: &String) -> MediaPageWrapper {
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
            "type": "MANGA",
            "count": AUTOCOMPLETE_COUNT,
            "format": "NOVEL"
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }
}

pub async fn send_auto_complete(
    ctx: Context,
    command: CommandInteraction,
    media: MediaPageWrapper,
) {
    let mut choices = Vec::new();
    for page in media.data.page.media.unwrap() {
        let data = page.unwrap();
        let title_data = data.title.unwrap();
        let english = title_data.english;
        let romaji = title_data.romaji;
        let title = english.unwrap_or(romaji);
        choices.push(AutocompleteChoice::new(title, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = command.create_response(ctx.http, builder).await;
}
