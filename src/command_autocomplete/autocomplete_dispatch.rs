use serenity::all::{CommandInteraction, Context};

use crate::command_autocomplete::anilist::{
    anime, character, compare, delete_activity, ln, manga, search, staff, studio, user,
};
use crate::command_autocomplete::game::steam_game_info;

pub async fn autocomplete_dispatching(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction.data.name.as_str() {
        "anilist" => anilist_autocomplete(ctx, autocomplete_interaction).await,
        "steam" => steam_autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn anilist_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction.data
        .options
        .first()
        .unwrap()
        .name
        .as_str(){
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
        "delete_activity" => delete_activity::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn steam_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction.data
        .options
        .first()
        .unwrap()
        .name
        .as_str(){
        "game" => steam_game_info::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}