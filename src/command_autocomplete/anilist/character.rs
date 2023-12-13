use crate::anilist_struct::autocomplete::media::{send_auto_complete, MediaPageWrapper};
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let search = &command
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap();
    let anime = MediaPageWrapper::new_autocomplete_anime(&search.to_string()).await;
    send_auto_complete(ctx, command, anime).await;
}
