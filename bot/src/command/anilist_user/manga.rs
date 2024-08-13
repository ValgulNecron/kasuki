use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
    Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
    MediaQuerrySearchVariables, MediaType,
};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

pub struct MangaCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for MangaCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for MangaCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        send_embed(
            &self.ctx,
            &self.command_interaction,
            self.config.clone(),
            self.anilist_cache.clone(),
        )
        .await
    }
}
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the name or ID of the manga from the command interaction
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("manga_name"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the manga data by ID if the value can be parsed as an `i32`, or by search otherwise
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = MediaQuerryIdVariables {
            format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
            id: Some(id),
            media_type: Some(MediaType::Manga),
        };

        let operation = MediaQuerryId::build(var);
        let data: GraphQlResponse<MediaQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().media.unwrap()
    } else {
        let var = MediaQuerrySearchVariables {
            format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
            search: Some(&*value),
            media_type: Some(MediaType::Manga),
        };
        let operation = MediaQuerrySearch::build(var);
        let data: GraphQlResponse<MediaQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().media.unwrap()
    };

    // Send an embed containing the manga data as a response to the command interaction
    media::send_embed(
        ctx,
        command_interaction,
        data,
        db_type,
        config.bot.config.clone(),
    )
    .await?;
    Ok(())
}
