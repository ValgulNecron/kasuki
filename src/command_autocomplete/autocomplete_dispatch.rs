use serenity::all::{CommandInteraction, Context};

use crate::command_autocomplete::anilist::{
    anime, character, compare, ln, manga, search, staff, studio, user,
};

pub async fn autocomplete_dispatching(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction.data.name.as_str() {
        "anime" => anime::autocomplete(ctx, autocomplete_interaction).await,
        "add_activity" => anime::autocomplete(ctx, autocomplete_interaction).await,
        "ln" => ln::autocomplete(ctx, autocomplete_interaction).await,
        "manga" => manga::autocomplete(ctx, autocomplete_interaction).await,
        "user" => user::autocomplete(ctx, autocomplete_interaction).await,
        "character" => character::autocomplete(ctx, autocomplete_interaction).await,
        "compare" => compare::autocomplete(ctx, autocomplete_interaction).await,
        "register" => user::autocomplete(ctx, autocomplete_interaction).await,
        "staff" => staff::autocomplete(ctx, autocomplete_interaction).await,
        "studio" => studio::autocomplete(ctx, autocomplete_interaction).await,
        "search" => search::autocomplete(ctx, autocomplete_interaction).await,
        "seiyuu" => staff::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}
