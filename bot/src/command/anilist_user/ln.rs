use anyhow::Result;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media;
use crate::structure::run::anilist::media::{
    Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
    MediaQuerrySearchVariables, MediaType,
};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct LnCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for LnCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for LnCommand {
    async fn run_slash(&self) -> Result<()> {
        let ctx = self.get_ctx();
        let bot_data = ctx.data::<BotData>().clone();
        send_embed(
            &self.ctx,
            &self.command_interaction,
            bot_data.config.clone(),
            bot_data.anilist_cache.clone(),
        )
        .await
    }
}

pub async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
    // Retrieve the name or ID of the LN from the command interaction
    let map = get_option_map_string(command_interaction);

    let value = map
        .get(&FixedString::from_str_trunc("ln_name"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the LN data by ID if the value can be parsed as an `i32`, or by search otherwise
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>()?;

        let var = MediaQuerryIdVariables {
            format_in: Some(vec![Some(MediaFormat::Novel)]),
            id: Some(id),
            media_type: Some(MediaType::Manga),
        };

        let operation = MediaQuerryId::build(var);

        let data: GraphQlResponse<MediaQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;

        data.data.unwrap().media.unwrap()
    } else {
        let var = MediaQuerrySearchVariables {
            format_in: Some(vec![Some(MediaFormat::Novel)]),
            search: Some(&*value),
            media_type: Some(MediaType::Manga),
        };

        let operation = MediaQuerrySearch::build(var);

        let data: GraphQlResponse<MediaQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;

        data.data.unwrap().media.unwrap()
    };

    // Send an embed containing the LN data as a response to the command interaction
    media::send_embed(ctx, command_interaction, data, config.db.clone()).await?;

    Ok(())
}
