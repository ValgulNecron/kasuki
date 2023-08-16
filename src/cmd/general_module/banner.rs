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

use crate::cmd::general_module::error_handling::{
    error_cant_read_file, error_file_not_found, error_message, error_no_guild_id,
    error_parsing_json, no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::lang_struct::BannerLocalisedText;

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

pub async fn no_banner(ctx: &Context, command: &ApplicationCommandInteraction, username: String) {
    let color = Colour::FABLED_PINK;

    let mut file = match File::open("lang_file/embed/general/banner.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, BannerLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
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
        }
    } else {
        no_langage_error(color, ctx, command).await;
    }
}

pub async fn banner_without_user(ctx: &Context, command: &ApplicationCommandInteraction) {
    let color = Colour::FABLED_PINK;

    let mut file = match File::open("lang_file/embed/general/banner.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, BannerLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let user = command.user.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            error_message(color, ctx, command, &localised_text.error_no_user).await;
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
    } else {
        no_langage_error(color, ctx, command).await;
    }
}

pub async fn banner_with_user(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    user_data: &User,
) {
    let color = Colour::FABLED_PINK;

    let mut file = match File::open("lang_file/embed/general/banner.json") {
        Ok(file) => file,
        Err(_) => {
            error_file_not_found(color, ctx, command).await;
            return;
        }
    };
    let mut json = String::new();
    match file.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => error_cant_read_file(color, ctx, command).await,
    }

    let json_data: HashMap<String, BannerLocalisedText> = match serde_json::from_str(&json) {
        Ok(data) => data,
        Err(_) => {
            error_parsing_json(color, ctx, command).await;
            return;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id.0.to_string(),
        None => {
            error_no_guild_id(color, ctx, command).await;
            return;
        }
    };
    let lang_choice = get_guild_langage(guild_id).await;

    if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
        let user = user_data.id.0;
        let real_user = Http::get_user(&ctx.http, user).await;
        let result = if let Ok(user) = real_user {
            user
        } else {
            error_message(color, ctx, command, &localised_text.error_no_user).await;
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
    } else {
        no_langage_error(color, ctx, command).await;
    }
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
