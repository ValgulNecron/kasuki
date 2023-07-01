use std::{env, fs};
use std::fs::File;
use std::io::{copy, empty, Write};
use std::path::Path;
use std::str::Bytes;

use reqwest::{multipart, Url};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::futures::AsyncWriteExt;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::AttachmentType;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::Timestamp;
use serenity::utils::Colour;
use uuid::Uuid;

pub async fn run(options: &[CommandDataOption], ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let option = options
        .get(0)
        .expect("Expected attachement option")
        .resolved
        .as_ref()
        .expect("Expected attachement object");
    let mut prompt: String;
    let mut lang: String;
    if let Some(option) = options.get(1) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::String(prompt_op) = resolved {
            prompt = prompt_op.clone();
        } else {
            return "error".to_string();
        }
        if !(option.name == "prompt") {
            lang = prompt.clone();
            prompt = "Do a transcript by first detecting the langage and then transcribing it in the detected langage".to_string();
        }
    } else {
        prompt = "Do a transcript by first detecting the langage and then transcribing it in the detected langage".to_string();
    }
    if let Some(option) = options.get(2) {
        let resolved = option.resolved.as_ref().unwrap();
        if let CommandDataOptionValue::String(lang_op) = resolved {
            lang = lang_op.clone();
        } else {
            return "error".to_string();
        }
        if !(option.name == "lang") {
            prompt = lang.clone();
            lang = "en".to_string();
        }
    } else {
        lang = "en".to_string();
    }
    if let CommandDataOptionValue::Attachment(attachement) = option {
        let content_type = attachement.content_type.clone().unwrap();
        let content = attachement.proxy_url.clone();

        if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
            return "wrong file type".to_string();
        }

        let allowed_extensions = vec![
            "mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm",
        ];
        let parsed_url = Url::parse(&*content).expect("Failed to parse URL");
        let path_segments = parsed_url.path_segments().expect("Failed to retrieve path segments");
        let last_segment = path_segments.last().expect("URL has no path segments");

        let file_extension = last_segment
            .rsplit('.')
            .next()
            .expect("No file extension found")
            .to_lowercase();

        if !allowed_extensions.contains(&&**&file_extension) {
            return "wrong file extension".to_string();
        }

        let response = reqwest::get(content).await.expect("download");
        let uuid_name = Uuid::new_v4();
        let fname = Path::new("./").join(format!("{}.{}", uuid_name, file_extension));
        let file_name = format!("/{}.{}", uuid_name, file_extension);
        let mut string_fname = format!("/{}.{}", uuid_name, file_extension);
        let mut file = File::create(fname.clone()).expect("file name");
        let mut resp_byte = response.bytes().await.unwrap();
        copy(&mut resp_byte.as_ref(), &mut file).unwrap();
        let color = Colour::FABLED_PINK;
        let file_to_delete = fname.clone();

        if let Ok(current_dir) = env::current_dir() {
            let current_dir_str = current_dir.display().to_string();
            let current_dir_str = current_dir_str.replace("\\", "/");
            string_fname = format!("{}{}", current_dir_str, string_fname)
        } else {
            println!("Failed to retrieve the current directory.");
        }

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await
        {
            std::fs::remove_file(&file_to_delete);
            println!("Cannot respond to slash command: {}", why);
        }

        let message = command
            .create_followup_message(&ctx.http, |f| {
                f.embed(|e| {
                    e.title("In progress")
                        .description("The task is being processed...be patient, it may take some time!")
                        .timestamp(Timestamp::now())
                        .color(color)
                })
            })
            .await;

        let my_path = "./src/.env";
        let path = std::path::Path::new(my_path);
        dotenv::from_path(path);
        let api_key = env::var("AI_API_TOKEN").expect("token");
        let api_base_url = env::var("AI_API_BASE_URL").expect("token");
        let api_url = format!("{}audio/transcriptions", api_base_url);
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());


        let file = fs::read(fname).unwrap();
        let part = multipart::Part::bytes(file).file_name(file_name)
            .mime_str(&*content_type).unwrap();
        let prompt = prompt;
        let form = multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-1")
            .text("prompt", prompt)
            .text("language", lang);

        let data = json!({
            "model": "whisper-1"
        });

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
                std::fs::remove_file(&file_to_delete);
                return format!("Error sending the request: {}", err);
            }
        };
        let res_result: Result<Value, reqwest::Error> = response.json().await;

        let res = match res_result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error parsing response as JSON: {}", err);
                std::fs::remove_file(&file_to_delete);
                return format!("Error sending the request: {}", err);
            }
        };

        std::fs::remove_file(&file_to_delete);

        let text = res["text"].as_str().unwrap_or("");
        let mut real_message = message.unwrap();
        real_message.edit(&ctx.http, |m|
            m.embed((|e| {
                e.title("Here your transcript")
                    .description(format!("{}", text))
                    .timestamp(Timestamp::now())
                    .color(color)
            })
            )).await.expect("TODO");
    }
    return "good".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("transcript").description("generate a transcript").create_option(
        |option| {
            option
                .name("video")
                .description("File of the video you want the transcript of 25mb max.")
                .kind(Attachment)
                .required(true)
        },
    ).create_option(
        |option| {
            option
                .name("prompt")
                .description("Use optional text to guide style or continue audio. Match audio language.")
                .kind(CommandOptionType::String)
                .required(false)
        }
    ).create_option(
        |option| {
            option
                .name("lang")
                .description("Input language in ISO-639-1 format improves accuracy and latency.")
                .kind(CommandOptionType::String)
                .required(false)
        }
    )
}