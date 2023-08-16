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
    error_cant_read_file, error_file_not_found, error_message, error_no_base_url,
    error_no_guild_id, error_no_token, error_parsing_json, no_langage_error,
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

    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
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
                    error_message(color, ctx, command, &localised_text.unknown_error).await;
                    return;
                }
                Err(error) => {
                    println!("Error: {}", error);
                }
            }

            let my_path = "./.env";
            let path = Path::new(my_path);
            let _ = dotenv::from_path(path);
            let prompt = description;
            let api_key = match env::var("AI_API_TOKEN") {
                Ok(x) => x,
                Err(x) => {
                    error_no_token(color, ctx, command).await;
                    return;
                }
            };
            let api_base_url = match env::var("AI_API_BASE_URL") {
                Ok(x) => x,
                Err(x) => {
                    error_no_base_url(color, ctx, command).await;
                    return;
                }
            };
            let data;
            if let Ok(image_generation_mode) = env::var("IMAGE_GENERATION_MODELS_ON") {
                let is_ok = image_generation_mode.to_lowercase() == "true";
                if is_ok {
                    let model = env::var("IMAGE_GENERATION_MODELS").expect("model name");
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
                HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let res: Value = client
                .post(api_url)
                .headers(headers)
                .json(&data)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            println!("{}", res);

            let mut url_string = "";
            if let Some(data) = res.get("data") {
                if let Some(object) = data.get(0) {
                    if let Some(url) = object.get("url") {
                        url_string = url.as_str().unwrap();
                    }
                }
            }

            let mut real_message = message.clone();
            let response = reqwest::get(url_string).await.unwrap();
            let bytes = response.bytes().await.unwrap();
            fs::write(filename.clone(), &bytes).unwrap();

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
