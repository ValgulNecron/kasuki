use serde::Deserialize;
use serde_json::json;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::log::trace;

use crate::common::make_anilist_request::make_request_anilist;
use crate::constant::AUTOCOMPLETE_COUNT_LIMIT;

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

/// `MediaPageWrapper` is an implementation block for the `MediaPageWrapper` struct.
impl MediaPageWrapper {
    /// `new_autocomplete_anime` is an asynchronous function that creates a new autocomplete anime.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `MediaPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search`, `type`, `count`, and `format` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter, the `type` variable is set to "ANIME", the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`, and the `format` variable is not set.
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaPageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `MediaPageWrapper` - A `MediaPageWrapper` that represents the page wrapper of the media.
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
            "count": AUTOCOMPLETE_COUNT_LIMIT,
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }

    // `new_autocomplete_manga` is an asynchronous function that creates a new autocomplete manga.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `MediaPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search`, `type`, `count`, and `format` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter, the `type` variable is set to "MANGA", the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`, and the `format` variable is set to "NOVEL".
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then traces the response and deserializes it into a `MediaPageWrapper`.
    /// It traces the deserialized data and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `MediaPageWrapper` - A `MediaPageWrapper` that represents the page wrapper of the media.
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
            "count": AUTOCOMPLETE_COUNT_LIMIT,
            "format": "NOVEL"
        }});

        let res = make_request_anilist(json, true).await;
        trace!("{:#?}", res);
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        trace!("{:#?}", data);
        data
    }

    /// `new_autocomplete_ln` is an asynchronous function that creates a new autocomplete light novel.
    /// It takes a `search` as a parameter.
    /// `search` is a reference to a String that represents the search query.
    /// It returns a `MediaPageWrapper`.
    ///
    /// This function first defines a GraphQL query string that takes a `search`, `type`, `count`, and `format` as variables.
    /// It then creates a JSON object with the query string and the variables.
    /// The `search` variable is set to the `search` parameter, the `type` variable is set to "MANGA", the `count` variable is set to `AUTOCOMPLETE_COUNT_LIMIT`, and the `format` variable is set to "NOVEL".
    /// It makes a request to AniList with the JSON object and waits for the response.
    /// It then deserializes the response into a `MediaPageWrapper` and returns it.
    ///
    /// # Arguments
    ///
    /// * `search` - A reference to a String that represents the search query.
    ///
    /// # Returns
    ///
    /// * `MediaPageWrapper` - A `MediaPageWrapper` that represents the page wrapper of the media.
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
            "count": AUTOCOMPLETE_COUNT_LIMIT,
            "format": "NOVEL"
        }});

        let res = make_request_anilist(json, true).await;
        let data: MediaPageWrapper = serde_json::from_str(&res).unwrap();
        data
    }
}

/// `send_auto_complete` is an asynchronous function that sends autocomplete choices.
/// It takes `ctx`, `autocomplete_interaction`, and `media` as parameters.
/// `ctx` is a Context that represents the context.
/// `autocomplete_interaction` is a CommandInteraction that represents the command interaction.
/// `media` is a MediaPageWrapper that represents the page wrapper of the media.
///
/// This function first creates an empty vector of choices.
/// It then iterates over the media in the page of the `media`.
/// For each media, it unwraps the media and the title of the media.
/// It gets the English title of the media if it exists, otherwise it gets the Romaji title.
/// It creates a new autocomplete choice with the title and the ID of the media and pushes it to the choices.
/// It creates a new autocomplete response with the choices and sets the choices of the response.
/// It creates a new interaction response with the autocomplete response.
/// It then sends the interaction response using the `autocomplete_interaction`.
///
/// # Arguments
///
/// * `ctx` - A Context that represents the context.
/// * `autocomplete_interaction` - A CommandInteraction that represents the command interaction.
/// * `media` - A MediaPageWrapper that represents the page wrapper of the media.
pub async fn send_auto_complete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
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

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}