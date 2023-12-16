use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp, User,
};

use crate::constant::{COLOR, COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::FailedToGetUser;
use crate::lang_struct::general::banner::load_localization_banner;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    if let Some(option) = options.get(0) {
        let resolved = &option.value;
        if let CommandDataOptionValue::User(user, ..) = resolved {
            let user = user
                .to_user(&ctx.http)
                .await
                .map_err(|_| FailedToGetUser(String::from("Failed to get the user.")))?;
            return banner_with_user(ctx, command, &user).await;
        }
    }
    banner_without_user(ctx, command).await
}

pub async fn no_banner(
    ctx: &Context,
    command: &CommandInteraction,
    username: &str,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
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

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub async fn banner_without_user(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let user = &command.user;

    banner_with_user(ctx, command, user).await
}

pub async fn banner_with_user(
    ctx: &Context,
    command: &CommandInteraction,
    user_data: &User,
) -> Result<(), AppError> {
    let user = user_data;
    let banner_url = match user.banner_url() {
        Some(banner) => banner,
        None => return no_banner(ctx, command, &user.name).await,
    };
    send_embed(ctx, command, banner_url).await
}

pub async fn send_embed(
    ctx: &Context,
    command: &CommandInteraction,
    banner: String,
) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let banner_localised = load_localization_banner(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(banner)
        .title(&banner_localised.no_banner_title);

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
