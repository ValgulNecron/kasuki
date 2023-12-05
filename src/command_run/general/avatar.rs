use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{FailedToGetUser, LangageGuildIdError};
use crate::lang_struct::general::avatar::load_localization_avatar;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp, User,
};

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
                .map_err(|_| FailedToGetUser(String::from("Could not get the user.")))?;
            return avatar_with_user(ctx, command, &user).await;
        }
    }
    avatar_without_user(ctx, command).await
}

async fn avatar_without_user(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let user = command.user.clone();
    avatar_with_user(ctx, command, &user).await
}

async fn avatar_with_user(
    ctx: &Context,
    command: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.avatar_url().ok_or(OPTION_ERROR.clone())?;

    send_embed(avatar_url, ctx, command, user.name.clone()).await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command: &CommandInteraction,
    username: String,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let avatar_localised = load_localization_avatar(guild_id).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(avatar_url)
        .title(avatar_localised.title.replace("$user$", username.as_str()));

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}