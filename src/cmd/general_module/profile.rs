use crate::function::error_management::common::custom_error;
use crate::function::error_management::error_avatar::error_no_avatar;
use crate::structure::embed::general::struct_lang_profile::ProfileLocalisedText;
use crate::structure::register::general::struct_profile_register::RegisterLocalisedProfile;
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

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            send_embed(ctx, command, user.clone()).await;
            return;
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
) -> String {
    let is_bot = &user.bot;
    let public_flag = &user.public_flags.unwrap();
    let user_id = &user.id;
    let created_at = &user.created_at();
    let member = &command.member.clone().unwrap();
    let joined_at = member
        .joined_at
        .unwrap_or(Timestamp::from_unix_timestamp(0i64).unwrap());
    format!(
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
    )
}

async fn send_embed(ctx: &Context, command: &ApplicationCommandInteraction, user: User) {
    let color = Colour::FABLED_PINK;

    let localised_text =
        match ProfileLocalisedText::get_profile_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };
    let avatar_url = user.avatar_url();
    let desc = description(user, command, localised_text);
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{}{}", &localised_text.title, user.name))
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
        println!("Error creating slash command: {}", why);
    }
}
