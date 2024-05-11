use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::constant::COLOR;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_user_subcommand;
use crate::structure::message::user::banner::load_localization_banner;

/// Executes the command to display a user's banner.
///
/// This function retrieves the user's name from the command interaction, checks if the user exists,
/// and then calls the appropriate function to display the banner based on whether the user exists or not.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_user_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    match user {
        Some(user) => {
            let user = user.to_user(&ctx.http).await.map_err(|e| {
                AppError::new(
                    format!("Could not get the user. {}", e),
                    ErrorType::Option,
                    ErrorResponseType::Message,
                )
            })?;
            banner_with_user(ctx, command_interaction, &user).await
        }
        None => banner_without_user(ctx, command_interaction).await,
    }
}

/// Sends a response indicating that the user does not have a banner.
///
/// This function is called when a user does not have a banner. It creates an embed with a message indicating that the user does not have a banner
/// and sends it as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `username` - The name of the user.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn no_banner(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    username: &str,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let banner_localised = load_localization_banner(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(banner_localised.no_banner.replace("$user$", username))
        .title(&banner_localised.no_banner_title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

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

/// Executes the command to display a user's banner when no user is specified.
///
/// This function retrieves the user who triggered the command and calls the `banner_with_user` function to display their banner.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn banner_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let user = &command_interaction.user;

    banner_with_user(ctx, command_interaction, user).await
}

/// Executes the command to display a specified user's banner.
///
/// This function retrieves the banner URL of the specified user and calls the `send_embed` function to send an embed with the user's banner.
/// If the user does not have a banner, it calls the `no_banner` function to send a response indicating that the user does not have a banner.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `user_data` - The user whose banner is to be displayed.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn banner_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user_data: &User,
) -> Result<(), AppError> {
    let user = user_data;
    let banner_url = match user.banner_url() {
        Some(banner) => banner,
        None => return no_banner(ctx, command_interaction, &user.name).await,
    };
    send_embed(ctx, command_interaction, banner_url, &user.name).await
}

/// Sends an embed with a user's banner.
///
/// This function creates an embed with the user's banner and sends it as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `banner` - The URL of the user's banner.
/// * `username` - The name of the user.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    banner: String,
    username: &str,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let banner_localised = load_localization_banner(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(banner)
        .title(banner_localised.title.replace("$user$", username));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

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
