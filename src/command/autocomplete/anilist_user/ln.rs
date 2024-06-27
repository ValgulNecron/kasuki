use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::structure::autocomplete::anilist::media::{
    send_auto_complete, MediaAutocompleteVariables, MediaFormat, MediaType,
};
use serenity::all::{CommandInteraction, Context};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for light novel search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the light novel name from the map of options.
/// If the light novel name is not found in the map, it defaults to a predefined string.
/// It then creates a new `MediaPageWrapper` for the autocomplete light novel search with the light novel name.
/// It sends the autocomplete response with the `MediaPageWrapper`.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is called.
/// * `autocomplete_interaction` - The command interaction that triggered this function.
///
/// # Async
///
/// This function is asynchronous. It awaits the creation of the `MediaPageWrapper` and the sending of the autocomplete response.
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let ln_search = map.get(&String::from("ln_name")).unwrap_or(DEFAULT_STRING);
    let var = MediaAutocompleteVariables {
        search: Some(ln_search.as_str()),
        in_media_format: Some(vec![Some(MediaFormat::Novel)]),
        media_type: Some(MediaType::Manga),
    };
    send_auto_complete(ctx, autocomplete_interaction, var, anilist_cache).await;
}
