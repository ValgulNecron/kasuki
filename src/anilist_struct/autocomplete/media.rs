use crate::common::make_anilist_request::make_request_anilist;
use serde::Deserialize;
use serde_json::{json, Value};
use serenity::all::{AutocompleteOption, CommandOptionType};

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
        search: &String,
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
}
