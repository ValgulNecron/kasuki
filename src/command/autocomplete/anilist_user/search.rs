use serenity::all::{CommandInteraction, Context};

use crate::command::autocomplete::anilist_user::{
    anime, character, ln, manga, staff, studio, user,
};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for various search types.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the search type from the map of options.
/// If the search type is not found in the map, it defaults to a predefined string.
/// It then matches the search type to the corresponding autocomplete function.
/// If the search type does not match any of the predefined types, it does nothing.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is called.
/// * `autocomplete_interaction` - The command interaction that triggered this function.
///
/// # Async
///
/// This function is asynchronous. It awaits the execution of the corresponding autocomplete function.
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    cache_type: String,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let search_type = map.get(&String::from("type")).unwrap_or(DEFAULT_STRING);

    match search_type.as_str() {
        "anime" => anime::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "ln" => ln::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "manga" => manga::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "user" => user::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "character" => character::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "staff" => staff::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        "studio" => studio::autocomplete(ctx, autocomplete_interaction, cache_type).await,
        _ => {}
    }
}
