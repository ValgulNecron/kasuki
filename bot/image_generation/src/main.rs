mod calculate;
mod color;
mod mosaic;

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Context, Result};
use redis::AsyncCommands;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::config::Config;
use shared::image_saver::storage::{create_image_store, ImageStore};
use shared::queue::publisher::{SERVER_IMAGE_QUEUE_KEY, USER_COLOR_QUEUE_KEY};
use shared::queue::tasks::{ImageTask, MemberColorData};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::calculate::{calculate_user_color_from_url, get_image_from_url};
use crate::color::create_color_vector;

use shared::database::guild_data::ActiveModel as GuildActiveModel;
use shared::database::prelude::{GuildData, ServerImage};
use shared::database::server_image::{ActiveModel, Column};

#[tokio::main]
async fn main() -> Result<()> {
	let config = Config::new().context("Failed to load config.toml")?;

	let _sentry_guard = config.sentry_url.as_deref().map(|url| {
		let guard = sentry::init((
			url,
			sentry::ClientOptions {
				release: sentry::release_name!(),
				..Default::default()
			},
		));
		println!("Sentry initialized successfully");
		guard
	});

	let log_level: tracing::Level = config
		.logging
		.log_level
		.parse()
		.unwrap_or(tracing::Level::INFO);
	let sentry_layer = sentry::integrations::tracing::layer();
	tracing_subscriber::registry()
		.with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
		.with(sentry_layer)
		.with(tracing_subscriber::fmt::layer())
		.init();

	info!("Starting Image Generation worker...");

	let db: Arc<DatabaseConnection> = Arc::new(
		config
			.db
			.connect()
			.await
			.context("Failed to connect to database")?,
	);
	info!("Connected to database");

	let store: Arc<dyn ImageStore> = Arc::from(
		create_image_store(&config.image.storage)
			.context("Failed to create image store")?,
	);
	info!(
		"Image store initialized (type: {})",
		config.image.storage.storage_type
	);

	let queue_config = &config.queue;
	let redis_url = queue_config.redis_url();
	let client =
		redis::Client::open(redis_url.as_str()).context("Failed to create Redis client")?;
	let mut connection = client
		.get_multiplexed_async_connection()
		.await
		.context("Failed to connect to Redis")?;

	info!(
		"Connected to Redis at {}:{}",
		queue_config.host, queue_config.port
	);

	let max_workers = config.image.max_workers;
	let semaphore = Arc::new(Semaphore::new(max_workers));
	info!("Image generation worker pool size: {}", max_workers);

	loop {
		let permit = semaphore
			.clone()
			.acquire_owned()
			.await
			.expect("semaphore closed unexpectedly");

		let payload = match get_priority_task(&mut connection).await {
			Ok(Some(payload)) => payload,
			Ok(None) => {
				drop(permit);
				continue;
			},
			Err(e) => {
				warn!("Redis error while waiting for task: {:#}", e);
				drop(permit);
				match client.get_multiplexed_async_connection().await {
					Ok(new_conn) => {
						connection = new_conn;
						info!(
							"Reconnected to Redis at {}:{}",
							queue_config.host, queue_config.port
						);
					},
					Err(re) => {
						error!("Redis reconnect failed: {:#}", re);
						tokio::time::sleep(std::time::Duration::from_secs(5)).await;
					},
				}
				continue;
			},
		};

		info!("Received task ({} bytes)", payload.len());
		let task: ImageTask = match serde_json::from_str(&payload) {
			Ok(t) => t,
			Err(e) => {
				error!("Failed to deserialize task: {:#}", e);
				drop(permit);
				continue;
			},
		};

		let db = db.clone();
		let store = store.clone();
		tokio::spawn(async move {
			let _permit = permit;
			if let Err(e) = handle_task(task, &db, &store).await {
				error!("Failed to process task: {:#}", e);
			}
		});
	}
}

async fn get_priority_task(
	connection: &mut redis::aio::MultiplexedConnection,
) -> redis::RedisResult<Option<String>> {
	let user_color: Option<String> = connection.lpop(USER_COLOR_QUEUE_KEY, None).await?;
	if user_color.is_some() {
		return Ok(user_color);
	}

	let server_image: Option<String> = connection.lpop(SERVER_IMAGE_QUEUE_KEY, None).await?;
	if server_image.is_some() {
		return Ok(server_image);
	}

	let result: Option<(String, String)> = connection
		.blpop(&[USER_COLOR_QUEUE_KEY, SERVER_IMAGE_QUEUE_KEY], 30.0)
		.await?;
	Ok(result.map(|(_key, payload)| payload))
}

