use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;
use serenity::model::Timestamp;

use crate::constant::COLOR;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::structure::embed::general::struct_lang_profile::ProfileLocalisedText;
use crate::structure::register::general::struct_profile_register::RegisterLocalisedProfile;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            return send_embed(ctx, command, user.clone()).await;
        }
    }
    let user = &command.user;
    send_embed(ctx, command, user.clone()).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let profiles = RegisterLocalisedProfile::get_profile_register_localised().unwrap();
    let command = command
        .name("profile")
        .description("Show the profile of a user")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the profile of")
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
            .description_localized(&profile.code, &profile.desc);
    }
    command
}

async fn description(
    user: User,
    command: &ApplicationCommandInteraction,
    localised_text: ProfileLocalisedText,
) -> Result<String, AppError> {
    let is_bot = &user.bot;
    let public_flag = &user.public_flags.ok_or(LangageGuildIdError(String::from(
        "Guild id for langage not found.",
    )))?;
    let user_id = &user.id;
    let created_at = &user.created_at();
    let member = &command
        .member
        .clone()
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?;
    let joined_at = member.joined_at.unwrap_or(
        Timestamp::from_unix_timestamp(0i64)
            .map_err(|_| LangageGuildIdError(String::from("Guild id for langage not found.")))?,
    );
    Ok(format!(
        "\n {}{} \n {}{} \n {}{:?} \n {}{} \n {}{}",
        &localised_text.user_id,
        user_id,
        &localised_text.is_bot,
        is_bot,
        &localised_text.public_flag,
        public_flag,
        &localised_text.created_at,
        created_at,
        &localised_text.joined_at,
        joined_at
    ))
}

async fn send_embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user: User,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = ProfileLocalisedText::get_profile_localised(guild_id).await?;
    let avatar_url = match user.avatar_url() {
        Some(a) => a,
        None => "exemple.com".to_string(),
    };
    let desc = description(user.clone(), command, localised_text.clone()).await?;
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, user.name))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .thumbnail(avatar_url)
                            .description(desc)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
