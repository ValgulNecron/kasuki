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

    send_auto_complete(&ctx, autocomplete_interaction, var, anilist_cache).await;
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
