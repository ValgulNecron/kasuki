use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::{BotConfigDetails, Config};
use crate::structure::database::server_activity::{ServerActivityFull, SmallServerActivity};
use crate::database::dispatcher::data_dispatch::{get_one_activity, set_data_activity};
use crate::helper::create_default_embed::{get_anilist_anime_embed, get_default_embed};
use crate::helper::error_management::error_enum::{
    FollowupError, ResponseError, UnknownResponseError,
};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim_webhook;
use crate::structure::message::admin::anilist::add_activity::{
    load_localization_add_activity, AddActivityLocalised,
};
use crate::structure::run::anilist::minimal_anime::{
    Media, MediaTitle, MinimalAnimeId, MinimalAnimeIdVariables, MinimalAnimeSearch,
    MinimalAnimeSearchVariables,
};
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::Engine as _;
use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use moka::future::Cache;
use reqwest::get;
use serde_json::json;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateAttachment,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EditWebhook, GuildId,
};
use tokio::sync::RwLock;
use tracing::{error, trace};

pub struct AddActivityCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for AddActivityCommand {
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }

    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
}

impl SlashCommand for AddActivityCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let anilist_cache = self.anilist_cache.clone();
        let command_interaction = self.command_interaction.clone();
        let anime = get_minimal_anime_media(anilist_cache, &command_interaction).await?;
        send_embed(&self.ctx, &command_interaction, self.config.clone(), anime).await
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    media: Media,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    let add_activity_localised = load_localization_add_activity(
        guild_id.clone(),
        db_type.clone(),
        config.bot.config.clone(),
    )
    .await?;

    let anime_id = media.id;

    let exist = check_if_activity_exist(
        anime_id,
        guild_id.clone(),
        db_type.clone(),
        config.bot.config.clone(),
    )
    .await;
    if exist {
        already_exist(&add_activity_localised, media, command_interaction, ctx).await
    } else {
        success(
            &add_activity_localised,
            media,
            command_interaction,
            ctx,
            config,
        )
        .await
    }
}

async fn already_exist(
    add_activity_localised: &AddActivityLocalised,
    media: Media,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) -> Result<(), Box<dyn Error>> {
    let title = media
        .title
        .ok_or(FollowupError::Option("No title".to_string()))?;
    let anime_name = get_name(title);
    let builder_embed = get_default_embed(None)
        .title(&add_activity_localised.fail)
        .url(format!("https://anilist.co/anime/{}", media.id))
        .description(
            add_activity_localised
                .fail_desc
                .replace("$anime$", anime_name.as_str()),
        );

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| FollowupError::Sending(e.to_string()))?;

    Ok(())
}

