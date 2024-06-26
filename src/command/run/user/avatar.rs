use crate::config::Config;
use crate::constant::COLOR;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::helper::get_user_data::get_user_data;
use crate::structure::message::user::avatar::load_localization_avatar;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};
use std::sync::Arc;

/// Executes the command to display a user's avatar.
///
/// This function retrieves the user's name from the command interaction, checks if the user exists,
/// and then calls the appropriate function to display the avatar based on whether the user exists or not.
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
) -> Result<(), AppError> {
    // Retrieve the user's name from the command interaction
    let map = get_option_map_user_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    // Check if the user exists
    match user {
        Some(user) => {
            let user = get_user_data(ctx.http.clone(), user).await?;
            // If the user exists, retrieve the user's information and display their avatar
            avatar_with_user(ctx, command_interaction, &user).await
        }
        None => {
            // If the user does not exist, display the avatar of the user who triggered the command
            avatar_without_user(ctx, command_interaction).await
        }
    }
}

/// Displays the avatar of the user who triggered the command.
///
/// This function retrieves the user who triggered the command and calls the function to display their avatar.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn avatar_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    // Retrieve the user who triggered the command
    let user = command_interaction.user.clone();
    // Display the user's avatar
    avatar_with_user(ctx, command_interaction, &user).await
}

/// Displays the avatar of a specified user.
///
/// This function retrieves the avatar URL of the specified user and the server avatar of the user if they are a member of the guild.
/// It then calls the `send_embed` function to send an embed with the user's avatar.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `user` - The user whose avatar is to be displayed.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn avatar_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.face();
    let guild_id = command_interaction.guild_id.unwrap_or_default();
    let user_id = user.id;
    let server_avatar = match guild_id.member(&ctx.http, user_id).await {
        Ok(member) => member.avatar_url(),
        Err(_) => None,
    };
    send_embed(
        avatar_url,
        ctx,
        command_interaction,
        user.name.clone(),
        server_avatar,
    )
    .await
}

/// Sends an embed with a user's avatar.
///
/// This function creates an embed with the user's avatar and sends it as a response to the command interaction.
/// If the user has a server avatar, it creates a second embed with the server avatar and sends it as well.
///
/// # Arguments
///
/// * `avatar_url` - The URL of the user's avatar.
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `username` - The name of the user.
/// * `server_avatar` - The URL of the user's server avatar, if they have one.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: String,
    server_avatar: Option<String>,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let avatar_localised = load_localization_avatar(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(avatar_url)
        .title(avatar_localised.title.replace("$user$", username.as_str()));

    let builder_message = if server_avatar.is_none() {
        CreateInteractionResponseMessage::new().embed(builder_embed)
    } else {
        let second_builder_embed = get_default_embed(None).image(server_avatar.unwrap()).title(
            avatar_localised
                .server_title
                .replace("$user$", username.as_str()),
        );
        let embeds = vec![builder_embed, second_builder_embed];
        CreateInteractionResponseMessage::new().embeds(embeds)
    };

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
