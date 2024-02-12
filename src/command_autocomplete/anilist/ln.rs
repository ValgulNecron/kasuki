use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use crate::command_run::get_option::get_option_map_string;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string(&autocomplete_interaction);
    let ln_search = map.get(&String::from("ln_name")).unwrap_or(DEFAULT_STRING);
    let manga = MediaPageWrapper::new_autocomplete_ln(ln_search).await;
    send_auto_complete(ctx, autocomplete_interaction, manga).await;
}
