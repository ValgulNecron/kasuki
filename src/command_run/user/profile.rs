use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::common::get_option::subcommand::get_option_map_user_subcommand;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::user::profile::load_localization_profile;

/// Executes the command to display a user's profile.
///
/// This function retrieves the user's name from the command interaction, checks if the user exists,
/// and then calls the appropriate function to display the profile based on whether the user exists or not.
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
    // Retrieve the user's name from the command interaction
    let map = get_option_map_user_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    // Check if the user exists
    match user {
        Some(user) => {
            // If the user exists, retrieve the user's information and display their profile
            let user = user.to_user(&ctx.http).await.map_err(|e| {
                AppError::new(
                    format!("Could not get the user. {}", e),
                    ErrorType::Option,
                    ErrorResponseType::Message,
                )
            })?;
            profile_with_user(ctx, command_interaction, &user).await
        }
        None => {
            // If the user does not exist, display the profile of the user who triggered the command
            profile_without_user(ctx, command_interaction).await
        }
    }
}

/// Displays the profile of the user who triggered the command.
///
/// This function retrieves the user who triggered the command and calls the function to display their profile.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn profile_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    // Retrieve the user who triggered the command
    let user = command_interaction.user.clone();
    // Display the user's profile
    profile_with_user(ctx, command_interaction, &user).await
}

/// Displays the profile of a specified user.
///
/// This function retrieves the avatar URL of the specified user and calls the `send_embed` function to send an embed with the user's profile.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `user` - The user whose profile is to be displayed.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
async fn profile_with_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    // Retrieve the avatar URL of the specified user
    let avatar_url = user.face();
    // Send an embed with the user's profile
    send_embed(avatar_url, ctx, command_interaction, user).await
}

/// Sends an embed with a user's profile.
///
/// This function creates an embed with the user's profile and sends it as a response to the command interaction.
/// It retrieves the guild ID from the command interaction and loads the localized profile.
/// It then retrieves the member from the command interaction and checks if there are any public flags for the user.
/// If there are, it iterates over the flags and adds them to a vector.
/// It then creates an embed with the user's profile information and sends it as a response to the command interaction.
///
/// # Arguments
///
/// * `avatar_url` - The URL of the user's avatar.
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
/// * `user` - The user whose profile is to be displayed.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    // Retrieve the guild ID from the command interaction
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    // Load the localized profile
    let profile_localised = load_localization_profile(guild_id).await?;

    // Retrieve the member from the command interaction
    let member = &command_interaction.member.clone().ok_or(AppError::new(
        String::from("There is no member in the option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;

    // Check if there are any public flags for the user
    let public_flag = match user.public_flags {
        Some(public_flag) => {
            let mut user_flags = Vec::new();
            // If there are, iterate over the flags and add them to a vector
            for (flag, _) in public_flag.iter_names() {
                user_flags.push(flag)
            }
            user_flags.join(" / ")
        }
        None => "None".to_string(),
    };

    // Create an embed with the user's profile information
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .thumbnail(avatar_url)
        .title(
            profile_localised
                .title
                .replace("$user$", user.name.as_str()),
        )
        .description(
            profile_localised
                .desc
                .replace("$user$", user.name.as_str())
                .replace("$id$", user.id.to_string().as_str())
                .replace("$creation_date$", user.created_at().to_string().as_str())
                .replace(
                    "$joined_date$",
                    member
                        .joined_at
                        .ok_or(AppError::new(
                            String::from("There is no joined date for the user"),
                            ErrorType::Option,
                            ErrorResponseType::Message,
                        ))?
                        .to_string()
                        .as_str(),
                )
                .replace("$bot$", user.bot.to_string().as_str())
                .replace("$public_flag$", public_flag.as_str())
                .replace("$nitro$", format!("{:?}", user.premium_type).as_str())
                .replace("$system$", user.system.to_string().as_str()),
        );

    // Create a message with the embed
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    // Create a response with the message
    let builder = CreateInteractionResponse::Message(builder_message);

    // Send the response
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
