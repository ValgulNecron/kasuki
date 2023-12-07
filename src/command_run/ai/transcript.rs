use crate::constant::{COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    DifferedFileExtensionError, DifferedFileTypeError, NoCommandOption,
};
use reqwest::Url;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    Attachment, CommandDataOption, CommandDataOptionValue, CommandInteraction, Context,
    CreateInteractionResponseMessage, ResolvedOption, ResolvedValue,
};

pub async fn run(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut prompt: String = String::new();
    let mut lang: String = String::new();
    let mut attachement: Option<Attachment> = None;
    for option in options.to_owned() {
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
                attachement = Some(attachment_option.to_owned())
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

    let content_type = attachement.content_type.clone().unwrap();
    let content = attachement.proxy_url.clone();

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

    Ok(())
}
