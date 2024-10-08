use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, PremiumCommand, PremiumCommandType, SlashCommand};
use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::event_handler::Handler;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::subcommand::{
    get_option_map_integer_subcommand, get_option_map_string_subcommand,
};
use crate::helper::image_saver::general_image_saver::image_saver;
use crate::structure::message::ai::image::{load_localization_image, ImageLocalised};
use prost::bytes::Bytes;
use prost::Message;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::{error, trace};
use uuid::Uuid;

pub struct ImageCommand<'de> {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub handler: &'de Handler,
    pub command_name: String,
}

impl Command for ImageCommand<'_> {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}
impl SlashCommand for ImageCommand<'_> {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        if self
            .check_hourly_limit(
                self.command_name.clone(),
                self.handler,
                PremiumCommandType::AIImage,
            )
            .await?
        {
            return Err(Box::new(error_dispatch::Error::Option(String::from(
                "You have reached your hourly limit. Please try again later.",
            ))));
        }
        let ctx = &self.ctx;
        let command_interaction = &self.command_interaction;
        let config = &self.config;

        let map = get_option_map_integer_subcommand(command_interaction);
        let n = *map.get(&String::from("n")).unwrap_or(&1);
        let data = get_value(command_interaction, n, config);
        send_embed(ctx, command_interaction, config, data, n).await
    }
}

fn get_value(command_interaction: &CommandInteraction, n: i64, config: &Arc<Config>) -> Value {
    let map = get_option_map_string_subcommand(command_interaction);
    let prompt = map
        .get(&String::from("description"))
        .unwrap_or(DEFAULT_STRING);
    let model = config.ai.image.ai_image_model.clone().unwrap_or_default();
    let model = model.as_str();
    let quality = config.ai.image.ai_image_style.clone();
    let style = config.ai.image.ai_image_quality.clone();
    let size = config
        .ai
        .image
        .ai_image_size
        .clone()
        .unwrap_or(String::from("1024x1024"));

    let data: Value = match (quality, style) {
        (Some(quality), Some(style)) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "quality": quality,
                "style": style,
                "response_format": "url"
            })
        }
        (None, Some(style)) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "style": style,
                "response_format": "url"
            })
        }
        (Some(quality), None) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "quality": quality,
                "response_format": "url"
            })
        }
        (None, None) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "response_format": "url"
            })
        }
    };
    data
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: &Arc<Config>,
    data: Value,
    n: i64,
) -> Result<(), Box<dyn Error>> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let image_localised = load_localization_image(guild_id.clone(), config.db.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.png", uuid_name);

    let token = config.ai.image.ai_image_token.clone().unwrap_or_default();
    let url = config
        .ai
        .image
        .ai_image_base_url
        .clone()
        .unwrap_or_default();
    // check the last 3 characters of the url if it v1/ or v1 or something else
    let url = if url.ends_with("v1/") {
        format!("{}images/generations", url)
    } else if url.ends_with("v1") {
        format!("{}/images/generations", url)
    } else {
        format!("{}/v1/images/generations", url)
    };

    let client = reqwest::Client::new();

    let token = token.as_str();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let url = url.as_str();
    let res = client.post(url).headers(headers).json(&data).send().await?;
    let res = res.json().await?;
    trace!(?res);
    let guild_id = match command_interaction.guild_id {
        Some(guild_id) => guild_id.to_string(),
        None => String::from("0"),
    };
    let bytes = get_image_from_response(
        res,
        config.image.save_image.clone(),
        config.image.save_server.clone(),
        config.image.token.clone(),
        guild_id,
    )
    .await?;

    if n == 1 {
        image_with_n_equal_1(
            image_localised,
            filename,
            command_interaction,
            ctx,
            bytes[0].clone(),
            config.image.save_image.clone(),
            config.image.save_server.clone(),
            config.image.token.clone(),
        )
        .await?
    } else {
        image_with_n_greater_than_1(image_localised, filename, command_interaction, ctx, bytes)
            .await?
    }
    Ok(())
}

async fn image_with_n_equal_1(
    image_localised: ImageLocalised,
    filename: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    bytes: Bytes,
    saver_server: String,
    token: Option<String>,
    save_type: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &filename))
        .title(image_localised.title);
    let guild_id = match command_interaction.guild_id {
        Some(guild_id) => guild_id.to_string(),
        None => String::from("0"),
    };
    let token = token.unwrap_or_default();
    let saver = save_type.unwrap_or_default();
    match image_saver(
        guild_id,
        filename.clone(),
        bytes.clone().encode_to_vec(),
        saver_server,
        token,
        saver,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => error!("Error saving image: {}", e),
    }
    let attachment = CreateAttachment::bytes(bytes.clone(), &filename);

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await?;
    Ok(())
}

async fn image_with_n_greater_than_1(
    image_localised: ImageLocalised,
    filename: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    bytes: Vec<Bytes>,
) -> Result<(), Box<dyn Error>> {
    let message = image_localised.title;
    let attachments: Vec<CreateAttachment> = bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let filename = format!("{}_{}.png", filename, index);
            CreateAttachment::bytes(byte.clone(), filename)
        })
        .collect();
    let builder_message = CreateInteractionResponseFollowup::new()
        .content(message)
        .files(attachments);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await?;
    Ok(())
}

async fn get_image_from_response(
    json: Value,
    saver_server: String,
    token: Option<String>,
    save_type: Option<String>,
    guild_id: String,
) -> Result<Vec<Bytes>, Box<dyn Error>> {
    let token = token.unwrap_or_default();
    let saver = save_type.unwrap_or_default();
    let mut bytes = Vec::new();
    let root: Root = match serde_json::from_value(json.clone()) {
        Ok(root) => root,
        Err(e) => {
            let root1: Root1 = serde_json::from_value(json)?;

            return Err(Box::new(error_dispatch::Error::Option(format!(
                "Error: {} ............ {:?}",
                e, root1.error
            ))));
        }
    };
    let urls: Vec<String> = root.data.iter().map(|data| data.url.clone()).collect();
    trace!("{:?}", urls);
    for (i, url) in urls.iter().enumerate() {
        let client = reqwest::Client::new();
        let res = client.get(url).send().await?;
        let body = res.bytes().await?;
        let filename = format!("ai_{}_{}.png", i, Uuid::new_v4());
        match image_saver(
            guild_id.clone(),
            filename.clone(),
            body.clone().encode_to_vec(),
            saver_server.clone(),
            token.clone(),
            saver.clone(),
        )
        .await
        {
            Ok(_) => (),
            Err(e) => error!("Error saving image: {}", e),
        }
        bytes.push(body);
    }
    Ok(bytes)
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "data")]
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
struct Data {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub param: Option<String>,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root1 {
    pub error: AiError,
}
