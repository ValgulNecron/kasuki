use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::command_run::get_option::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let anime_search = map
        .get(&String::from("anime_name"))
        .unwrap_or(DEFAULT_STRING);
    let anime = MediaPageWrapper::new_autocomplete_anime(anime_search).await;
    send_auto_complete(ctx, autocomplete_interaction, anime).await;
}
