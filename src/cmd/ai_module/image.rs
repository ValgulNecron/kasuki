use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::channel::Message;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::cmd::general_module::differed_response::differed_response;
use crate::cmd::general_module::error_handling::{
    error_cant_read_file, error_file_not_found, error_making_request_edit, error_message,
    error_message_edit, error_message_followup, error_no_base_url_edit, error_no_guild_id,
    error_no_token_edit, error_parsing_json, error_parsing_json_edit, no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::in_progress::in_progress_embed;
use crate::cmd::general_module::lang_struct::ImageLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;

    let option = match options.get(0) {
        Some(data) => data,
        None => {
            error_message(color, ctx, command, &"Unable to get argument.".to_string()).await;
            return;
        }
    };
    let option = match option.resolved.as_ref() {
        Some(data) => data,
        None => {
            error_message(
                color,
                ctx,
                command,
                &"Unable to resolve argument value.".to_string(),
            )
            .await;
            return;
        }
    };
    if let CommandDataOptionValue::String(description) = option {
        let uuid_name = Uuid::new_v4();
        let filename = format!("{}.png", uuid_name);
        let filename_str = filename.as_str();

        let mut file = match File::open("lang_file/embed/ai/image.json") {
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

        let json_data: HashMap<String, ImageLocalisedText> = match serde_json::from_str(&json) {
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
            differed_response(ctx, command).await;

            let message: Message;
            match in_progress_embed(&ctx, &command).await {
                Ok(Some(message_option)) => {
                    message = message_option;
                }
                Ok(None) => {
                    error_message_followup(color, ctx, command, &localised_text.unknown_error)
                        .await;
                    return;
                }
                Err(error) => {
                    println!("Error: {}", error);
                    return;
                }
            }

            let my_path = "./.env";
            let path = Path::new(my_path);
            let _ = dotenv::from_path(path);
            let prompt = description;
            let api_key = match env::var("AI_API_TOKEN") {
                Ok(x) => x,
                Err(_) => {
                    error_no_token_edit(color, ctx, command, message).await;
                    return;
                }
            };
            let api_base_url = match env::var("AI_API_BASE_URL") {
                Ok(x) => x,
                Err(_) => {
                    error_no_base_url_edit(color, ctx, command, message).await;
                    return;
                }
            };
            let data;
            if let Ok(image_generation_mode) = env::var("IMAGE_GENERATION_MODELS_ON") {
                let is_ok = image_generation_mode.to_lowercase() == "true";
                if is_ok {
                    let model = match env::var("IMAGE_GENERATION_MODELS") {
                        Ok(data) => data,
                        Err(why) => {
                            error_message_edit(
                                color,
                                ctx,
                                command,
                                &format!("{}: {}", &localised_text.admin_instance_error, why),
                                message,
                            )
                            .await;
                            return;
                        }
                    };
                    data = json!({
                        "prompt": prompt,
                        "n": 1,
                        "size": "1024x1024",
                        "model": model,
                       "response_format": "url"
                    })
                } else {
                    data = json!({
                        "prompt": prompt,
                        "n": 1,
                        "size": "1024x1024",
                        "response_format": "url"
                    })
                }
            } else {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": "1024x1024",
                    "response_format": "url"
                })
            }
            let api_url = format!("{}images/generations", api_base_url);
            let client = reqwest::Client::new();

            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                match HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                    Ok(data) => data,
                    Err(why) => {
                        error_message_edit(color, ctx, command, &format!("{}", why), message).await;
                        return;
                    }
                },
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let res: Value = match client
                .post(api_url)
                .headers(headers)
                .json(&data)
                .send()
                .await
            {
                Ok(data) => match data.json().await {
                    Ok(data) => data,
                    Err(why) => {
                        println!("{}", why);
                        error_parsing_json_edit(color, ctx, message).await;
                        return;
                    }
                },
                Err(why) => {
                    println!("{}", why);
                    error_making_request_edit(color, ctx, command, message).await;
                    return;
                }
            };

            let mut url_string = "";
            if let Some(data) = res.get("data") {
                if let Some(object) = data.get(0) {
                    if let Some(url) = object.get("url") {
                        url_string = match url.as_str() {
                            Some(url) => url,
                            None => {
                                error_message_edit(
                                    color,
                                    ctx,
                                    command,
                                    &localised_text.no_url,
                                    message,
                                )
                                .await;
                                return;
                            }
                        }
                    }
                }
            }

            let mut real_message = message.clone();
            let response = match reqwest::get(url_string).await {
                Ok(data) => data,
                Err(why) => {
                    error_message_edit(color, ctx, command, &why.to_string(), message).await;
                    return;
                }
            };
            let bytes = match response.bytes().await {
                Ok(data) => data,
                Err(why) => {
                    error_message_edit(color, ctx, command, &why.to_string(), message).await;
                    return;
                }
            };
            match fs::write(filename.clone(), &bytes) {
                Ok(_) => {}
                Err(why) => {
                    error_message_edit(color, ctx, command, &why.to_string(), message).await;
                    return;
                }
            }

            let path = Path::new(filename_str);

            if let Err(why) = real_message
                .edit(&ctx.http, |m| {
                    m.attachment(path).embed(|e| {
                        e.title(&localised_text.title)
                            .image(format!("attachment://{}", filename))
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                })
                .await
            {
                let _ = fs::remove_file(filename_str);
                println!("Cannot respond to slash command: {}", why);
            }
        } else {
            let _ = fs::remove_file(filename_str);
            no_langage_error(color, ctx, command).await;
        }
        let _ = fs::remove_file(filename_str);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("image")
        .description("generate an image")
        .create_option(|option| {
            option
                .name("description")
                .description("Description of the image you want to generate.")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
