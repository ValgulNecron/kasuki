use std::collections::HashMap;
use std::fs::File;
use std::io::{copy, Read};
use std::path::Path;
use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::channel::Message;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::cmd::error::common::{custom_error, custom_error_edit, custom_followup_error};
use crate::cmd::error::error_base_url::error_no_base_url_edit;
use crate::cmd::error::error_token::error_no_token_edit;
use crate::cmd::error::no_lang_error::{
    error_cant_read_langage_file, error_langage_file_not_found, error_no_langage_guild_id,
    error_parsing_langage_json, no_langage_error,
};
use crate::cmd::general_module::differed_response::differed_response_with_file_deletion;
use crate::cmd::general_module::get_guild_langage::get_guild_langage;
use crate::cmd::general_module::in_progress::in_progress_embed;
use crate::cmd::general_module::lang_struct::TranscriptLocalisedText;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let color = Colour::FABLED_PINK;
    let attachement_option;
    if options.get(0).expect("Expected attachement option").name == "video" {
        attachement_option = options
            .get(0)
            .expect("Expected attachement option")
            .resolved
            .as_ref()
            .expect("Expected attachement object");
    } else if options.get(1).expect("Expected attachement option").name == "video" {
        attachement_option = options
            .get(1)
            .expect("Expected attachement option")
            .resolved
            .as_ref()
            .expect("Expected attachement object");
    } else {
        attachement_option = options
            .get(2)
            .expect("Expected attachement option")
            .resolved
            .as_ref()
            .expect("Expected attachement object");
    }
    let mut prompt: String = "Do a transcript by first detecting the langage and then transcribing it in the detected langage".to_string();
    let mut lang: String = "en".to_string();
    for option in options {
        if option.name == "lang" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(lang_option) = resolved {
                lang = lang_option.clone()
            } else {
                lang = "En".to_string();
            }
        }
        if option.name == "prompt" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::String(prompt_option) = resolved {
                prompt = prompt_option.clone()
            } else {
                prompt = "Do a transcript by first detecting the langage and then transcribing it in the detected langage".to_string();
            }
        }
    }
    if let CommandDataOptionValue::Attachment(attachement) = attachement_option {
        let mut file = match File::open("lang_file/embed/ai/transcript.json") {
            Ok(file) => file,
            Err(_) => {
                error_langage_file_not_found(color, ctx, command).await;
                return;
            }
        };
        let mut json = String::new();
        match file.read_to_string(&mut json) {
            Ok(_) => {}
            Err(_) => error_cant_read_langage_file(color, ctx, command).await,
        }

        let json_data: HashMap<String, TranscriptLocalisedText> = match serde_json::from_str(&json)
        {
            Ok(data) => data,
            Err(_) => {
                error_parsing_langage_json(color, ctx, command).await;
                return;
            }
        };

        let guild_id = match command.guild_id {
            Some(id) => id.0.to_string(),
            None => {
                error_no_langage_guild_id(color, ctx, command).await;
                return;
            }
        };
        let lang_choice = get_guild_langage(guild_id).await;

        if let Some(localised_text) = json_data.get(lang_choice.as_str()) {
            let content_type = attachement.content_type.clone().unwrap();
            let content = attachement.proxy_url.clone();

            if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
                custom_error(color, ctx, command, &localised_text.error_file_type).await;
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

            if !allowed_extensions.contains(&&**&file_extension) {
                if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
                    custom_error(color, ctx, command, &localised_text.error_file_extension).await;
                    return;
                }
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
                    custom_followup_error(color, ctx, command, &localised_text.unknown_error).await;
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
            let api_key = match env::var("AI_API_TOKEN") {
                Ok(x) => x,
                Err(_) => {
                    error_no_token_edit(color, ctx, command, message.clone()).await;
                    return;
                }
            };
            let api_base_url = match env::var("AI_API_BASE_URL") {
                Ok(x) => x,
                Err(_) => {
                    error_no_base_url_edit(color, ctx, command, message.clone()).await;
                    return;
                }
            };
            let api_url = format!("{}audio/transcriptions", api_base_url);
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
            let prompt = prompt;
            let form = multipart::Form::new()
                .part("file", part)
                .text("model", "whisper-1")
                .text("prompt", prompt)
                .text("language", lang)
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
                    eprintln!("Error sending the request: {}", err);
                    let _ = fs::remove_file(&file_to_delete);
                    custom_error_edit(
                        color,
                        ctx,
                        command,
                        &format!("{}: {}", &localised_text.error_request, err),
                        message.clone(),
                    )
                    .await;
                    return;
                }
            };
            let res_result: Result<Value, reqwest::Error> = response.json().await;

            let res = match res_result {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("Error parsing response as JSON: {}", err);
                    let _ = fs::remove_file(&file_to_delete);
                    custom_error_edit(
                        color,
                        ctx,
                        command,
                        &format!("{}: {}", &localised_text.error_request, err),
                        message.clone(),
                    )
                    .await;
                    return;
                }
            };

            let _ = fs::remove_file(&file_to_delete);

            let text = res["text"].as_str().unwrap_or("");
            let mut real_message = message.clone();

            if let Err(why) = real_message
                .edit(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(&localised_text.title)
                            .description(format!("{}", text))
                            .timestamp(Timestamp::now())
                            .color(color)
                    })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
            let _ = fs::remove_file(&file_to_delete);
        } else {
            no_langage_error(color, ctx, command).await;
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("transcript")
        .description("generate a transcript")
        .create_option(|option| {
            option
                .name("video")
                .description("File of the video you want the transcript of 25mb max.")
                .kind(Attachment)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("prompt")
                .description(
                    "Use optional text to guide style or continue audio. Match audio language.",
                )
                .kind(CommandOptionType::String)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("lang")
                .description("Input language in ISO-639-1 format improves accuracy and latency.")
                .kind(CommandOptionType::String)
                .required(false)
        })
}
