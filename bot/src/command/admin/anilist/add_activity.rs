use anyhow::anyhow;
use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::command::command_trait::Embed;
use crate::command::command_trait::{Command, EmbedType, SlashCommand};
use crate::config::{Config, DbConfig};
use crate::get_url;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim_webhook;
use crate::structure::database::activity_data;
use crate::structure::database::activity_data::Column;
use crate::structure::database::prelude::ActivityData;
use crate::structure::message::admin::anilist::add_activity::load_localization_add_activity;
use crate::structure::run::anilist::minimal_anime::{
    Media, MediaTitle, MinimalAnimeId, MinimalAnimeIdVariables, MinimalAnimeSearch,
    MinimalAnimeSearchVariables,
};
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use base64::Engine as _;
use chrono::Utc;
use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{guess_format, GenericImageView, ImageFormat};
use moka::future::Cache;
use prost::bytes::Bytes;
use reqwest::get;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde_json::json;
use serenity::all::{ChannelId, CommandInteraction, Context, CreateAttachment, EditWebhook};
use tokio::sync::RwLock;
use tracing::trace;

pub struct AddActivityCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}
impl Command for AddActivityCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}
impl SlashCommand for AddActivityCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let anilist_cache = self.anilist_cache.clone();
        let command_interaction = self.command_interaction.clone();
        let ctx = self.ctx.clone();
        let config = self.config.clone();

        let map = get_option_map_string_subcommand_group(&command_interaction);
        let anime = map
            .get(&String::from("anime_name"))
            .cloned()
            .unwrap_or(String::new());
        let media = get_minimal_anime_media(anime.to_string(), anilist_cache).await?;

        let guild_id = match command_interaction.guild_id {
            Some(id) => id.to_string(),
            None => String::from("1"),
        };
        trace!(?guild_id);

        let add_activity_localised =
            load_localization_add_activity(guild_id.clone(), config.db.clone()).await?;

        let anime_id = media.id;

        let exist = check_if_activity_exist(anime_id, guild_id.clone(), config.db.clone()).await;

        self.defer().await?;
        let url = format!("https://anilist.co/anime/{}", media.id);

        let title = media
            .title
            .ok_or(anyhow!("No title for the media".to_string()))?;
        let anime_name = get_name(title);
        if exist {
            self.send_embed(
                Vec::new(),
                None,
                add_activity_localised.fail.clone(),
                add_activity_localised
                    .fail_desc
                    .replace("$anime$", anime_name.as_str()),
                None,
                Some(url),
                EmbedType::Followup,
                None,
            )
            .await?;
        } else {
            let channel_id = command_interaction.channel_id;

            let delay = map
                .get(&String::from("delay"))
                .unwrap_or(&String::from("0"))
                .parse()
                .unwrap_or(0);

            let trimmed_anime_name = if anime_name.len() >= 50 {
                trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
            } else {
                anime_name.clone()
            };

            let bytes = get(media.cover_image.ok_or(
                anyhow!("No cover image for this media".to_string()),
            )?.extra_large.
                unwrap_or(
                    "https://imgs.search.brave.com/ CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
                        .to_string()
                )
            ).await?.bytes().await?;

            let buf = resize_image(&bytes).await?;
            let base64 = STANDARD.encode(buf.into_inner());
            let image = format!("data:image/jpeg;base64,{}", base64);

            let next_airing = media.next_airing_episode.clone().ok_or(anyhow!(format!(
                "No next episode found for {} on anilist",
                anime_name
            )))?;
            let webhook = get_webhook(
                &ctx,
                channel_id,
                image.clone(),
                base64.clone(),
                trimmed_anime_name.clone(),
            )
            .await?;
            let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;
            let timestamp = next_airing.airing_at as i64;

            let chrono = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_default()
                .naive_utc();
            ActivityData::insert(activity_data::ActiveModel {
                anime_id: Set(media.id),
                timestamp: Set(chrono),
                server_id: Set(guild_id),
                webhook: Set(webhook),
                episode: Set(next_airing.episode),
                name: Set(trimmed_anime_name),
                delay: Set(delay),
                image: Set(image),
            })
            .exec(&connection)
            .await?;

            self.send_embed(
                Vec::new(),
                None,
                add_activity_localised.success.clone(),
                add_activity_localised
                    .success_desc
                    .replace("$anime$", anime_name.as_str()),
                None,
                Some(url),
                EmbedType::Followup,
                None,
            )
            .await?;
        }

        Ok(())
    }
}

