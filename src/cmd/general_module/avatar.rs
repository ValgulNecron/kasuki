use crate::constant::COLOR;
use crate::error_enum::AppError::{FailedToGetUser, LangageGuildIdError};
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR, NO_AVATAR_ERROR};
use crate::structure::embed::general::struct_lang_avatar::AvatarLocalisedText;
use crate::structure::register::general::struct_avatar_register::RegisterLocalisedAvatar;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::Http;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{InteractionResponseType, User};
use serenity::model::Timestamp;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            return avatar_with_user(ctx, command, user).await;
        }
    }
    avatar_without_user(ctx, command).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let profiles = RegisterLocalisedAvatar::get_avatar_register_localised().unwrap();
    let command = command
        .name("avatar")
        .description("Show the avatar of a user")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the avatar of")
                .kind(CommandOptionType::User)
                .required(false);
            for profile in profiles.values() {
                option
                    .name_localized(&profile.code, &profile.option1)
                    .description_localized(&profile.code, &profile.option1_desc);
            }
            option
        });
    for profile in profiles.values() {
        command
            .name_localized(&profile.code, &profile.name)
            .description_localized(&profile.code, &profile.description);
    }
    command
}

async fn avatar_without_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    let user = command.user.id.0;
    let real_user = Http::get_user(&ctx.http, user).await;
    let result = real_user.map_err(|_| FailedToGetUser(String::from("Could no resolve user.")))?;

    avatar_with_user(ctx, command, &result).await
}

async fn avatar_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user: &User,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = AvatarLocalisedText::get_avatar_localised(guild_id).await?;

    let avatar_url = user.avatar_url().ok_or(NO_AVATAR_ERROR.clone())?;

    send_embed(
        avatar_url,
        ctx,
        command,
        localised_text.clone(),
        user.name.clone(),
    )
    .await
}

pub async fn send_embed(
    avatar_url: String,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: AvatarLocalisedText,
    username: String,
) -> Result<(), AppError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, username))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .image(avatar_url)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
