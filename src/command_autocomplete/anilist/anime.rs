use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search = String::new();
    for option in &autocomplete_interaction.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
    let anime = MediaPageWrapper::new_autocomplete_anime(&search.to_string()).await;
    send_auto_complete(ctx, autocomplete_interaction, anime).await;
}
