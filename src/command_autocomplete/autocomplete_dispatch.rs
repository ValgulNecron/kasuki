use crate::command_autocomplete::anilist::anime;
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete_dispatching(ctx: Context, command: CommandInteraction) {
    match command.data.name.as_str() {
        "anime" => anime::autocomplete(ctx, command).await,
        _ => {}
    }
}
