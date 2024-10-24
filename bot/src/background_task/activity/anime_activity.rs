use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::Duration;

use crate::command::admin::anilist::add_activity::get_minimal_anime_media;
use crate::config::DbConfig;
use crate::database::activity_data;
use crate::database::activity_data::Model;
use crate::database::prelude::ActivityData;
use crate::get_url;
use crate::helper::create_default_embed::get_default_embed;
use crate::structure::message::anilist_user::send_activity::load_localization_send_activity;
use anyhow::{Context, Error, Result};
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use sea_orm::ActiveValue::Set;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use sea_orm::{ColumnTrait, DeleteResult};
use serenity::builder::{CreateAttachment, EditWebhook, ExecuteWebhook};
use serenity::model::webhook::Webhook;
use serenity::prelude::Context as SerenityContext;
use tokio::sync::RwLock;
use tracing::{error, instrument, trace};

pub async fn manage_activity(
    ctx: SerenityContext,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) {
    send_activity(&ctx, anilist_cache, db_config).await;
}

async fn send_activity(
    ctx: &SerenityContext,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) {
    let now = Utc::now().naive_utc();

    let connection = match sea_orm::Database::connect(get_url(db_config.clone())).await {
        Ok(connection) => connection,
        Err(e) => {
            error!("{}", e);

            return;
        }
    };

    let rows = match ActivityData::find()
        .filter(<activity_data::Entity as EntityTrait>::Column::Timestamp.eq(now))
        .all(&connection)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("{}", e);

            return;
        }
    };

    for row in rows {
        if now != row.timestamp {
            continue;
        }

        let guild_id = row.server_id.clone();

        if row.delay != 0 {
            let anilist_cache = anilist_cache.clone();

            let db_config = db_config.clone();

            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(row.delay as u64)).await;

                if let Err(e) =
                    send_specific_activity(&row, guild_id, &ctx_clone, anilist_cache, db_config)
                        .await
                {
                    error!("{}", e)
                }
            });
        } else {
            let anilist_cache = anilist_cache.clone();

            let db_config = db_config.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    send_specific_activity(&row, guild_id, &ctx, anilist_cache, db_config).await
                {
                    error!("{}", e);
                }
            });
        }
    }
}

#[instrument(skip(ctx, anilist_cache, db_config))]

async fn send_specific_activity(
    row: &Model,
    guild_id: String,
    ctx: &SerenityContext,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) -> Result<()> {
    let localised_text = load_localization_send_activity(guild_id, db_config.clone()).await?;

    let mut webhook = Webhook::from_url(&ctx.http, &row.webhook).await?;

    trace!("Decoding image");

    let decoded_bytes = decode_image(&row.image)?;

    let trimmed_name = row.name.chars().take(100).collect::<String>();

    let attachment = CreateAttachment::bytes(decoded_bytes, "avatar");

    let edit_webhook = EditWebhook::new().name(trimmed_name).avatar(&attachment);

    webhook.edit(&ctx.http, edit_webhook).await?;

    let embed = get_default_embed(None)
        .description(
            localised_text
                .desc
                .replace("$ep$", &row.episode.to_string())
                .replace("$anime$", &row.name),
        )
        .url(format!("https://anilist.co/anime/{}", row.anime_id))
        .title(&localised_text.title);

    let builder_message = ExecuteWebhook::new().embed(embed);

    webhook.execute(&ctx.http, false, builder_message).await?;

    tokio::spawn(async move {
        if let Err(e) = update_info(row, &*guild_id, anilist_cache, db_config).await {
            error!("Failed to update info: {}", e);
        }
    });

    Ok(())
}

fn decode_image(image: &str) -> Result<Vec<u8>> {
    let cursor = Cursor::new(image);

    let mut decoder = DecoderReader::new(cursor, &STANDARD);

    let mut decoded_bytes = Vec::new();

    decoder.read_to_end(&mut decoded_bytes)?;

    Ok(decoded_bytes)
}

async fn update_info(
    row: &Model,
    guild_id: &str,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
    db_config: DbConfig,
) -> Result<()> {
    let media = get_minimal_anime_media(row.anime_id.to_string(), anilist_cache).await;
    let media = media?;

    let next_airing = media.next_airing_episode.ok_or_else(|| {
        trace!("No next airing episode for anime_id: {}", row.anime_id);

        remove_activity(row, guild_id, db_config.clone())
    })?;

    let title = media.title.ok_or(Err("No title"))?;

    let name = title
        .english
        .or(title.romaji)
        .unwrap_or_else(|| "Unknown".to_string());

    let connection: DatabaseConnection = sea_orm::Database::connect(get_url(db_config)).await?;

    let timestamp = DateTime::<Utc>::from_timestamp(next_airing.airing_at as i64, 0)
        .unwrap_or_default()
        .naive_utc();

    let new_activity = activity_data::ActiveModel {
        anime_id: Set(row.anime_id),
        timestamp: Set(timestamp),
        server_id: Set(guild_id.to_string()),
        webhook: Set(row.webhook.clone()),
        episode: Set(next_airing.episode),
        name: Set(name),
        delay: Set(row.delay),
        image: Set(row.image.clone()),
        ..Default::default()
    };

    ActivityData::insert(new_activity).exec(&connection).await?;

    Ok(())
}

async fn remove_activity(row: &Model, guild_id: &str, db_config: DbConfig) -> Result<DeleteResult> {
    trace!(
        "Attempting to remove activity for anime_id: {} in guild: {}",
        row.anime_id,
        guild_id
    );

    let connection: DatabaseConnection = sea_orm::Database::connect(get_url(db_config)).await?;

    let delete_result = ActivityData::delete(activity_data::ActiveModel {
        anime_id: Set(row.anime_id),
        server_id: Set(guild_id.to_string()),
        ..Default::default()
    })
    .exec(&connection)
    .await?;

    trace!(
        "Removed {} row(s) for anime_id: {} in guild: {}",
        delete_result.rows_affected,
        row.anime_id,
        guild_id
    );

    Ok(delete_result)
}
