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
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct AnimeCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for AnimeCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for AnimeCommand {
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

async fn send_embed(
    ctx: &SerenityContext,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<()> {
    // Retrieve the name or ID of the anime from the command interaction options
    let map = get_option_map_string(command_interaction);

    let value = map
        .get(&FixedString::from_str_trunc("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let format_in = Some(vec![
        Some(MediaFormat::Tv),
        Some(MediaFormat::TvShort),
        Some(MediaFormat::Movie),
        Some(MediaFormat::Special),
        Some(MediaFormat::Ova),
        Some(MediaFormat::Ona),
        Some(MediaFormat::Music),
    ]);

    // If the value is an integer, treat it as an ID and retrieve the anime with that ID
    // If the value is not an integer, treat it as a name and retrieve the anime with that name
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();

        let var = MediaQuerryIdVariables {
            format_in,
            id: Some(id),
            media_type: Some(MediaType::Anime),
        };

        let operation = MediaQuerryId::build(var);

        let data: GraphQlResponse<MediaQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;

        match data.data {
            Some(data) => match data.media {
                Some(media) => media,
                None => return Err(anyhow!("Anime not found")),
            },
            None => return Err(anyhow!("Anime not found")),
        }
    } else {
        let var = MediaQuerrySearchVariables {
            format_in,
            search: Some(&*value),
            media_type: Some(MediaType::Anime),
        };

        let operation = MediaQuerrySearch::build(var);

        let data: GraphQlResponse<MediaQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;

        match data.data {
            Some(data) => match data.media {
                Some(media) => media,
                None => return Err(anyhow!("Anime not found")),
            },
            None => return Err(anyhow!("Anime not found")),
        }
    };

    // Send an embed with the anime information as a response to the command interaction
    media::send_embed(ctx, command_interaction, data, config.db.clone()).await?;

    Ok(())
}
