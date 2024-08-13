use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tokio::sync::RwLock;

use crate::command::anilist_user::user::get_user;
use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::set_registered_user;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::register::load_localization_register;
use crate::structure::run::anilist::user::{get_color, get_user_url, User};

pub struct RegisterCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for RegisterCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for RegisterCommand {
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
    let register_localised =
        load_localization_register(guild_id, db_type.clone(), config.bot.config.clone()).await?;

    // Retrieve the user's Discord ID and username
    let user_id = &command_interaction.user.id.to_string();
    let username = &command_interaction.user.name;

    // Register the user's AniList account by storing the user's Discord ID and AniList ID in the database
    let registered_user = RegisteredUser {
        user_id: user_id.clone(),
        anilist_id: user_data.id.to_string(),
    };
    set_registered_user(registered_user, db_type, config.bot.config.clone()).await?;

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
