use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::common::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let manga_search = map
        .get(&String::from("manga_name"))
        .unwrap_or(DEFAULT_STRING);
    let manga = MediaPageWrapper::new_autocomplete_manga(manga_search).await;
    send_auto_complete(ctx, autocomplete_interaction, manga).await;
}
