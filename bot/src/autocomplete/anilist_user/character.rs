use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::character::{
    CharacterAutocomplete, CharacterAutocompleteVariables,
};

pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);

    let character_search = map.get(&String::from("name")).unwrap_or(DEFAULT_STRING);

    let var = CharacterAutocompleteVariables {
        search: Some(character_search.as_str()),
    };

    let operation = CharacterAutocomplete::build(var);

    let data: GraphQlResponse<CharacterAutocomplete> =
        match make_request_anilist(operation, false, anilist_cache).await {
            Ok(data) => data,
            Err(e) => {
                tracing::debug!(?e);

                return;
            }
        };

    trace!(?data);

    let mut choices = Vec::new();

    let characters = match data.data {
        Some(data) => match data.page {
            Some(page) => match page.characters {
                Some(characters) => characters,
                None => return,
            },
            None => return,
        },
        None => {
            tracing::debug!(?data.errors);

            return;
        }
    };

    for character in characters {
        let data = character.unwrap();

        let name = data.name.unwrap();

        let full = name.full.clone();

        let user_pref = name.user_preferred.clone();

        let native = name.native.clone();

        let name = user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(DEFAULT_STRING.clone())));

        choices.push(AutocompleteChoice::new(name, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);

    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(&ctx.http, builder)
        .await;
}
