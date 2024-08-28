use std::collections::HashMap;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::autocomplete::anilist_server::{add_anime_activity, delete_activity};
use crate::autocomplete::anilist_user::{
    anime, character, compare, ln, manga, search, staff, studio, user,
};
use crate::autocomplete::game::steam_game_info;
use crate::autocomplete::management::give_premium_sub::give_premium_sub_autocomplete;
use crate::autocomplete::vn;
use crate::autocomplete::vn::{game, producer};
use crate::config::DbConfig;
use crate::event_handler::Handler;
use crate::helper::get_option::subcommand_group::get_subcommand;

pub async fn autocomplete_dispatching(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    self_handler: &Handler,
) {
    let anilist_cache = self_handler.bot_data.anilist_cache.clone();
    let vndb_cache = self_handler.bot_data.vndb_cache.clone();
    let apps = self_handler.bot_data.apps.clone();
    let db_config = self_handler.bot_data.config.db.clone();
    match autocomplete_interaction.data.name.as_str() {
        "admin" => {
            admin_autocomplete(ctx, autocomplete_interaction, anilist_cache, db_config).await
        }
        "anime" => anime::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "ln" => ln::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "manga" => manga::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "user" => user::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "character" => character::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "compare" => compare::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "register" => user::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "staff" => staff::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "studio" => studio::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "search" => search::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "seiyuu" => staff::autocomplete(ctx, autocomplete_interaction, anilist_cache).await,
        "steam" => steam_autocomplete(ctx, autocomplete_interaction, apps).await,
        "vn" => vn_autocomplete(ctx, autocomplete_interaction, vndb_cache).await,
        "give_premium_sub" => give_premium_sub_autocomplete(ctx, autocomplete_interaction).await,
        _ => {}
    }
}

async fn admin_autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) {
    if autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
        == "anilist"
    {
        anilist_admin_autocomplete(ctx, autocomplete_interaction, anilist_cache, db_config).await
    }
}

async fn vn_autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) {
    match autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
    {
        "game" => game::autocomplete(ctx, autocomplete_interaction, vndb_cache).await,
        "character" => vn::character::autocomplete(ctx, autocomplete_interaction, vndb_cache).await,
        "producer" => producer::autocomplete(ctx, autocomplete_interaction, vndb_cache).await,
        _ => {}
    }
}
async fn anilist_admin_autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) {
    let subcommand = get_subcommand(&autocomplete_interaction).unwrap();
    let subcommand_name = subcommand.name;
    match subcommand_name {
        "add_anime_activity" => {
            add_anime_activity::autocomplete(ctx, autocomplete_interaction, anilist_cache).await
        }
        "delete_activity" => {
            delete_activity::autocomplete(ctx, autocomplete_interaction, db_config).await
        }
        _ => {}
    }
}

async fn steam_autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    apps: Arc<RwLock<HashMap<String, u128>>>,
) {
    if autocomplete_interaction
        .data
        .options
        .first()
        .unwrap()
        .name
        .as_str()
        == "game"
    {
        steam_game_info::autocomplete(ctx, autocomplete_interaction, apps).await
    }
}
