use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut search = String::new();
    for option in &command.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
    let manga = MediaPageWrapper::new_autocomplete_manga(&search.to_string()).await;
    send_auto_complete(ctx, command, manga).await;
}
