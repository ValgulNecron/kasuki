use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, PremiumCommand, PremiumCommandType, SlashCommand};
use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::Handler;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand::{
    get_option_map_attachment_subcommand, get_option_map_string_subcommand,
};
use crate::structure::message::ai::transcript::load_localization_transcript;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{multipart, Url};
use serde_json::Value;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::trace;
use uuid::Uuid;

pub struct TranscriptCommand<'de> {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub handler: &'de Handler,
    pub command_name: String,
}

impl Command for TranscriptCommand<'_> {
    fn get_ctx(&self) -> &Context {

        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {

        &self.command_interaction
    }
}

impl SlashCommand for TranscriptCommand<'_> {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {

        if self
            .check_hourly_limit(
                self.command_name.clone(),
                self.handler,
                PremiumCommandType::AITranscript,
            )
            .await?
        {

            return Err(Box::new(error_dispatch::Error::Option(String::from(
                "You have reached your hourly limit. Please try again later.",
            ))));
        }

        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {

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

    let attachment =
        attachment_map
            .get(&String::from("video"))
            .ok_or(error_dispatch::Error::Option(String::from(
                "No option for video",
            )))?;

    let content_type = attachment
        .content_type
        .clone()
        .ok_or(error_dispatch::Error::File(String::from(
            "Failed to get the content type",
        )))?;

    let content = attachment.proxy_url.clone();

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let transcript_localised = load_localization_transcript(guild_id, config.db.clone()).await?;

    if !content_type.starts_with("audio/") && !content_type.starts_with("video/") {

        return Err(Box::new(error_dispatch::Error::File(String::from(
            "Unsupported file type",
        ))));
    }

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;

    let allowed_extensions = ["mp3", "mp4", "mpeg", "mpga", "m4a", "wav", "webm", "ogg"];

    let parsed_url = Url::parse(content.as_str())?;

    let path_segments = parsed_url
        .path_segments()
        .ok_or(error_dispatch::Error::File(String::from(
            "Failed to get the path segments",
        )))?;

    let last_segment = path_segments.last().unwrap_or_default();

    let file_extension = last_segment
        .rsplit('.')
        .next()
        .ok_or(error_dispatch::Error::File(String::from(
            "Failed to get the file extension",
        )))?
        .to_lowercase();

    if !allowed_extensions.contains(&&*file_extension) {

        return Err(Box::new(error_dispatch::Error::File(String::from(
            "Unsupported file extension",
        ))));
    }

    let response = reqwest::get(content).await?;

    // save the file into a buffer
    let buffer = response.bytes().await?;

    let uuid_name = Uuid::new_v4().to_string();

    let token = config
        .ai
        .transcription
        .ai_transcription_token
        .clone()
        .unwrap_or_default();

    let model = config
        .ai
        .transcription
        .ai_transcription_model
        .clone()
        .unwrap_or_default();

    let api_base_url = config
        .ai
        .transcription
        .ai_transcription_base_url
        .clone()
        .unwrap_or_default();

    // check the last 3 characters of the url if it v1/ or v1 or something else
    let url = if api_base_url.ends_with("v1/") {

        format!("{}audio/transcriptions/", api_base_url)
    } else if api_base_url.ends_with("v1") {

        format!("{}/audio/transcriptions/", api_base_url)
    } else {

        format!("{}/v1/audio/transcriptions/", api_base_url)
    };

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    let part = multipart::Part::bytes(buffer.to_vec())
        .file_name(uuid_name)
        .mime_str(content_type.as_str())?;

    let form = multipart::Form::new()
        .part("file", part)
        .text("model", model)
        .text("prompt", prompt)
        .text("language", lang)
        .text("response_format", "json");

    let response_result = client
        .post(url)
        .headers(headers)
        .multipart(form)
        .send()
        .await;

    let response = response_result?;

    trace!("{:?}", response);

    let res_result: Result<Value, reqwest::Error> = response.json().await;

    let res = res_result?;

    let text = res["text"].as_str().unwrap_or("");

    let builder_embed = get_default_embed(None)
        .title(transcript_localised.title)
        .description(text);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await?;

    Ok(())
}
