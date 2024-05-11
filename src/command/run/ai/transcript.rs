use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::log::trace;
use uuid::Uuid;

use crate::helper::create_normalise_embed::get_default_embed;
use crate::helper::get_option::subcommand::{
    get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use crate::constant::{DEFAULT_STRING, TRANSCRIPT_BASE_URL, TRANSCRIPT_MODELS, TRANSCRIPT_TOKEN};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::ai::transcript::load_localization_transcript;

/// This asynchronous function runs the command interaction for transcribing an audio or video file.
///
/// It first retrieves the language and prompt for the transcription from the command interaction options.
/// It also retrieves the attachment to be transcribed.
///
/// It checks the content type of the attachment and returns an error if it is not an audio or video file.
///
/// It sends a deferred response to the command interaction.
///
/// It downloads the attachment and saves it to a local file.
///
/// It sends a request to the OpenAI API to transcribe the file.
///
/// It retrieves the transcription from the API response and sends a followup message with the transcription.
///
/// It handles any errors that occur during the process and returns an `AppError` if an error occurs.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let attachment_map = get_option_map_attachment_subcommand(command_interaction);
    let prompt = map
        .get(&String::from("lang"))
        .unwrap_or(DEFAULT_STRING)
        .clone();
    let lang = map
        .get(&String::from("prompt"))
        .unwrap_or(DEFAULT_STRING)
        .clone();
    let attachment = attachment_map
        .get(&String::from("video"))
        .ok_or(AppError::new(
            String::from("There is no attachment"),
            ErrorType::Option,
            ErrorResponseType::Message,
        ))?;

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

    let token = TRANSCRIPT_TOKEN;
    let token = token.as_str();
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
    let model = TRANSCRIPT_MODELS.to_string();
    let form = multipart::Form::new()
        .part("file", part)
        .text("model", model)
        .text("prompt", prompt)
        .text("language", lang)
        .text("response_format", "json");

    let url = format!("{}transcriptions", TRANSCRIPT_BASE_URL.as_str());
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
    trace!("{:?}", response);
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

    let builder_embed = get_default_embed(None)
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
