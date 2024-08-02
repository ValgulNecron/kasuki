use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

use crate::command::run::anilist_user::user::get_user;
use crate::config::Config;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::set_registered_user;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::anilist_user::register::load_localization_register;
use crate::structure::run::anilist::user::{get_color, get_user_url, User};

/// Executes the command to register a user's AniList account.
///
/// This function retrieves the username of the AniList account from the command interaction and fetches the user data from AniList.
/// It then registers the user's AniList account by storing the user's Discord ID and AniList ID in the database.
/// The function then sends a response to the command interaction containing an embed with the user's AniList information.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the username of the AniList account from the command interaction
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("username"))
        .ok_or(ResponseError::Option(String::from("No username provided")))?;

    // Fetch the user data from AniList
    let user_data: User = get_user(value, anilist_cache).await?;

    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized register strings
    let register_localised = load_localization_register(guild_id, db_type.clone()).await?;

    // Retrieve the user's Discord ID and username
    let user_id = &command_interaction.user.id.to_string();
    let username = &command_interaction.user.name;

    // Register the user's AniList account by storing the user's Discord ID and AniList ID in the database
    let registered_user = RegisteredUser {
        user_id: user_id.clone(),
        anilist_id: user_data.id.to_string(),
    };
    set_registered_user(registered_user, db_type).await?;

    // Construct the description for the embed
    let desc = register_localised
        .desc
        .replace("$user$", username.as_str())
        .replace("$id$", user_id)
        .replace("$anilist$", user_data.name.clone().as_str());

    // Construct the embed
    let builder_embed = get_default_embed(Some(get_color(user_data.clone())))
        .title(user_data.name)
        .url(get_user_url(user_data.id))
        .thumbnail(user_data.avatar.unwrap().large.unwrap())
        .description(desc);

    // Construct the message for the response
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Construct the response
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response to the command interaction
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}
