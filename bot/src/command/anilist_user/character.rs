use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::{
    Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
    CharacterQuerrySearchVariables,
};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use tokio::sync::RwLock;

pub struct CharacterCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl Command for CharacterCommand {
    fn get_ctx(&self) -> &SerenityContext {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for CharacterCommand {
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
    // Retrieve the name or ID of the character from the command interaction options
    let map = get_option_map_string(command_interaction);

    let value = map
        .get(&FixedString::from_str_trunc("name"))
        .cloned()
        .unwrap_or(String::new());

    // If the value is an integer, treat it as an ID and retrieve the character with that ID
    // If the value is not an integer, treat it as a name and retrieve the character with that name
    let data: Character = if value.parse::<i32>().is_ok() {
        get_character_by_id(value.parse::<i32>().unwrap(), anilist_cache).await?
    } else {
        let var = CharacterQuerrySearchVariables {
            search: Some(&*value),
        };

        let operation = CharacterQuerrySearch::build(var);

        let data: GraphQlResponse<CharacterQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;

        data.data.unwrap().character.unwrap()
    };

    // Send an embed with the character information as a response to the command interaction
    character::send_embed(ctx, command_interaction, data, config.db.clone()).await?;

    Ok(())
}

pub async fn get_character_by_id(
    value: i32,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Character> {
    let var = CharacterQuerryIdVariables { id: Some(value) };

    let operation = CharacterQuerryId::build(var);

    let data: GraphQlResponse<CharacterQuerryId> =
        make_request_anilist(operation, false, anilist_cache).await?;

    Ok(match data.data {
        Some(data) => match data.character {
            Some(media) => media,
            None => return Err(anyhow!("No character found")),
        },
        None => return Err(anyhow!("No data found")),
    })
}
