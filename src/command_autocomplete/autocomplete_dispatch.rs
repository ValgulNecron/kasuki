use crate::command_autocomplete::anilist::{anime, character, ln, manga, user};
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete_dispatching(ctx: Context, command: CommandInteraction) {
    match command.data.name.as_str() {
        "anime" => anime::autocomplete(ctx, command).await,
        "add_activity" => anime::autocomplete(ctx, command).await,
        "ln" => ln::autocomplete(ctx, command).await,
        "manga" => manga::autocomplete(ctx, command).await,
        "user" => user::autocomplete(ctx, command).await,
        "character" => character::autocomplete(ctx, command).await,
        _ => {}
    }
}
