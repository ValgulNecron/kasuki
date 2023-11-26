use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::command::CommandOptionType::Attachment;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::Timestamp;
use uuid::Uuid;

use crate::constant::COLOR;
use crate::function::error_management::error_base_url::error_no_base_url_edit;
use crate::function::error_management::error_file::{error_file_extension, error_file_type};
use crate::function::error_management::error_request::error_making_request_edit;
use crate::function::error_management::error_resolving_value::error_resolving_value_followup;
use crate::function::error_management::error_token::error_no_token_edit;
use crate::function::general::differed_response::differed_response_with_file_deletion;
use crate::function::general::in_progress::in_progress_embed;
use crate::structure::embed::ai::struct_lang_transcript::TranscriptLocalisedText;
use crate::structure::register::ai::struct_transcript_register::RegisterLocalisedTranscript;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let mut prompt: String = "Do a transcript by first detecting the langage and then transcribing it in the detected langage".to_string();
    let mut lang: String = "en".to_string();
    let mut attachement: Option<serenity::model::prelude::Attachment> = None;
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
        if option.name == "video" {
            let resolved = option.resolved.as_ref().unwrap();
            if let CommandDataOptionValue::Attachment(attachement_option) = resolved {
                attachement = Some(attachement_option.clone())
            } else {
                return;
            }
        }
    }
    let attachement = match attachement {
        Some(att) => att,
        None => return,
    };
    let localised_text = match TranscriptLocalisedText::get_transcript_localised(ctx, command).await
    {
        Ok(data) => data,
        Err(_) => return,
    };
    let content_type = attachement.content_type.clone().unwrap();
    let content = attachement.proxy_url.clone();

    if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
        error_file_type(ctx, command).await;
        return;
    }

    let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"];
    let parsed_url = Url::parse(content.as_str()).expect("Failed to parse URL");
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
        error_file_extension(ctx, command).await;
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

    let message = match in_progress_embed(ctx, command).await {
        Ok(Some(message_option)) => message_option,
        Ok(None) => {
            error_resolving_value_followup(ctx, command).await;
            return;
        }
        Err(error) => {
            println!("Error: {}", error);
            return;
        }
    };

    let my_path = "./.env";
    let path = Path::new(my_path);
    let _ = dotenv::from_path(path);
    let api_key = match env::var("AI_API_TOKEN") {
        Ok(x) => x,
        Err(_) => {
            error_no_token_edit(ctx, command, message.clone()).await;
            return;
        }
    };
    let api_base_url = match env::var("AI_API_BASE_URL") {
        Ok(x) => x,
        Err(_) => {
            error_no_base_url_edit(ctx, command, message.clone()).await;
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
        .mime_str(content_type.as_str())
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
            eprintln!("Error sending the requests: {}", err);
            let _ = fs::remove_file(&file_to_delete);
            error_making_request_edit(ctx, command, message).await;
            return;
        }
    };
    let res_result: Result<Value, reqwest::Error> = response.json().await;

    let res = match res_result {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error parsing response as JSON: {}", err);
            let _ = fs::remove_file(&file_to_delete);
            error_making_request_edit(ctx, command, message.clone()).await;
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
                    .description(text.to_string())
                    .timestamp(Timestamp::now())
                    .color(COLOR)
            })
        })
        .await
    {
        println!("Cannot respond to slash command: {}", why);
    }
    let _ = fs::remove_file(&file_to_delete);
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let transcripts = RegisterLocalisedTranscript::get_transcript_register_localised().unwrap();
    let command = command
        .name("transcript")
        .description("generate a transcript")
        .create_option(|option| {
            let option = option
                .name("video")
                .description("File of the video you want the transcript of 25mb max.")
                .kind(Attachment)
                .required(true);
            for transcript in transcripts.values() {
                option
                    .name_localized(&transcript.code, &transcript.option1)
                    .description_localized(&transcript.code, &transcript.option1_desc);
            }
            option
        })
        .create_option(|option| {
            let option = option
                .name("prompt")
                .description(
                    "Use optional text to guide style or continue audio. Match audio language.",
                )
                .kind(CommandOptionType::String)
                .required(false);
            for transcript in transcripts.values() {
                option
                    .name_localized(&transcript.code, &transcript.option2)
                    .description_localized(&transcript.code, &transcript.option2_desc);
            }
            option
        })
        .create_option(|option| {
            let option = option
                .name("lang")
                .description("Input language in ISO-639-1 format improves accuracy and latency.")
                .kind(CommandOptionType::String)
                .required(false);
            for transcript in transcripts.values() {
                option
                    .name_localized(&transcript.code, &transcript.option3)
                    .description_localized(&transcript.code, &transcript.option3_desc);
            }
            option
        });
    for transcript in transcripts.values() {
        command
            .name_localized(&transcript.code, &transcript.name)
            .description_localized(&transcript.code, &transcript.desc);
    }
    command
}
