use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::constant::DEFAULT_STRING;

#[cynic::schema("anilist")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct MediaAutocompleteVariables<'a> {
    pub in_media_format: Option<Vec<Option<MediaFormat>>>,
    pub media_type: Option<MediaType>,
    pub search: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "MediaAutocompleteVariables")]
pub struct MediaAutocomplete {
    #[arguments(perPage: 25)]
    #[cynic(rename = "Page")]
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "MediaAutocompleteVariables")]
pub struct Page {
    #[arguments(search: $ search, type: $ media_type, format_in: $ in_media_format)]
    pub media: Option<Vec<Option<Media>>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Media {
    pub id: i32,
    pub title: Option<MediaTitle>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MediaTitle {
    pub user_preferred: Option<String>,
    pub romaji: Option<String>,
    pub native: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MediaType {
    Anime,
    Manga,
}

pub async fn send_auto_complete<'a>(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    media: MediaAutocompleteVariables<'a>,
) {
    let operation = MediaAutocomplete::build(media);
    let data: GraphQlResponse<MediaAutocomplete> =
        match crate::helper::make_graphql_cached::make_request_anilist(operation, false).await {
            Ok(data) => match data.json::<GraphQlResponse<MediaAutocomplete>>().await {
                Ok(data) => data,
                Err(e) => {
                    tracing::trace!(?e);
                    return;
                }
            },
            Err(e) => {
                tracing::trace!(?e);
                return;
            }
        };

    let mut choices = Vec::new();
    let data = data.data.unwrap().page.unwrap().media.unwrap();
    for page in data {
        let data = page.unwrap();
        let title_data = data.title.unwrap();
        let english = title_data.user_preferred;
        let romaji = title_data.romaji;
        let native = title_data.native;
        let title = english.unwrap_or(romaji.unwrap_or(native.unwrap_or(DEFAULT_STRING.clone())));
        choices.push(AutocompleteChoice::new(title, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}
