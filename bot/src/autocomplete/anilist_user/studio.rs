use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tokio::sync::RwLock;

use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::studio::{
    StudioAutocomplete, StudioAutocompleteVariables,
};

pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);

    let studio_search = map.get(&String::from("studio")).unwrap_or(DEFAULT_STRING);

    let var = StudioAutocompleteVariables {
        search: Some(studio_search),
    };

    let operation = StudioAutocomplete::build(var);

    let data: GraphQlResponse<StudioAutocomplete> =
        match make_request_anilist(operation, false, anilist_cache).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(?e);

                return;
            }
        };

    let studios = match data.data {
        Some(data) => match data.page {
            Some(page) => match page.studios {
                Some(studios) => studios,
                None => return,
            },
            None => return,
        },
        None => {
            tracing::debug!(?data.errors);

            return;
        }
    };

    let mut choices = Vec::new();

    for studio in studios {
        let data = studio.unwrap();

        let user = data.name;

        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);

    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(&ctx.http, builder)
        .await;
}
