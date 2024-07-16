use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;
use tracing::trace;
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::structure::autocomplete::anilist::media::{
    send_auto_complete, MediaAutocompleteVariables, MediaFormat, MediaType,
};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for anime search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the anime name from the map of options.
/// If the anime name is not found in the map, it defaults to a predefined string.
/// It then creates a new `MediaPageWrapper` for the autocomplete anime search with the anime name.
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
    let anime_search = map
        .get(&String::from("anime_name"))
        .unwrap_or(DEFAULT_STRING);
    trace!("anime_search: {}", anime_search);
    let var = get_autocomplete_media_variables(anime_search);
    send_auto_complete(ctx, autocomplete_interaction, var, anilist_cache).await;
}

pub fn get_autocomplete_media_variables(anime_search: &str) -> MediaAutocompleteVariables {
    MediaAutocompleteVariables {
        search: Some(anime_search),
        in_media_format: Some(vec![
            Some(MediaFormat::Tv),
            Some(MediaFormat::TvShort),
            Some(MediaFormat::Movie),
            Some(MediaFormat::Special),
            Some(MediaFormat::Ova),
            Some(MediaFormat::Ona),
            Some(MediaFormat::Music),
        ]),
        media_type: Some(MediaType::Anime),
    }
}
