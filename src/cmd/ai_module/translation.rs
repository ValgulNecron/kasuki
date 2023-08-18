use std::collections::HashMap;
use std::fs::File;
use std::io::{copy, Read};
use std::path::Path;
use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{multipart, Url};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::channel::Message;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::utils::Colour;
use uuid::Uuid;

use crate::cmd::ai_module::translation_embed::translation_embed;
use crate::cmd::general_module::differed_response::differed_response_with_file_deletion;
use crate::cmd::general_module::error_handling::{
    error_cant_read_file, error_file_not_found, error_message, error_message_edit,
    error_message_followup, error_no_guild_id, error_parsing_json, no_langage_error,
};
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::in_progress::in_progress_embed;
use crate::cmd::general_module::lang_struct::TranslationLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;

    let mut lang: String = "en".to_string();
    let attachement_option;
    if options.get(0).expect("Expected attachement option").name == "video" {
        attachement_option = options
            .get(0)
            .expect("Expected attachement option")
            .resolved
            .as_ref()
            .expect("Expected attachement object");
    } else {
        attachement_option = options
            .get(1)
            .expect("Expected attachement option")
            .resolved
            .as_ref()
            .expect("Expected attachement object");
    }

    for option in options {
        if option.name == "lang" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(lang_option) = resolved {
                lang = lang_option.clone()
            } else {
                lang = "En".to_string();
            }
        }
    }

    if let CommandDataOptionValue::Attachment(attachement) = attachement_option {
        let mut file = match File::open("lang_file/embed/ai/translation.json.json") {
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

        let json_data: HashMap<String, TranslationLocalisedText> = match serde_json::from_str(&json)
        {
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
            let content_type = attachement.content_type.clone().unwrap();
            let content = attachement.proxy_url.clone();

            if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
                error_message(color, ctx, command, &localised_text.error_file_type).await;
                return;
            }

            let allowed_extensions = vec!["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"];
            let parsed_url = Url::parse(&*content).expect("Failed to parse URL");
            let path_segments = parsed_url
                .path_segments()
                .expect("Failed to retrieve path segments");
            let last_segment = path_segments.last().expect("URL has no path segments");

            let file_extension = last_segment
                .rsplit('.')
                .next()
                .expect("No file extension found")
                .to_lowercase();

            if !allowed_extensions.contains(&&*file_extension) {
                error_message(color, ctx, command, &localised_text.error_file_extension).await;
                return;
            }

            let response = reqwest::get(content).await.expect("download");

            let uuid_name = Uuid::new_v4();
            let fname = Path::new("./").join(format!("{}.{}", uuid_name, file_extension));
            let file_name = format!("/{}.{}", uuid_name, file_extension);
            let mut file = File::create(fname.clone()).expect("file name");
            let resp_byte = response.bytes().await.unwrap();
            copy(&mut resp_byte.as_ref(), &mut file).unwrap();
            let file_to_delete = fname.clone();

            differed_response_with_file_deletion(ctx, command, file_to_delete.clone()).await;

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
            let api_key = env::var("AI_API_TOKEN").expect("token");
            let api_base_url = env::var("AI_API_BASE_URL").expect("base url");
            let api_url = format!("{}audio/translations", api_base_url);
            let client = reqwest::Client::new();
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );

            let file = fs::read(fname).unwrap();
            let part = multipart::Part::bytes(file)
                .file_name(file_name)
                .mime_str(&*content_type)
                .unwrap();
            let form = multipart::Form::new()
                .part("file", part)
                .text("model", "whisper-1")
                .text("response_format", "json");

            let response_result = client
                .post(api_url)
                .headers(headers)
                .multipart(form)
                .send()
                .await;
            let response = match response_result {
                Ok(res) => res,
                Err(err) => {
                    let _ = fs::remove_file(&file_to_delete);
                    eprintln!("Error sending the request: {}", err);
                    error_message_edit(
                        color,
                        ctx,
                        command,
                        &format!("{}: {}", &localised_text.error_request, err),
                        message,
                    )
                    .await;
                    return;
                }
            };
            println!("{:?}", response);
            let res_result: Result<Value, reqwest::Error> = response.json().await;

            let res = match res_result {
                Ok(value) => value,
                Err(err) => {
                    let _ = fs::remove_file(&file_to_delete);
                    eprintln!("Error parsing response as JSON: {}", err);
                    error_message_edit(
                        color,
                        ctx,
                        command,
                        &format!("{}: {}", localised_text.error_request, err),
                        message,
                    )
                    .await;
                    return;
                }
            };

            let text = res["text"].as_str().unwrap_or("");
            let _ = fs::remove_file(&file_to_delete);
            if lang != "en" {
                let text = translation(lang, text.to_string(), api_key, api_base_url).await;
                translation_embed(ctx, text, message, localised_text.clone()).await;
                return;
            } else {
                translation_embed(ctx, text.to_string(), message, localised_text.clone()).await;
                return;
            }
        } else {
            no_langage_error(color, ctx, command).await
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("translation")
        .description("generate a translation")
        .create_option(|option| {
            option
                .name("video")
                .description("File of the video you want the translation of 25mb max.")
                .kind(Attachment)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("lang")
                .description("Lang in ISO-639-1 format.")
                .kind(CommandOptionType::String)
                .required(false)
        })
}

pub async fn translation(
    lang: String,
    text: String,
    api_key: String,
    api_base_url: String,
) -> String {
    let prompt_gpt = format!("
            i will give you a text and a ISO-639-1 code and you will translate it in the corresponding langage
            iso code: {}
            text:
            {}
            ", lang, text);

    let api_url = format!("{}chat/completions", api_base_url);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let data = json!({
         "model": "gpt-3.5-turbo-16k",
         "messages": [{"role": "system", "content": "You are a expert in translating and only do that."},{"role": "user", "content": prompt_gpt}]
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
    let content = res["choices"][0]["message"]["content"].to_string();
    let no_quote = content.replace("\"", "");
    let line_break = no_quote.replace("\\n", " \\n ");
    return line_break;
}
