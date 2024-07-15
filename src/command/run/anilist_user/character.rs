use std::error::Error;
use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::helper::error_management::error_enum::UnknownResponseError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::character::{
    send_embed, Character, CharacterQuerryId, CharacterQuerryIdVariables, CharacterQuerrySearch,
    CharacterQuerrySearchVariables,
};

/// This asynchronous function runs the command interaction for retrieving information about a character.
///
/// It first retrieves the name or ID of the character from the command interaction options.
///
/// If the value is an integer, it treats it as an ID and retrieves the character with that ID.
/// If the value is not an integer, it treats it as a name and retrieves the character with that name.
///
/// It sends an embed with the character information as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the name or ID of the character from the command interaction options
    let map = get_option_map_string_subcommand(command_interaction);
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
    send_embed(ctx, command_interaction, data, db_type).await?;

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
                return Err(Box::new(UnknownResponseError::Option(
                    "No character found".to_string(),
                )))
            }
        },
        None => {
            return Err(Box::new(UnknownResponseError::Option(
                "No data found".to_string(),
            )))
        }
    })
}
