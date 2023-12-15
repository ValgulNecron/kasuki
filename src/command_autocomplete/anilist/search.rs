use crate::command_autocomplete::anilist::{anime, character, ln, manga, staff, studio, user};
use serenity::all::{CommandInteraction, Context};

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut search_type = String::new();
    for option in &command.data.options {
        if option.name.as_str() == "type" {
            search_type = option.value.as_str().unwrap().to_string()
        }
    }
    match search_type.as_str() {
        "anime" => anime::autocomplete(ctx, command).await,
        "ln" => ln::autocomplete(ctx, command).await,
        "manga" => manga::autocomplete(ctx, command).await,
        "user" => user::autocomplete(ctx, command).await,
        "character" => character::autocomplete(ctx, command).await,
        "staff" => staff::autocomplete(ctx, command).await,
        "studio" => studio::autocomplete(ctx, command).await,
        _ => {}
    }
}
