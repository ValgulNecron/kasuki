use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

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
pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let ln_search = map.get(&String::from("ln_name")).unwrap_or(DEFAULT_STRING);
    let manga = MediaPageWrapper::new_autocomplete_ln(ln_search).await;
    send_auto_complete(ctx, autocomplete_interaction, manga).await;
}