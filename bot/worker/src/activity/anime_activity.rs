use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateAttachment, EditWebhook, ExecuteWebhook};
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use shared::anilist::minimal_anime::get_minimal_anime_media;
use shared::cache::CacheInterface;
use shared::database::activity_data;
use shared::database::activity_data::Model;
use shared::database::prelude::ActivityData;
use shared::localization::load_localization;
use tokio::sync::RwLock;
use tracing::{error, info, trace};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendActivityLocalised {
	pub title: String,
	pub desc: String,
}

/// Manages activity notifications. Tracks the last check time internally
/// so it can find all activities that should have fired since the last poll.
pub async fn manage_activity(
	http: Arc<Http>, anilist_cache: Arc<RwLock<CacheInterface>>,
	db_connection: Arc<DatabaseConnection>,
) {
	// Use a static to track the last time we checked.
	// On first run, we start from "now" so we don't replay old activities.
	use std::sync::OnceLock;
	static LAST_CHECK: OnceLock<tokio::sync::Mutex<NaiveDateTime>> = OnceLock::new();

	let now = Utc::now().naive_utc();
	let last_check_mutex = LAST_CHECK.get_or_init(|| tokio::sync::Mutex::new(now));
	let mut last_check = last_check_mutex.lock().await;

	// Query: timestamp > last_check AND timestamp <= now
	let rows = match ActivityData::find()
		.filter(activity_data::Column::Timestamp.gt(*last_check))
		.filter(activity_data::Column::Timestamp.lte(now))
		.all(&*db_connection)
		.await
	{
		Ok(rows) => rows,
		Err(e) => {
			error!("Failed to query activity data: {}", e);
			return;
		},
	};

	if !rows.is_empty() {
		info!("Found {} activities to process", rows.len());
	}

	// Update last_check to now so next cycle picks up from here
	*last_check = now;
	// Drop the lock early
	drop(last_check);

	for row in rows {
		let guild_id = row.server_id.clone();
		let anilist_cache = anilist_cache.clone();
		let db_connection = db_connection.clone();
		let http_clone = http.clone();
		let delay = row.delay;

		tokio::spawn(async move {
			if delay > 0 {
				tokio::time::sleep(Duration::from_secs(delay as u64)).await;
			}

			if let Err(e) =
				send_specific_activity(&row, guild_id, &http_clone, anilist_cache, db_connection)
					.await
			{
				error!(
					"Failed to send activity for anime_id={} server={}: {:#}",
					row.anime_id, row.server_id, e
				);
			}
		});
	}
}

async fn send_specific_activity(
	row: &Model, guild_id: String, http: &Arc<Http>, anilist_cache: Arc<RwLock<CacheInterface>>,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let localised_text: SendActivityLocalised = load_localization(
		guild_id.clone(),
		"json/message/anilist_user/send_activity.json",
		db_connection.clone(),
	)
	.await?;

	let mut webhook = Webhook::from_url(http, &row.webhook).await?;

	let decoded_bytes = decode_image(&row.image)?;
	let trimmed_name = row.name.chars().take(100).collect::<String>();

	let filename = format!("{}_{}.png", guild_id, row.anime_id);
	let attachment = CreateAttachment::bytes(decoded_bytes, filename);
	let attachment = attachment.encode("image/png").await?;

	let edit_webhook = EditWebhook::new().name(trimmed_name).avatar(attachment);
	webhook.edit(http, edit_webhook).await?;

	let embed = serenity::builder::CreateEmbed::new()
		.description(
			localised_text
				.desc
				.replace("$ep$", &row.episode.to_string())
				.replace("$anime$", &row.name),
		)
		.url(format!("https://anilist.co/anime/{}", row.anime_id))
		.title(&localised_text.title);

	let builder_message = ExecuteWebhook::new().embed(embed);
	webhook.execute(http, false, builder_message).await?;

	let row_clone = row.clone();
	let guild_id_clone = guild_id.clone();
	tokio::spawn(async move {
		if let Err(e) = update_info(&row_clone, &guild_id_clone, anilist_cache, db_connection).await
		{
			error!("Failed to update activity info: {:#}", e);
		}
	});

	Ok(())
}

fn decode_image(image: &str) -> Result<Vec<u8>> {
	// Strip optional data URI prefix (e.g. "data:image/png;base64,")
	let base64_str = match image.find(',') {
		Some(idx) => &image[idx + 1..],
		None => image,
	};

	STANDARD
		.decode(base64_str)
		.context("Failed to decode base64 image")
}

async fn update_info(
	row: &Model, guild_id: &str, anilist_cache: Arc<RwLock<CacheInterface>>,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let media = get_minimal_anime_media(row.anime_id.to_string(), anilist_cache).await?;

	let next_airing = match media.next_airing_episode {
		Some(airing) => airing,
		None => {
			trace!(
				"No next airing for anime_id={}, removing activity",
				row.anime_id
			);
			remove_activity(row, guild_id, db_connection.clone()).await?;
			return Ok(());
		},
	};

	let title = media.title.ok_or(anyhow!("No title"))?;
	let name = title
		.english
		.or(title.romaji)
		.unwrap_or_else(|| "Unknown".to_string());

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

	// Upsert: update if this anime_id+server_id already exists
	ActivityData::insert(new_activity)
		.on_conflict(
			sea_orm::sea_query::OnConflict::columns([
				activity_data::Column::AnimeId,
				activity_data::Column::ServerId,
			])
			.update_columns([
				activity_data::Column::Timestamp,
				activity_data::Column::Episode,
				activity_data::Column::Name,
				activity_data::Column::Webhook,
				activity_data::Column::Delay,
				activity_data::Column::Image,
			])
			.to_owned(),
		)
		.exec(&*db_connection)
		.await?;

	Ok(())
}

async fn remove_activity(
	row: &Model, guild_id: &str, db_connection: Arc<DatabaseConnection>,
) -> Result<DeleteResult> {
	trace!(
		"Removing activity for anime_id={} guild={}",
		row.anime_id,
		guild_id
	);

	let result = ActivityData::delete(activity_data::ActiveModel {
		anime_id: Set(row.anime_id),
		server_id: Set(guild_id.to_string()),
		..Default::default()
	})
	.exec(&*db_connection)
	.await?;

	Ok(result)
}
