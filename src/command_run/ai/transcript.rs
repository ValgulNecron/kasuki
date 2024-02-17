use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;

use crate::command_run::get_option::{
    get_option_map_attachment, get_option_map_string, get_the_attachment,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::log::trace;
use uuid::Uuid;

use crate::constant::{
    COLOR, DEFAULT_STRING, TRANSCRIPT_BASE_URL, TRANSCRIPT_MODELS, TRANSCRIPT_TOKEN,
};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::ai::transcript::load_localization_transcript;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let attachment_map = get_option_map_attachment(command_interaction);
    let prompt = map
        .get(&String::from("lang"))
        .unwrap_or(DEFAULT_STRING)
        .clone();
    let lang = map
        .get(&String::from("prompt"))
        .unwrap_or(DEFAULT_STRING)
        .clone();
    let attachment = attachment_map.get(&String::from("video"));

    let attachment = get_the_attachment(attachment)?;

    let content_type = attachment.content_type.clone().ok_or(AppError::new(
        String::from("Error getting content type"),
        ErrorType::File,
        ErrorResponseType::Message,
    ))?;
    let content = attachment.proxy_url.clone();

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let transcript_localised = load_localization_transcript(guild_id).await?;

    if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {
        return Err(AppError::new(
            String::from("Bad file type."),
            ErrorType::File,
            ErrorResponseType::Message,
        ));
    }

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;

    let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm"];
    let parsed_url = Url::parse(content.as_str()).expect("Failed to parse URL");
    let path_segments = parsed_url.path_segments().ok_or(AppError::new(
        String::from("Failed to retrieve path segments"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;
    let last_segment = path_segments.last().unwrap_or_default();

    let file_extension = last_segment
        .rsplit('.')
        .next()
        .ok_or(AppError::new(
            String::from("No file extension found"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ))?
        .to_lowercase();

    if !allowed_extensions.contains(&&*file_extension) {
        return Err(AppError::new(
            String::from("Bad file extension"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ));
    }

    let response = reqwest::get(content).await.expect("download");
    let uuid_name = Uuid::new_v4();
    let fname = Path::new("./").join(format!("{}.{}", uuid_name, file_extension));
    let file_name = format!("/{}.{}", uuid_name, file_extension);
    let mut file = File::create(fname.clone()).expect("file name");
    let resp_byte = response.bytes().await.map_err(|e| {
        AppError::new(
            format!("Failed to get the bytes from the response. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    copy(&mut resp_byte.as_ref(), &mut file).map_err(|e| {
        AppError::new(
            format!("Failed to copy bytes data. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    let file_to_delete = fname.clone();

    let token = unsafe { TRANSCRIPT_TOKEN.as_str() };
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let file = fs::read(fname).unwrap();
    let part = multipart::Part::bytes(file)
        .file_name(file_name)
        .mime_str(content_type.as_str())
        .unwrap();
    let prompt = prompt;
    let model = unsafe { TRANSCRIPT_MODELS.as_str() };
    let form = multipart::Form::new()
        .part("file", part)
        .text("model", model)
        .text("prompt", prompt)
        .text("language", lang)
        .text("response_format", "json");

    let url = unsafe { format!("{}audio/transcriptions", TRANSCRIPT_BASE_URL.as_str()) };
    let response_result = client
        .post(url)
        .headers(headers)
        .multipart(form)
        .send()
        .await;
    let response = response_result.map_err(|e| {
        AppError::new(
            format!("Failed to get the response from the server. {}", e),
            ErrorType::Option,
            ErrorResponseType::Followup,
        )
    })?;
    let res_result: Result<Value, reqwest::Error> = response.json().await;

    let res = res_result.map_err(|e| {
        AppError::new(
            format!("Failed to get the response from the server. {}", e),
            ErrorType::Option,
            ErrorResponseType::Followup,
        )
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

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;

    Ok(())
}
