use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::helper::get_option::subcommand_group::get_option_map_string_autocomplete_subcommand_group;
use crate::constant::DEFAULT_STRING;

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for anime group search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the anime name from the map of options.
/// If the anime name is not found in the map, it defaults to a predefined string.
/// It then creates a new `MediaPageWrapper` for the autocomplete anime group search with the anime name.
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
pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand_group(&autocomplete_interaction);
    let anime_search = map
        .get(&String::from("anime_name"))
        .unwrap_or(DEFAULT_STRING);
    let anime = MediaPageWrapper::new_autocomplete_anime(anime_search).await;
    send_auto_complete(ctx, autocomplete_interaction, anime).await;
}