use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::{env, fs};

use crate::constant::COLOR;
use crate::function::error_management::error_file::{error_file_extension, error_file_type};
use crate::function::error_management::error_parsing_json::error_parsing_json_edit;
use crate::function::error_management::error_request::error_making_request_edit;
use crate::function::error_management::error_resolving_value::error_resolving_value_followup;
use crate::function::general::differed_response::differed_response_with_file_deletion;
use crate::function::general::in_progress::in_progress_embed;
use crate::structure::embed::ai::struct_lang_translation::TranslationLocalisedText;
use crate::structure::register::ai::struct_translation_register::RegisterLocalisedTranslation;
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
use serenity::model::Timestamp;
use uuid::Uuid;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
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
        Some(a) => a,
        None => return,
    };

    let localised_text =
        match TranslationLocalisedText::get_translation_localised(ctx, command).await {
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
    let parsed_url = Url::parse(&content).expect("Failed to parse URL");
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

    let message: Message = match in_progress_embed(ctx, command).await {
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
        .mime_str(&content_type)
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
            eprintln!("Error sending the requests: {}", err);
            error_making_request_edit(ctx, command, message).await;
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
            error_parsing_json_edit(ctx, message, command).await;
            return;
        }
    };

    let text = res["text"].as_str().unwrap_or("");
    let _ = fs::remove_file(&file_to_delete);
    if lang != "en" {
        let text = translation(lang, text.to_string(), api_key, api_base_url).await;
        translation_embed(ctx, text, message, localised_text.clone()).await;
    } else {
        translation_embed(ctx, text.to_string(), message, localised_text.clone()).await;
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    let translations = RegisterLocalisedTranslation::get_translation_register_localised().unwrap();
    let command = command
        .name("translation")
        .description("generate a translation")
        .create_option(|option| {
            let option = option
                .name("video")
                .description("File of the video you want the translation of 25mb max.")
                .kind(Attachment)
                .required(true);
            for translation in translations.values() {
                option
                    .name_localized(&translation.code, &translation.option1)
                    .description_localized(&translation.code, &translation.option1_desc);
            }
            option
        })
        .create_option(|option| {
            let option = option
                .name("lang")
                .description("Lang in ISO-639-1 format.")
                .kind(CommandOptionType::String)
                .required(false);
            for translation in translations.values() {
                option
                    .name_localized(&translation.code, &translation.option2)
                    .description_localized(&translation.code, &translation.option2_desc);
            }
            option
        });
    for translation in translations.values() {
        command
            .name_localized(&translation.code, &translation.name)
            .description_localized(&translation.code, &translation.desc);
    }
    command
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
    let no_quote = content.replace('"', "");

    no_quote.replace("\\n", " \\n ")
}

pub async fn translation_embed(
    ctx: &Context,
    text: String,
    message: Message,
    localised_text: TranslationLocalisedText,
) {
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
        println!("Error creating slash command: {}", why);
    }
}
