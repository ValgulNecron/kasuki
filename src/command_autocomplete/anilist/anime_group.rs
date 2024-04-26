use serenity::all::{CommandInteraction, Context};
use tracing::trace;

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::common::get_option::subcommand_group::get_option_map_string_autocomplete_subcommand_group;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand_group(&autocomplete_interaction);
    let anime_search = map
        .get(&String::from("anime_name"))
        .unwrap_or(DEFAULT_STRING);
    let anime = MediaPageWrapper::new_autocomplete_anime(anime_search).await;
    send_auto_complete(ctx, autocomplete_interaction, anime).await;
}