async fn success(
    add_activity_localised: &AddActivityLocalised,
    media: Media,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let config = config.clone();
    let channel_id = command_interaction.channel_id;
    let title = media
        .title
        .ok_or(FollowupError::Option("No title".to_string()))?;
    let anime_name = get_name(title);
    let map = get_option_map_string_subcommand_group(command_interaction);
    let delay = map
        .get(&String::from("delay"))
        .unwrap_or(&String::from("0"))
        .parse()
        .unwrap_or(0);
    let trimed_anime_name = if anime_name.len() >= 50 {
        trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
    } else {
        anime_name.clone()
    };

    let anime_id = media.id;
    let guild_id = command_interaction
        .guild_id
        .unwrap_or(GuildId::from(0))
        .to_string();

    let bytes = get(media.cover_image.unwrap().extra_large.
        unwrap_or(
            "https://imgs.search.brave.com/ CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
                .to_string()
        )
    ).await.unwrap().bytes().await.unwrap();
    let mut img = image::load(
        Cursor::new(&bytes),
        guess_format(&bytes).map_err(|e| FollowupError::ImageProcessing(format!("{:#?}", e)))?,
    )
    .map_err(|e| FollowupError::Byte(format!("{:#?}", e)))?;
    let (width, height) = img.dimensions();
    let square_size = width.min(height);
    let crop_x = (width - square_size) / 2;
    let crop_y = (height - square_size) / 2;

    let img = img
        .crop(crop_x, crop_y, square_size, square_size)
        .resize_exact(128, 128, FilterType::Nearest);
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Jpeg)
        .expect("Failed to encode image");
    let base64 = STANDARD.encode(buf.into_inner());
    let image = format!("data:image/jpeg;base64,{}", base64);

    let next_airing = match media.next_airing_episode.clone() {
        Some(na) => na,
        None => {
            return Err(Box::new(FollowupError::Option(format!(
                "No next episode found for {} on anilist",
                anime_name
            ))));
        }
    };

    let webhook = get_webhook(ctx, channel_id, image, base64.clone(), trimed_anime_name).await?;

    set_data_activity(
        ServerActivityFull {
            anime_id,
            timestamp: next_airing.airing_at as i64,
            guild_id,
            webhook,
            episode: next_airing.episode,
            name: anime_name.clone(),
            delays: delay,
            image: base64,
        },
        db_type.clone(),
        config.bot.config.clone(),
    )
    .await?;

    let builder_embed = get_anilist_anime_embed(None, media.id)
        .title(&add_activity_localised.success)
        .description(
            add_activity_localised
                .success_desc
                .replace("$anime$", anime_name.as_str()),
        );

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| FollowupError::Sending(format!("{:#?}", e)))?;
    Ok(())
}

async fn check_if_activity_exist(
    anime_id: i32,
    server_id: String,
    db_type: String,
    db_config: BotConfigDetails,
) -> bool {
    let row: SmallServerActivity =
        get_one_activity(anime_id, server_id.clone(), db_type, db_config)
            .await
            .unwrap_or(SmallServerActivity {
                anime_id: None,
                timestamp: None,
                server_id: None,
            });
    if row.anime_id.is_none() || row.timestamp.is_none() || row.server_id.is_none() {
        return false;
    };
    true
}

/// This function gets the name of an anime from a `Title` struct.
///
/// It first checks if the English and Romaji titles exist. If they do, it concatenates them with a " / " separator.
/// If only one of them exists, it returns that one. If neither exist, it returns an empty string.
///
/// # Arguments
///
/// * `title` - A `Title` struct containing the English and Romaji titles of the anime.
///
/// # Returns
///
/// A string representing the name of the anime.
pub fn get_name(title: MediaTitle) -> String {
    let en = title.english.clone();
    let rj = title.romaji.clone();

    match (rj, en) {
        (Some(rj), Some(en)) => format!("{} / {}", en, rj),
        (Some(rj), None) => rj,
        (None, Some(en)) => en,
        (None, None) => String::new(),
    }
}

