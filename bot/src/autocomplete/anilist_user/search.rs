use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::autocomplete::anilist_user::{anime, character, ln, manga, staff, studio, user};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);

    let search_type = map.get(&String::from("type")).unwrap_or(DEFAULT_STRING);

    match search_type.as_str() {
        "anime" => anime::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "ln" => ln::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "manga" => manga::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "user" => user::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "character" => character::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "staff" => staff::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "studio" => studio::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        _ => {}
    }
}
