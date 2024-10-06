use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::error_management::error_dispatch;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character;
use crate::structure::run::anilist::character::{
    Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
    CharacterQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

pub struct CharacterCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for CharacterCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for CharacterCommand {
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
    // Retrieve the name or ID of the character from the command interaction options
    let map = get_option_map_string(command_interaction);

    let value = map
        .get(&String::from("name"))
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
) -> Result<Character, Box<dyn Error>> {
    let var = CharacterQuerryIdVariables { id: Some(value) };

    let operation = CharacterQuerryId::build(var);

    let data: GraphQlResponse<CharacterQuerryId> =
        make_request_anilist(operation, false, anilist_cache).await?;

    Ok(match data.data {
        Some(data) => match data.character {
            Some(media) => media,
            None => {
                return Err(Box::new(error_dispatch::Error::Option(
                    "No character found".to_string(),
                )))
            }
        },
        None => {
            return Err(Box::new(error_dispatch::Error::Option(
                "No data found".to_string(),
            )))
        }
    })
}
