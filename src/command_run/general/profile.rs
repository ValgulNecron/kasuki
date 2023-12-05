use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{FailedToGetUser, LangageGuildIdError};
use crate::lang_struct::general::profile::load_localization_profile;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, Timestamp, User, UserPublicFlags,
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
            return profile_with_user(ctx, command, &user).await;
        }
    }
    profile_without_user(ctx, command).await
}

async fn profile_without_user(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let user = command.user.clone();
    profile_with_user(ctx, command, &user).await
}

async fn profile_with_user(
    ctx: &Context,
    command: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let avatar_url = user.avatar_url().ok_or(OPTION_ERROR.clone())?;

    send_embed(avatar_url, ctx, command, user).await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command: &CommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let profile_localised = load_localization_profile(guild_id).await?;

    let member = &command.member.clone().ok_or(OPTION_ERROR.clone())?;

    let public_flag = match user.public_flags {
        Some(public_flag) => {
            let mut user_flags = Vec::new();
            for (flag, _) in public_flag.iter_names() {
                user_flags.push(flag);
            }
            user_flags.join(" / ")
        }
        None => "None".to_string(),
    };

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
                        .ok_or(OPTION_ERROR.clone())?
                        .to_string()
                        .as_str(),
                )
                .replace("$bot$", user.bot.to_string().as_str())
                .replace("$public_flag$", public_flag.as_str())
                .replace("$nitro$", format!("{:?}", user.premium_type).as_str())
                .replace("$system$", user.system.to_string().as_str()),
        );

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command
        .create_response(&ctx.http, builder)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}