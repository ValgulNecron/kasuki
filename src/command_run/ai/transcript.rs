use crate::constant::{COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    DifferedCopyBytesError, DifferedFileExtensionError, DifferedFileTypeError,
    DifferedGettingBytesError, DifferedResponseError, DifferedTokenError, LangageGuildIdError,
    NoCommandOption,
};
use crate::lang_struct::ai::image::load_localization_image;
use crate::lang_struct::ai::transcript::load_localization_transcript;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    Attachment, CommandInteraction, Context, CreateAttachment, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, ResolvedOption,
    ResolvedValue, Timestamp,
};
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::{env, fs};
use tracing::log::trace;
use uuid::Uuid;

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut prompt: String = String::new();
    let mut lang: String = String::new();
    let mut attachement: Option<Attachment> = None;
    for option in options.iter().clone() {
        if option.name == "lang_struct" {
            let resolved = option.value.clone();
            if let ResolvedValue::String(lang_option) = resolved {
                lang = String::from(lang_option)
            }
        }
        if option.name == "prompt" {
            let resolved = option.value.clone();
            if let ResolvedValue::String(prompt_option) = resolved {
                prompt = String::from(prompt_option)
            }
        }
        if option.name == "video" {
            if let ResolvedOption {
                value: ResolvedValue::Attachment(attachment_option),
                ..
            } = option
            {
                attachement = Some(attachment_option.clone().clone())
            } else {
                return Err(NoCommandOption(String::from(
                    "The command contain no option.",
                )));
            }
        }
    }

    let attachement = match attachement {
        Some(att) => att,
        None => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )))
        }
    };

    let content_type = attachement
        .content_type
        .clone()
        .ok_or(OPTION_ERROR.clone())?;
    let content = attachement.proxy_url.clone();

    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let transcript_localised = load_localization_transcript(guild_id).await?;

    if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
        return Err(DifferedFileTypeError(String::from("Bad file type.")));
    }

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

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
        return Err(DifferedFileExtensionError(String::from(
            "Bad file extension",
        )));
    }

    let response = reqwest::get(content).await.expect("download");
    let uuid_name = Uuid::new_v4();
    let fname = Path::new("./").join(format!("{}.{}", uuid_name, file_extension));
    let file_name = format!("/{}.{}", uuid_name, file_extension);
    let mut file = File::create(fname.clone()).expect("file name");
    let resp_byte = response.bytes().await.map_err(|_| {
        DifferedGettingBytesError(String::from("Failed to get the bytes from the response."))
    })?;
    copy(&mut resp_byte.as_ref(), &mut file)
        .map_err(|_| DifferedCopyBytesError(String::from("Failed to copy bytes data.")))?;
    let file_to_delete = fname.clone();

    let my_path = "./.env";
    let path = Path::new(my_path);
    let _ = dotenv::from_path(path);
    let api_key = match env::var("AI_API_TOKEN") {
        Ok(x) => x,
        Err(_) => {
            return Err(DifferedTokenError(String::from(
                "There was an error while getting the token.",
            )))
        }
    };
    let api_base_url = match env::var("AI_API_BASE_URL") {
        Ok(x) => x,
        Err(_) => "https://api.openai.com/v1/".to_string(),
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
    let response = response_result.map_err(|_| {
        DifferedResponseError(String::from("Failed to get the response from the server."))
    })?;
    let res_result: Result<Value, reqwest::Error> = response.json().await;

    let res = res_result.map_err(|_| {
        DifferedResponseError(String::from("Failed to get the response from the server."))
    })?;

    let _ = fs::remove_file(&file_to_delete);
    trace!("{}", res);
    let text = res["text"].as_str().unwrap_or("");
    trace!("{}", text);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .title(transcript_localised.title)
        .description(text);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

    Ok(())
}
