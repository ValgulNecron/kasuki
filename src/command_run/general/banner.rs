use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, Timestamp, User,
};

use crate::command_run::get_option::get_option_map_user_subcommand;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::general::banner::load_localization_banner;

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

pub async fn banner_without_user(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let user = &command_interaction.user;

    banner_with_user(ctx, command_interaction, user).await
}

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