/// This asynchronous function gets or creates a webhook for a given channel.
///
/// It first checks if a webhook already exists for the channel. If it does, it returns the URL of the existing webhook.
/// If a webhook does not exist, it creates a new one with the given image and name, and returns its URL.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `channel_id` - The ID of the channel for which to get or create the webhook.
/// * `image` - The image to use for the webhook.
/// * `base64` - The base64 representation of the image.
/// * `anime_name` - The name to use for the webhook.
///
/// # Returns
///
/// A `Result` containing either the URL of the webhook if it is successfully retrieved or created, or an `AppError` if an error occurs.
async fn get_webhook(
    ctx: &Context,
    channel_id: ChannelId,
    image: String,
    base64: String,
    anime_name: String,
) -> Result<String, Box<dyn Error>> {
    let map = json!({
        "avatar": image,
        "name": anime_name
    });

    let bot_id = match ctx.http.get_current_application_info().await {
        Ok(bot_info) => bot_info.id.to_string(),
        Err(e) => {
            error!("{}", e);
            String::new()
        }
    };

    trace!(bot_id);
    let mut webhook_return = String::new();

    let webhooks = match ctx.http.get_channel_webhooks(channel_id).await {
        Ok(vec) => vec,
        Err(_) => {
            let webhook = ctx
                .http
                .create_webhook(channel_id, &map, None)
                .await
                .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;
            webhook_return = webhook
                .url()
                .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;

            return Ok(webhook_return);
        }
    };
    if webhooks.is_empty() {
        let webhook = ctx
            .http
            .create_webhook(channel_id, &map, None)
            .await
            .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;
        webhook_return = webhook
            .url()
            .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;

        return Ok(webhook_return);
    }
    for webhook in webhooks {
        trace!("{:#?}", webhook);
        let webhook_user_id = webhook.user.clone().unwrap().id.to_string();
        trace!(webhook_user_id);
        if webhook_user_id == bot_id {
            trace!("Getting webhook");
            webhook_return = webhook
                .url()
                .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;
        } else {
            trace!(webhook_return);
            let is_ok = webhook_return == String::new();
            trace!(is_ok);
            if is_ok {
                trace!("Creating webhook");
                let webhook = ctx
                    .http
                    .create_webhook(channel_id, &map, None)
                    .await
                    .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;
                webhook_return = webhook
                    .url()
                    .map_err(|e| FollowupError::Webhook(format!("{:#?}", e)))?;
            }
        }
    }
    trace!("Done");
    trace!(webhook_return);
    let cursor = Cursor::new(base64);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    // Read the decoded bytes into a Vec
    let mut decoded_bytes = Vec::new();
    match decoder.read_to_end(&mut decoded_bytes) {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            return Err(Box::new(FollowupError::Decoding(format!("{:#?}", e))));
        }
    }
    let mut webhook = match ctx.http.get_webhook_from_url(webhook_return.as_str()).await {
        Ok(webhook) => webhook,
        Err(e) => {
            error!("{}", e);
            return Err(Box::new(FollowupError::Webhook(format!("{:#?}", e))));
        }
    };
    let attachement = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new().name(anime_name).avatar(&attachement);
    match webhook.edit(&ctx.http, edit_webhook).await {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            return Err(Box::new(FollowupError::Webhook(format!("{:#?}", e))));
        }
    };

    Ok(webhook_return)
}

pub(crate) async fn get_minimal_anime_by_id(
    id: i32,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let query = MinimalAnimeIdVariables { id: Some(id) };
    let operation = MinimalAnimeId::build(query);
    let data: GraphQlResponse<MinimalAnimeId> =
        make_request_anilist(operation, false, anilist_cache).await?;
    Ok(match data.data {
        Some(data) => match data.media {
            Some(media) => media,
            None => {
                return Err(Box::new(UnknownResponseError::Option(
                    "No media found".to_string(),
                )))
            }
        },
        None => {
            return Err(Box::new(UnknownResponseError::Option(
                "No data found".to_string(),
            )))
        }
    })
}

async fn get_minimal_anime_by_search(
    value: &str,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let query = MinimalAnimeSearchVariables {
        search: Some(value),
    };
    let operation = MinimalAnimeSearch::build(query);
    let data: GraphQlResponse<MinimalAnimeSearch> =
        make_request_anilist(operation, false, anilist_cache).await?;
    Ok(match data.data {
        Some(data) => match data.media {
            Some(media) => media,
            None => {
                return Err(Box::new(UnknownResponseError::Option(
                    "No media found".to_string(),
                )))
            }
        },
        None => {
            return Err(Box::new(UnknownResponseError::Option(
                "No data found".to_string(),
            )))
        }
    })
}

pub async fn get_minimal_anime_media(
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    command_interaction: &CommandInteraction,
) -> Result<Media, Box<dyn Error>> {
    let map = get_option_map_string_subcommand_group(command_interaction);
    let anime = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());
    let media = if anime.parse::<i32>().is_ok() {
        get_minimal_anime_by_id(anime.parse::<i32>().unwrap(), anilist_cache).await?
    } else {
        get_minimal_anime_by_search(anime.as_str(), anilist_cache).await?
    };
    Ok(media)
}
