use serenity::all::{CommandInteraction, Context};

use crate::command_autocomplete::anilist::{
    anime, anime_group, character, compare, delete_activity, ln, manga, search, staff, studio, user,
};
use crate::command_autocomplete::game::steam_game_info;
use crate::common::get_option::subcommand_group::get_subcommand;

pub async fn autocomplete_dispatching(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction.data.name.as_str() {
        "admin" => admin_autocomplete(ctx, autocomplete_interaction).await,
        "anilist_user" => anilist_autocomplete(ctx, autocomplete_interaction).await,
        "steam" => steam_autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn admin_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
    {
        "anilist" => anilist_admin_autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn anilist_admin_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let subcommand = get_subcommand(&autocomplete_interaction).unwrap();
    let subcommand_name = subcommand.name;
    match subcommand_name {
        "add_anime_activity" => anime_group::autocomplete(ctx, autocomplete_interaction).await,
        "delete_activity" => delete_activity::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn anilist_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
    {
        "anime" => anime::autocomplete(ctx, autocomplete_interaction).await,
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

async fn steam_autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    match autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
    {
        "game" => steam_game_info::autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}