async fn resize_image(image_bytes: &Bytes) -> Result<Cursor<Vec<u8>>, Box<dyn Error>> {
    let image = image::load_from_memory_with_format(image_bytes, guess_format(image_bytes)?)?;
    let (width, height) = image.dimensions();
    let (crop_x, crop_y, square_size) = calculate_crop_params(width, height);

    let resized_image = image
        .crop_imm(crop_x, crop_y, square_size, square_size)
        .resize_exact(128, 128, FilterType::Nearest);

    let mut buffer = Cursor::new(Vec::new());
    resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;

    Ok(buffer)
}

fn calculate_crop_params(width: u32, height: u32) -> (u32, u32, u32) {
    let square_size = width.min(height);
    let crop_x = (width - square_size) / 2;
    let crop_y = (height - square_size) / 2;
    (crop_x, crop_y, square_size)
}
async fn check_if_activity_exist(anime_id: i32, server_id: String, config: DbConfig) -> bool {
    let conn = match sea_orm::Database::connect(get_url(config.clone())).await {
        Ok(conn) => conn,
        Err(_) => return false,
    };

    let row = match ActivityData::find()
        .filter(Column::ServerId.eq(server_id))
        .filter(Column::AnimeId.eq(anime_id))
        .one(&conn)
        .await
    {
        Ok(row) => row,
        Err(_) => return false,
    };
    trace!(?row);

    row.is_some()
}

pub fn get_name(title: MediaTitle) -> String {
    let english_title = title.english;
    let romaji_title = title.romaji;

    let title = match (romaji_title, english_title) {
        (Some(romaji), Some(english)) => format!("{} / {}", english, romaji),
        (Some(romaji), None) => romaji,
        (None, Some(english)) => english,
        (None, None) => String::new(),
    };
    trace!(?title);
    title
}
async fn get_webhook(
    ctx: &Context,
    channel_id: ChannelId,
    image: String,
    base64: String,
    anime_name: String,
) -> Result<String, Box<dyn Error>> {
    trace!(?image);
    trace!(?anime_name);
    let webhook_info = json!({
        "avatar": image,
        "name": anime_name
    });

    let bot_id = ctx
        .http
        .get_current_application_info()
        .await?
        .id
        .to_string();
    trace!(?bot_id);

    let mut webhook_url = String::new();

    let webhooks = ctx.http.get_channel_webhooks(channel_id).await?;
    if webhooks.is_empty() {
        let webhook = ctx
            .http
            .create_webhook(channel_id, &webhook_info, None)
            .await?;
        webhook_url = webhook.url()?;
    } else {
        for webhook in webhooks {
            if webhook
                .user
                .clone()
                .ok_or(anyhow!("webhook user not found"))?
                .id
                .to_string()
                == bot_id
            {
                webhook_url = webhook.url()?;
                break;
            }
        }
        if webhook_url.is_empty() {
            let webhook = ctx
                .http
                .create_webhook(channel_id, &webhook_info, None)
                .await?;
            webhook_url = webhook.url()?;
        }
    }
    trace!(?webhook_url);

    let cursor = Cursor::new(base64);
    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes)?;

    let mut webhook = ctx.http.get_webhook_from_url(webhook_url.as_str()).await?;
    let attachment = CreateAttachment::bytes(decoded_bytes, "avatar");
    let edit_webhook = EditWebhook::new().name(anime_name).avatar(&attachment);
    webhook.edit(&ctx.http, edit_webhook).await?;

    Ok(webhook_url)
}
pub async fn get_minimal_anime_by_id(
    id: i32,
    cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    trace!(?id);
    let query = MinimalAnimeIdVariables { id: Some(id) };
    let operation = MinimalAnimeId::build(query);
    let response: GraphQlResponse<MinimalAnimeId> =
        make_request_anilist(operation, false, cache).await?;
    let media = response
        .data
        .ok_or(anyhow!("Error with request"))?
        .media
        .ok_or(anyhow!("No media found"))?;
    Ok(media)
}

async fn get_minimal_anime_by_search(
    query: &str,
    cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    trace!(?query);
    let search_query = MinimalAnimeSearchVariables {
        search: Some(query),
    };
    let operation = MinimalAnimeSearch::build(search_query);
    let response: GraphQlResponse<MinimalAnimeSearch> =
        make_request_anilist(operation, false, cache).await?;
    let media = response
        .data
        .ok_or(anyhow!("Error with request"))?
        .media
        .ok_or(anyhow!("No media found"))?;
    Ok(media)
}

pub async fn get_minimal_anime_media(
    anime: String,
    cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media, Box<dyn Error>> {
    let media = if let Ok(id) = anime.parse::<i32>() {
        get_minimal_anime_by_id(id, cache).await?
    } else {
        get_minimal_anime_by_search(&anime, cache).await?
    };
    trace!(?media);
    Ok(media)
}
