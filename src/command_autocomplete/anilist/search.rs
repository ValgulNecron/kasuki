use serenity::all::{CommandInteraction, Context};

use crate::command_autocomplete::anilist::{anime, character, ln, manga, staff, studio, user};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search_type = String::new();
    for option in &autocomplete_interaction.data.options {
        if option.name.as_str() == "type" {
            search_type = option.value.as_str().unwrap().to_string()
        }
    }
    match search_type.as_str() {
        "anime" => anime::autocomplete(ctx, autocomplete_interaction).await,
        "ln" => ln::autocomplete(ctx, autocomplete_interaction).await,
        "manga" => manga::autocomplete(ctx, autocomplete_interaction).await,
        "user" => user::autocomplete(ctx, autocomplete_interaction).await,
        "character" => character::autocomplete(ctx, autocomplete_interaction).await,
        "staff" => staff::autocomplete(ctx, autocomplete_interaction).await,
        "studio" => studio::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}
