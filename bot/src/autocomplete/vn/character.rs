use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tokio::sync::RwLock;
use tracing::trace;

use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::vndbapi::character::{get_character, Character};

pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);

    let game = map.get(&String::from("name")).unwrap();

    let char = get_character(game.clone(), vndb_cache).await.unwrap();

    let vn_result = char.results;

    // take the 25 first results
    let characters: Vec<Character> = vn_result.iter().take(25).cloned().collect();

    let mut choices = Vec::new();

    trace!("Game: {}", game);

    trace!("Map: {:?}", map);

    for character in characters {
        choices.push(AutocompleteChoice::new(
            character.name.clone(),
            character.id.clone(),
        ))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);

    let builder = CreateInteractionResponse::Autocomplete(data);

    if let Err(e) = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await
    {
        tracing::error!("Error sending response: {:?}", e);
    }
}