async fn handle_task(
	task: ImageTask, db: &Arc<DatabaseConnection>, store: &Arc<dyn ImageStore>,
) -> Result<()> {
	match task {
		ImageTask::GenerateServerImage {
			guild_id,
			guild_name,
			guild_icon_url,
			image_type,
			members,
			blacklist,
		} => {
			handle_generate_server_image(
				guild_id,
				guild_name,
				guild_icon_url,
				image_type,
				members,
				blacklist,
				db,
				store,
			)
			.await
		},
		ImageTask::CalculateUserColor {
			user_id,
			profile_picture_url,
		} => handle_calculate_user_color(user_id, profile_picture_url, db, store).await,
	}
}

async fn handle_generate_server_image(
	guild_id: String, guild_name: String, guild_icon_url: String, image_type: String,
	members: Vec<MemberColorData>, blacklist: HashSet<String>, db: &Arc<DatabaseConnection>,
	store: &Arc<dyn ImageStore>,
) -> Result<()> {
	let is_global = members.is_empty();

	use shared::database::prelude::UserColor;
	use shared::database::user_color::Column as UserColorColumn;

	let (effective_members, color_map) = if is_global {
		let all_colors = UserColor::find().all(&**db).await.unwrap_or_default();
		info!(
			"Generating global server image for guild {} ({} users from DB)",
			guild_id,
			all_colors.len()
		);

		let mut members_out = Vec::with_capacity(all_colors.len());
		let mut map = std::collections::HashMap::with_capacity(all_colors.len());

		for uc in all_colors {
			if blacklist.contains(&uc.user_id) {
				continue;
			}
			members_out.push(MemberColorData {
				user_id: uc.user_id.clone(),
				profile_picture_url: uc.profile_picture_url.clone(),
			});
			map.insert(uc.user_id.clone(), uc);
		}

		(members_out, map)
	} else {
		info!(
			"Generating local server image for guild {} ({} members)",
			guild_id,
			members.len()
		);

		let member_user_ids: Vec<String> = members
			.iter()
			.filter(|m| !blacklist.contains(&m.user_id))
			.map(|m| m.user_id.clone())
			.collect();

		let color_records = UserColor::find()
			.filter(UserColorColumn::UserId.is_in(member_user_ids))
			.all(&**db)
			.await
			.unwrap_or_default();

		let map = color_records
			.into_iter()
			.map(|r| (r.user_id.clone(), r))
			.collect();

		(members, map)
	};

	let mut color_tuples: Vec<(String, Vec<u8>)> = Vec::with_capacity(effective_members.len());

	for member in &effective_members {
		if blacklist.contains(&member.user_id) {
			continue;
		}

		let db_record = color_map.get(&member.user_id);

		let (hex_color, png_bytes) = match db_record {
			Some(record)
				if record.profile_picture_url == member.profile_picture_url
					&& !record.images.starts_with("data:") =>
			{
				let color = record.color.clone();
				match store.load(&record.images).await {
					Ok(bytes) => (color, bytes),
					Err(e) => {
						debug!(
							"Failed to load cached image for user {} from storage: {:#}",
							member.user_id, e
						);
						match calculate_user_color_from_url(&member.profile_picture_url).await {
							Ok((color, thumb_png, full_png)) => {
								save_user_color(
									&member.user_id,
									&member.profile_picture_url,
									&color,
									&thumb_png,
									&full_png,
									db,
									store,
								)
								.await;
								(color, thumb_png)
							},
							Err(e) => {
								warn!(
									"Failed to calculate color for user {}: {:#}",
									member.user_id, e
								);
								continue;
							},
						}
					},
				}
			},
			_ => match calculate_user_color_from_url(&member.profile_picture_url).await {
				Ok((color, thumb_png, full_png)) => {
					save_user_color(
						&member.user_id,
						&member.profile_picture_url,
						&color,
						&thumb_png,
						&full_png,
						db,
						store,
					)
					.await;
					(color, thumb_png)
				},
				Err(e) => {
					warn!(
						"Failed to calculate color for user {}: {:#}",
						member.user_id, e
					);
					continue;
				},
			},
		};

		color_tuples.push((hex_color, png_bytes));
	}

	if color_tuples.is_empty() {
		warn!(
			"No color data for guild {}, skipping image generation",
			guild_id
		);
		return Ok(());
	}

	let color_vec = tokio::task::spawn_blocking(move || create_color_vector(color_tuples))
		.await
		.context("spawn_blocking panicked")?;

	let guild_icon_download_url = calculate::change_to_x128_url(&guild_icon_url);
	let guild_icon = get_image_from_url(&guild_icon_download_url).await?;

	let image_data =
		tokio::task::spawn_blocking(move || mosaic::generate_mosaic(&guild_icon, &color_vec))
			.await
			.context("spawn_blocking panicked")??;

	let storage_key = format!("server_images/{}/{}.png", guild_id, image_type);
	store
		.save(&storage_key, &image_data)
		.await
		.context("Failed to save server image to storage")?;

	GuildData::insert(GuildActiveModel {
		guild_id: Set(guild_id.clone()),
		guild_name: Set(guild_name.clone()),
		updated_at: Set(chrono::Utc::now().naive_utc()),
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(shared::database::guild_data::Column::GuildId)
			.update_column(shared::database::guild_data::Column::GuildName)
			.update_column(shared::database::guild_data::Column::UpdatedAt)
			.to_owned(),
	)
	.exec(&**db)
	.await
	.context("Failed to upsert guild_data before server image")?;

	ServerImage::insert(ActiveModel {
		server_id: Set(guild_id.clone()),
		server_name: Set(guild_name),
		image_type: Set(image_type),
		image: Set(storage_key),
		image_url: Set(guild_icon_url),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::columns([Column::ImageType, Column::ServerId])
			.update_column(Column::Image)
			.update_column(Column::ImageUrl)
			.update_column(Column::GeneratedAt)
			.update_column(Column::ServerName)
			.to_owned(),
	)
	.exec(&**db)
	.await
	.context("Failed to upsert server image into database")?;

	info!("Generated server image for guild {}", guild_id);
	Ok(())
}

async fn handle_calculate_user_color(
	user_id: String, profile_picture_url: String, db: &Arc<DatabaseConnection>,
	store: &Arc<dyn ImageStore>,
) -> Result<()> {
	info!("Calculating color for user {}", user_id);

	use shared::database::prelude::UserColor;
	use shared::database::user_color::Column;

	let existing = UserColor::find()
		.filter(Column::UserId.eq(&user_id))
		.one(&**db)
		.await?;

	if let Some(ref record) = existing {
		let age = chrono::Utc::now().naive_utc() - record.calculated_at;
		let is_stale = age > chrono::Duration::days(7);
		if record.profile_picture_url == profile_picture_url
			&& !record.images.starts_with("data:")
			&& !is_stale
		{
			info!("User {} color is up to date, skipping", user_id);
			return Ok(());
		}
	}

	let (average_color, thumb_png, full_png) =
		calculate_user_color_from_url(&profile_picture_url).await?;

	save_user_color(
		&user_id,
		&profile_picture_url,
		&average_color,
		&thumb_png,
		&full_png,
		db,
		store,
	)
	.await;

	info!("Calculated color {} for user {}", average_color, user_id);
	Ok(())
}

async fn save_user_color(
	user_id: &str, profile_picture_url: &str, color: &str, thumb_png: &[u8], full_png: &[u8],
	db: &Arc<DatabaseConnection>, store: &Arc<dyn ImageStore>,
) {
	let thumb_key = format!("user_colors/{}.png", user_id);
	let full_key = format!("user_colors/{}_full.png", user_id);

	if let Err(e) = store.save(&thumb_key, thumb_png).await {
		error!(
			"Failed to save user color thumbnail for {} to storage: {:#}",
			user_id, e
		);
		return;
	}

	if let Err(e) = store.save(&full_key, full_png).await {
		error!(
			"Failed to save user color full image for {} to storage: {:#}",
			user_id, e
		);
	}

	use shared::database::prelude::UserColor;
	use shared::database::user_color::{ActiveModel, Column};

	if let Err(e) = UserColor::insert(ActiveModel {
		user_id: Set(user_id.to_string()),
		profile_picture_url: Set(profile_picture_url.to_string()),
		color: Set(color.to_string()),
		images: Set(thumb_key),
		calculated_at: Set(chrono::Utc::now().naive_utc()),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(Column::UserId)
			.update_column(Column::Color)
			.update_column(Column::ProfilePictureUrl)
			.update_column(Column::Images)
			.update_column(Column::CalculatedAt)
			.to_owned(),
	)
	.exec(&**db)
	.await
	{
		error!("Failed to upsert user color for {}: {:?}", user_id, e);
	}
}
