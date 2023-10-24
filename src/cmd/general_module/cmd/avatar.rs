use crate::cmd::lang_struct::embed::general::struct_lang_avatar::AvatarLocalisedText;
use crate::cmd::lang_struct::register::general::struct_avatar_register::RegisterLocalisedAvatar;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{InteractionResponseType, User};
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
            avatar_with_user(ctx, command, user).await
        } else {
            avatar_without_user(ctx, command).await
        }
    } else {
        avatar_without_user(ctx, command).await
    }
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

async fn avatar_without_user(ctx: &Context, command: &ApplicationCommandInteraction) {}

async fn avatar_with_user(ctx: &Context, command: &ApplicationCommandInteraction, user: &User) {}

pub async fn send_embed(
    avatar_url: String,
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: AvatarLocalisedText,
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
                            .image(avatar_url)
                    })
                })
        })
        .await
    {
        println!("Error creating slash command: {}", why);
    }
}
