use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::http::client::Http;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;
use serenity::model::Timestamp;
use serenity::utils::Colour;

use crate::cmd::error_module::common::custom_error;
use crate::cmd::error_module::error_avatar::error_no_avatar;
use crate::cmd::lang_struct::embed::general::struct_lang_profile::ProfileLocalisedText;
use crate::cmd::lang_struct::register::general::struct_profile_register::RegisterLocalisedProfile;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    return if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            let result = profile_with_user(ctx, command, &user).await;
            result
        } else {
            let result = profile_without_user(ctx, command).await;
            result
        }
    } else {
        let result = profile_without_user(ctx, command).await;
        result
    };
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
            for (_key, profile) in &profiles {
                option
                    .name_localized(&profile.code, &profile.option1)
                    .description_localized(&profile.code, &profile.option1_desc);
            }
            option
        });
    for (_key, profile) in &profiles {
        command
            .name_localized(&profile.code, &profile.name)
            .description_localized(&profile.code, &profile.desc);
    }
    command
}

pub async fn profile_without_user(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    let localised_text =
        match ProfileLocalisedText::get_profile_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    let user = command.user.id.0;
    let real_user = Http::get_user(&ctx.http, user).await;
    let result = if let Ok(user) = real_user {
        user
    } else {
        custom_error(color, ctx, command, &localised_text.error_no_user).await;
        return;
    };
    let avatar_url = result.avatar_url().unwrap();

    let desc = description(result.clone(), command, localised_text.clone()).await;

    send_embed(
        avatar_url,
        desc,
        color,
        ctx,
        command,
        localised_text.clone(),
        result,
    )
    .await
}

pub async fn profile_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) {
    let color = Colour::FABLED_PINK;
    let localised_text =
        match ProfileLocalisedText::get_profile_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    let user = user_data.id.0;
    let real_user = Http::get_user(&ctx.http, user).await;
    let result = if let Ok(user) = real_user {
        user
    } else {
        custom_error(color, ctx, command, &localised_text.error_no_user).await;
        return;
    };

    let avatar_url = match result.avatar_url() {
        Some(url) => url,
        None => {
            error_no_avatar(color, ctx, command).await;
            return;
        }
    };

    let desc = description(result.clone(), command, localised_text.clone()).await;

    send_embed(
        avatar_url,
        desc,
        color,
        ctx,
        command,
        localised_text.clone(),
        result,
    )
    .await
}

pub async fn description(
    user: User,
    command: &ApplicationCommandInteraction,
    localised_text: ProfileLocalisedText,
) -> String {
    let is_bot = &user.bot;
    let public_flag = &user.public_flags.unwrap();
    let user_id = &user.id;
    let created_at = &user.created_at();
    let member = &command.member.clone().unwrap();
    let joined_at = member
        .joined_at
        .unwrap_or(Timestamp::from_unix_timestamp(0i64).unwrap());
    let desc = format!(
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
    );

    return desc;
}

pub async fn send_embed(
    avatar_url: String,
    desc: String,
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: ProfileLocalisedText,
    result: User,
) {
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, result.name))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                            .thumbnail(avatar_url)
                            .description(desc)
                    })
                })
        })
        .await
    {
        println!("{}: {}", "Error creating slash command", why);
    }
}
