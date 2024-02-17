use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;

use crate::command_run::get_option::{get_option_map_attachment, get_option_map_string};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{multipart, Url};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;
use uuid::Uuid;

use crate::constant::{
    CHAT_BASE_URL, CHAT_MODELS, CHAT_TOKEN, COLOR, DEFAULT_STRING, TRANSCRIPT_BASE_URL,
    TRANSCRIPT_MODELS, TRANSCRIPT_TOKEN,
};
use crate::error_management::command_error::CommandError::{FileError, Generic};
use crate::error_management::file_error::FileError::WrongContentType;
use crate::error_management::generic_error::GenericError::OptionError;
use crate::error_management::interaction_error::InteractionError;
use crate::lang_struct::ai::translation::load_localization_translation;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let map = get_option_map_string(command_interaction);
    let attachment_map = get_option_map_attachment(command_interaction);
    let lang = map
        .get(&String::from("prompt"))
        .unwrap_or(DEFAULT_STRING)
        .clone();
    let attachment = attachment_map.get(&String::from("video"));

    let attachment = match attachment {
        Some(Some(att)) => att,
        _ => {
            return Err(InteractionError::Command(Generic(OptionError(
                String::from("The command contain no attachment."),
            ))));
        }
    };

    let content_type = attachment
        .content_type
        .clone()
        .ok_or(InteractionError::Command(Generic(OptionError(
            String::from("Error getting content type"),
        ))))?;
    let content = attachment.proxy_url.clone();

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let translation_localised = load_localization_translation(guild_id).await?;

    if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
        return Err(InteractionError::Command(FileError(WrongContentType(
            String::from("Bad file type."),
        ))));
    }

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            InteractionError::Command((format!("Error while sending the command {}", e)))
        })?;

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
        return Err(DifferedError(FileExtensionError(String::from(
            "Bad file extension",
        ))));
    }

    let response = reqwest::get(content).await.expect("download");
    let uuid_name = Uuid::new_v4();
    let fname = Path::new("./").join(format!("{}.{}", uuid_name, file_extension));
    let file_name = format!("/{}.{}", uuid_name, file_extension);
    let mut file = FileError::create(fname.clone()).expect("file name");
    let resp_byte = response.bytes().await.map_err(|e| {
        DifferedError(GettingBytesError(format!(
            "Failed to get the bytes from the response. {}",
            e
        )))
    })?;
    copy(&mut resp_byte.as_ref(), &mut file)
        .map_err(|e| DifferedError(CopyBytesError(format!("Failed to copy bytes data. {}", e))))?;
    let file_to_delete = fname.clone();

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let token = unsafe { TRANSCRIPT_TOKEN.as_str() };

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let file = fs::read(fname).unwrap();
    let part = multipart::Part::bytes(file)
        .file_name(file_name)
        .mime_str(content_type.as_str())
        .unwrap();
    let model = unsafe { TRANSCRIPT_MODELS.as_str() };
    let form = multipart::Form::new()
        .part("file", part)
        .text("model", model)
        .text("language", lang.clone())
        .text("response_format", "json");

    let url = unsafe { format!("{}audio/transcriptions", TRANSCRIPT_BASE_URL.as_str()) };
    let response_result = client
        .post(url)
        .headers(headers)
        .multipart(form)
        .send()
        .await;
    let response = response_result.map_err(|e| {
        DifferedError(ResponseError(format!(
            "Failed to get the response from the server. {}",
            e
        )))
    })?;
    let res_result: Result<Value, reqwest::Error> = response.json().await;

    let res = res_result.map_err(|e| {
        DifferedError(ResponseError(format!(
            "Failed to get the response from the server. {}",
            e
        )))
    })?;

    let _ = fs::remove_file(&file_to_delete);
    trace!("{}", res);
    let text = res["text"].as_str().unwrap_or("");
    trace!("{}", text);

    let text = if lang != "en" {
        let api_key = unsafe { CHAT_TOKEN.clone() };
        let api_base_url = unsafe { CHAT_BASE_URL.clone() };
        let model = unsafe { CHAT_MODELS.clone() };
        translation(lang, text.to_string(), api_key, api_base_url, model).await?
    } else {
        String::from(text)
    };

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(translation_localised.title)
        .description(text);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            DifferedError(DifferedCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;

    Ok(())
}

pub async fn translation(
    lang: String,
    text: String,
    api_key: String,
    api_base_url: String,
    model: String,
) -> Result<String, AppError> {
    let prompt_gpt = format!("
            i will give you a text and a ISO-639-1 code and you will translate it in the corresponding language
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
         "model": model,
         "messages": [{"role": "system", "content": "You are a expert in translating and only do that."},{"role": "user", "content": prompt_gpt}]
    });

    let res: Value = client
        .post(api_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
        .map_err(|e| DifferedError(ResponseError(format!("error during translation. {}", e))))?
        .json()
        .await
        .map_err(|e| DifferedError(ResponseError(format!("error during translation. {}", e))))?;
    let content = res["choices"][0]["message"]["content"].to_string();
    let no_quote = content.replace('"', "");

    Ok(no_quote.replace("\\n", " \n "))
}
