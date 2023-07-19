use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
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
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::in_progress::in_progress_embed;
use crate::cmd::general_module::lang_struct::ImageLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> String {
    let option = options
        .get(0)
        .expect("Expected username option")
        .resolved
        .as_ref()
        .expect("Expected username object");
    if let CommandDataOptionValue::String(description) = option {
        let result_diff = differed_response(ctx, command).await;

        if result_diff != "good".as_ref() {
            return result_diff;
        }

        let uuid_name = Uuid::new_v4();
        let filename = format!("{}.png", uuid_name);
        let filename_str = filename.as_str();

        let mut file = File::open("lang_file/ai/image.json").expect("Failed to open file");
        let mut json = String::new();
        file.read_to_string(&mut json).expect("Failed to read file");

        let json_data: HashMap<String, ImageLocalisedText> =
            serde_json::from_str(&json).expect("Failed to parse JSON");

        let guild_id = command.guild_id.unwrap().0.to_string().clone();
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let message: Message;
            match in_progress_embed(&ctx, &command).await {
                Ok(Some(message_option)) => {
                    message = message_option;
                }
                Ok(None) => {
                    return localised_text.unknown_error.clone();
                }
                Err(error) => {
                    println!("Error: {}", error);
                    return localised_text.error_slash_command.clone();
                }
            }


            let my_path = "./.env";
            let path = Path::new(my_path);
            let _ = dotenv::from_path(path);
            let prompt = description;
            let api_key = env::var("AI_API_TOKEN").expect("token");
            let api_base_url = env::var("AI_API_BASE_URL").expect("token");
            let api_url = format!("{}images/generations", api_base_url);
            let client = reqwest::Client::new();

            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

            let data = json!({
            "prompt": prompt,
            "n": 1,
            "size": "1024x1024"
        });

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

            let color = Colour::FABLED_PINK;
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
                return format!("{}: {}", &localised_text.error_slash_command, why);
            }
        } else {
            let _ = fs::remove_file(filename_str);
            return "Language not found".to_string();
        }
        let _ = fs::remove_file(filename_str);
    }
    return "good".to_string();
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
