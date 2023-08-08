use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

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

use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::BannerLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    return if let Some(option) = options.get(0) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::User(user, ..) = resolved {
            let result = banner_with_user(ctx, command, &user).await;
            result
        } else {
            let result = banner_without_user(ctx, command).await;
            result
        }
    } else {
        let result = banner_without_user(ctx, command).await;
        result
    };
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("banner")
        .description("Get the banner")
        .create_option(|option| {
            option
                .name("user")
                .description("The user you wan the banner of")
                .kind(CommandOptionType::User)
                .required(false)
        })
}

pub async fn no_banner(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    username: String,
) -> String {
    let color = Colour::FABLED_PINK;

    let mut file = File::open("lang_file/general/banner.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, BannerLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
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
            return format!("{}: {}", &localised_text.error_slash_command, why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}

pub async fn banner_without_user(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let mut file = File::open("lang_file/general/banner.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, BannerLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let user = command.user.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            return localised_text.error_no_user.clone();
        };
        let banner_url = &result.banner_url();
        let banner = if let Some(string) = banner_url {
            string
        } else {
            return no_banner(ctx, command, command.user.name.clone()).await;
        };

        let color = Colour::FABLED_PINK;

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
            return format!("{}: {}", &localised_text.error_slash_command, why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}

pub async fn banner_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) -> String {
    let mut file = File::open("lang_file/general/banner.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).expect("Failed to read file");

    let json_data: HashMap<String, BannerLocalisedText> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    let guild_id = command.guild_id.unwrap().0.to_string().clone();
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let user = user_data.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            return localised_text.error_no_user.clone();
        };
        let banner_url = &result.banner_url();
        let banner = if let Some(string) = banner_url {
            string
        } else {
            return no_banner(ctx, command, user_data.name.clone()).await;
        };

        let color = Colour::FABLED_PINK;

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
            return format!("{}: {}", &localised_text.error_slash_command, why);
        }
    } else {
        return "Language not found".to_string();
    }
    return "good".to_string();
}
