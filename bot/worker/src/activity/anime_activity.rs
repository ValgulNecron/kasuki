use std::sync::Arc;
use std::time::Duration;

use std::borrow::Cow;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter};
use serenity::builder::{CreateAttachment, EditWebhook, ExecuteWebhook};
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use shared::anilist::minimal_anime::get_minimal_anime_media;
use shared::cache::CacheInterface;
use shared::database::activity_data;
use shared::database::activity_data::Model;
use shared::database::prelude::ActivityData;
use shared::localization::{get_language_identifier, FluentValue, Loader, USABLE_LOCALES};
use tracing::{error, info, trace};

/// Manages activity notifications. Tracks the last check time internally
/// so it can find all activities that should have fired since the last poll.
pub async fn manage_activity(
	http: Arc<Http>, anilist_cache: Arc<CacheInterface>, db_connection: Arc<DatabaseConnection>,
) {
	use std::sync::OnceLock;
	// OnceLock + Mutex: first call seeds "now" so the initial query returns no rows (no backfill).
	// Subsequent calls query only the window since the previous poll.
	static LAST_CHECK: OnceLock<tokio::sync::Mutex<NaiveDateTime>> = OnceLock::new();

	let now = Utc::now().naive_utc();
	let last_check_mutex = LAST_CHECK.get_or_init(|| tokio::sync::Mutex::new(now));
	let mut last_check = last_check_mutex.lock().await;

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

	// Advance checkpoint before processing so concurrent polls won't re-fetch the same rows
	*last_check = now;
	// Drop the lock early — processing is spawned as independent tasks below
	drop(last_check);

	for row in rows {
		let guild_id = row.server_id.clone();
		let anilist_cache = anilist_cache.clone();
		let db_connection = db_connection.clone();
		let http_clone = http.clone();
		let delay = row.delay;

		tokio::spawn(async move {
			// User-configured delay to stagger notifications (e.g., avoid spoilers right at air time)
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
	row: &Model, guild_id: String, http: &Arc<Http>, anilist_cache: Arc<CacheInterface>,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let lang_id = get_language_identifier(guild_id.clone(), db_connection.clone()).await;

	let mut args = std::collections::HashMap::new();
	args.insert(
		Cow::Borrowed("ep"),
		FluentValue::from(row.episode.to_string()),
	);
	args.insert(Cow::Borrowed("anime"), FluentValue::from(row.name.clone()));

	let title = USABLE_LOCALES.lookup(&lang_id, "anilist_user_send_activity-title");
	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_send_activity-desc", &args);

	// If webhook is None but channel_id is set, create a webhook in that channel
	let webhook_url = match &row.webhook {
		Some(url) => url.clone(),
		// Fallback: API-created activities may only have a channel_id — create a webhook on the fly
		None => {
			let channel_id_str = row
				.channel_id
				.as_ref()
				.ok_or_else(|| anyhow!("Activity has no webhook and no channel_id"))?;
			let channel_id: u64 = channel_id_str.parse().context("Invalid channel_id")?;

			// Discord webhook names are limited to 80 chars; cap at 100 for safety
			let trimmed_name = row.name.chars().take(100).collect::<String>();
			let new_webhook = http
				.create_webhook(
					serenity::model::id::ChannelId::new(channel_id),
					&serde_json::json!({ "name": trimmed_name }),
					None,
				)
				.await
				.context("Failed to create webhook in channel")?;

			let url = new_webhook.url().context("Created webhook has no URL")?;

			// Update DB with the new webhook URL
			use sea_orm::ActiveModelTrait;
			let mut active = activity_data::ActiveModel {
				anime_id: Set(row.anime_id),
				server_id: Set(row.server_id.clone()),
				..Default::default()
			};
			active.webhook = Set(Some(url.clone()));
			active.update(&*db_connection).await?;

			info!(
				anime_id = row.anime_id,
				server = %row.server_id,
				"created webhook for API-created activity"
			);

			url
		},
	};

	let mut webhook = Webhook::from_url(http, &webhook_url).await?;

	let decoded_bytes = decode_image(&row.image)?;
	let trimmed_name = row.name.chars().take(100).collect::<String>();

	let filename = format!("{}_{}.png", guild_id, row.anime_id);
	let attachment = CreateAttachment::bytes(decoded_bytes, filename);
	let attachment = attachment.encode("image/png").await?;

	let edit_webhook = EditWebhook::new().name(trimmed_name).avatar(attachment);
	webhook.edit(http, edit_webhook).await?;

	let embed = serenity::builder::CreateEmbed::new()
		.description(desc)
		.url(format!("https://anilist.co/anime/{}", row.anime_id))
		.title(title);

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
	// Strip the optional data-URI prefix (e.g., "data:image/png;base64,") if present
	let base64_str = match image.find(',') {
		Some(idx) => &image[idx + 1..],
		None => image,
	};

	STANDARD
		.decode(base64_str)
		.context("Failed to decode base64 image")
}

async fn update_info(
	row: &Model, guild_id: &str, anilist_cache: Arc<CacheInterface>,
	db_connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let media = get_minimal_anime_media(row.anime_id.to_string(), anilist_cache).await?;

	// No next episode means the series finished — clean up the subscription
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
		channel_id: Set(row.channel_id.clone()),
	};

	// Upsert: (anime_id, server_id) is the composite PK — update the next-episode timestamp and metadata
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
				activity_data::Column::ChannelId,
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
