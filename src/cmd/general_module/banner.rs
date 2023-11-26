use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::client::Http;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;
use serenity::model::Timestamp;

use crate::constant::COLOR;
use crate::error_enum::AppError::{FailedToGetUser, LangageGuildIdError};
use crate::error_enum::{AppError, COMMAND_SENDING_ERROR};
use crate::structure::embed::general::struct_lang_banner::BannerLocalisedText;
use crate::structure::register::general::struct_banner_register::RegisterLocalisedBanner;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            return banner_with_user(ctx, command, user).await;
        }
    }
    banner_without_user(ctx, command).await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let banners = RegisterLocalisedBanner::get_banner_register_localised().unwrap();
    let command = command
        .name("banner")
        .description("Get the banner")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the banner of")
                .kind(CommandOptionType::User)
                .required(false);
            for banner in banners.values() {
                option
                    .name_localized(&banner.code, &banner.option1)
                    .description_localized(&banner.code, &banner.option1_desc);
            }
            option
        });
    for banner in banners.values() {
        command
            .name_localized(&banner.code, &banner.name)
            .description_localized(&banner.code, &banner.description);
    }
    command
}

pub async fn no_banner(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    username: &String,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = BannerLocalisedText::get_banner_localised(guild_id).await?;

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{} {}", &localised_text.title, username))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .description(&localised_text.description)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

pub async fn banner_without_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = BannerLocalisedText::get_banner_localised(guild_id).await?;
    let user = command.user.id.0;
    let real_user = Http::get_user(&ctx.http, user).await;
    let result = real_user.map_err(|_| FailedToGetUser(String::from("Could no resolve user.")))?;
    let banner_url = match result.banner_url() {
        Some(banner) => banner,
        None => return no_banner(ctx, command, &result.name).await,
    };

    send_embed(ctx, command, localised_text.clone(), banner_url, result).await
}

pub async fn banner_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .0
        .to_string();
    let localised_text = BannerLocalisedText::get_banner_localised(guild_id).await?;
    let user = user_data.id.0;
    let real_user = Http::get_user(&ctx.http, user).await;
    let result = real_user.map_err(|_| FailedToGetUser(String::from("Could no resolve user.")))?;
    let banner_url = match result.banner_url() {
        Some(banner) => banner,
        None => return no_banner(ctx, command, &result.name).await,
    };
    send_embed(ctx, command, localised_text.clone(), banner_url, result).await
}

pub async fn send_embed(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: BannerLocalisedText,
    banner: String,
    result: User,
) -> Result<(), AppError> {
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, result.name))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(COLOR)
                            .image(banner)
                    })
                })
        })
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}
