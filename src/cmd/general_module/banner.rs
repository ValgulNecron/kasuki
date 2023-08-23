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

use crate::cmd::error::common::custom_error;
use crate::cmd::lang_struct::embed::general::struct_lang_banner::BannerLocalisedText;
use crate::cmd::lang_struct::register::general::struct_banner_register::BannerRegister;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    return if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            banner_with_user(ctx, command, &user).await;
        } else {
            banner_without_user(ctx, command).await;
        }
    } else {
        banner_without_user(ctx, command).await;
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let banners = BannerRegister::get_banner_register_localised().unwrap();
    let command = command
        .name("banner")
        .description("Get the banner")
        .create_option(|option| {
            let option = option
                .name("user")
                .description("The user you wan the banner of")
                .kind(CommandOptionType::User)
                .required(false);
            for (_key, banner) in &banners {
                option
                    .name_localized(&banner.code, &banner.option1)
                    .description_localized(&banner.code, &banner.option1_desc);
            }
            option
        });
    for (_key, banner) in &banners {
        command
            .name_localized(&banner.code, &banner.name)
            .description_localized(&banner.code, &banner.description);
    }
    command
}

pub async fn no_banner(ctx: &Context, command: &ApplicationCommandInteraction, username: String) {
    let color = Colour::FABLED_PINK;
    let localised_text =
        match BannerLocalisedText::get_in_progress_localised(color, ctx, command).await {
            Ok(data) => data,
            Err(_) => return,
        };

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.embed(|m| {
                        m.title(format!("{} {}", &localised_text.title, username))
                            // Add a timestamp for the current time
                            // This also accepts a rfc3339 Timestamp
                            .timestamp(Timestamp::now())
                            .color(color)
                            .description(&localised_text.description)
                    })
                })
        })
        .await
    {
        println!("{}: {}", &localised_text.error_slash_command, why);
    }
}

pub async fn banner_without_user(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    let localised_text =
        match BannerLocalisedText::get_in_progress_localised(color, ctx, command).await {
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
    let banner_url = &result.banner_url();
    let banner = if let Some(string) = banner_url {
        string
    } else {
        no_banner(ctx, command, command.user.name.clone()).await;
        return;
    };

    send_embed(color, ctx, command, localised_text.clone(), banner, result).await;
}

pub async fn banner_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) {
    let color = Colour::FABLED_PINK;

    let localised_text =
        match BannerLocalisedText::get_in_progress_localised(color, ctx, command).await {
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
    let banner_url = &result.banner_url();
    let banner = if let Some(string) = banner_url {
        string
    } else {
        no_banner(ctx, command, user_data.name.clone()).await;
        return;
    };

    send_embed(color, ctx, command, localised_text.clone(), banner, result).await;
}

pub async fn send_embed(
    color: Colour,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    localised_text: BannerLocalisedText,
    banner: &String,
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
                            .image(banner)
                    })
                })
        })
        .await
    {
        println!("{}: {}", &localised_text.error_slash_command, why);
    }
}
