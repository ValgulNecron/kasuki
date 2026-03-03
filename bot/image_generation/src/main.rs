mod calculate;
mod color;
mod mosaic;

use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Semaphore;
use redis::AsyncCommands;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::config::Config;
use shared::image_saver::general_image_saver::image_saver;
use shared::queue::publisher::{SERVER_IMAGE_QUEUE_KEY, USER_COLOR_QUEUE_KEY};
use shared::queue::tasks::{ImageSaveConfig, ImageTask, MemberColorData};
use tracing::{error, info, warn, debug};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use crate::calculate::{calculate_user_color_from_url, get_image_from_url};
use crate::color::create_color_vector_from_tuple;

use shared::database::guild_data::ActiveModel as GuildActiveModel;
use shared::database::prelude::{GuildData, ServerImage};
use shared::database::server_image::{ActiveModel, Column};

#[tokio::main]
async fn main() -> Result<()> {
	let config = Config::new().context("Failed to load config.toml")?;

	let log_level = config
		.logging
		.log_level
		.parse()
		.unwrap_or(tracing::Level::INFO);
	let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
	tracing::subscriber::set_global_default(subscriber)
		.context("Failed to set default subscriber")?;

	info!("Starting Image Generation worker...");

	let db: Arc<DatabaseConnection> = Arc::new(
		config
			.db
			.connect()
			.await
			.context("Failed to connect to database")?,
	);
	info!("Connected to database");

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
		let permit = semaphore.clone().acquire_owned().await.unwrap();

		let result: Option<(String, String)> = match connection
			.blpop(&[SERVER_IMAGE_QUEUE_KEY, USER_COLOR_QUEUE_KEY], 30.0)
			.await
		{
			Ok(r) => r,
			Err(e) => {
				debug!("Redis error while waiting for task: {:#}", e);
				drop(permit);
				continue;
			},
		};

		match result {
			Some((_key, payload)) => {
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
				tokio::spawn(async move {
					let _permit = permit;
					if let Err(e) = handle_task(task, &db).await {
						error!("Failed to process task: {:#}", e);
					}
				});
			},
			None => {
				drop(permit);
				continue;
			},
		}
	}
}

async fn handle_task(task: ImageTask, db: &Arc<DatabaseConnection>) -> Result<()> {
	match task {
		ImageTask::GenerateServerImage {
			guild_id,
			guild_name,
			guild_icon_url,
			image_type,
			members,
			blacklist,
			image_save_config,
		} => {
			handle_generate_server_image(
				guild_id,
				guild_name,
				guild_icon_url,
				image_type,
				members,
				blacklist,
				image_save_config,
				db,
			)
			.await
		},
		ImageTask::CalculateUserColor {
			user_id,
			profile_picture_url,
		} => handle_calculate_user_color(user_id, profile_picture_url, db).await,
	}
}

async fn handle_generate_server_image(
	guild_id: String, guild_name: String, guild_icon_url: String, image_type: String,
	members: Vec<MemberColorData>, blacklist: Vec<String>, image_save_config: ImageSaveConfig,
	db: &Arc<DatabaseConnection>,
) -> Result<()> {
	info!(
		"Generating {} server image for guild {} ({} members)",
		image_type,
		guild_id,
		members.len()
	);

	let mut color_tuples: Vec<(String, String, String)> = Vec::with_capacity(members.len());

	for member in &members {
		if blacklist.contains(&member.user_id) {
			continue;
		}

		let (hex_color, base64_image) = match (&member.cached_color, &member.cached_image) {
			(Some(c), Some(i)) => (c.clone(), i.clone()),
			_ => match calculate_user_color_from_url(&member.profile_picture_url).await {
				Ok((color, image)) => {
					upsert_user_color(
						&member.user_id,
						&member.profile_picture_url,
						&color,
						&image,
						db,
					)
					.await;
					(color, image)
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

		color_tuples.push((hex_color, member.profile_picture_url.clone(), base64_image));
	}

	if color_tuples.is_empty() {
		warn!(
			"No color data for guild {}, skipping image generation",
			guild_id
		);
		return Ok(());
	}

	let color_vec =
		tokio::task::spawn_blocking(move || create_color_vector_from_tuple(color_tuples))
			.await
			.context("spawn_blocking panicked")?;

	let guild_icon = get_image_from_url(&guild_icon_url).await?;

	let (image_data, base64_image) =
		tokio::task::spawn_blocking(move || mosaic::generate_mosaic(&guild_icon, &color_vec))
			.await
			.context("spawn_blocking panicked")??;

	let image = format!("data:image/png;base64,{}", base64_image);
	let uuid = Uuid::new_v4();

	image_saver(
		guild_id.clone(),
		format!("{}.png", uuid),
		image_data,
		image_save_config.save_server,
		image_save_config.token,
		image_save_config.save_type,
	)
	.await
	.context("Failed to save server image")?;


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
		image: Set(image),
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
) -> Result<()> {
	info!("Calculating color for user {}", user_id);

	use shared::database::prelude::UserColor;
	use shared::database::user_color::Column;

	let existing = UserColor::find()
		.filter(Column::UserId.eq(&user_id))
		.one(&**db)
		.await?;

	if let Some(ref record) = existing {
		if record.profile_picture_url == profile_picture_url {
			info!("User {} color is up to date, skipping", user_id);
			return Ok(());
		}
	}

	let (average_color, image) = calculate_user_color_from_url(&profile_picture_url).await?;

	upsert_user_color(&user_id, &profile_picture_url, &average_color, &image, db).await;

	info!("Calculated color {} for user {}", average_color, user_id);
	Ok(())
}

async fn upsert_user_color(
	user_id: &str, profile_picture_url: &str, color: &str, image: &str,
	db: &Arc<DatabaseConnection>,
) {
	use shared::database::prelude::UserColor;
	use shared::database::user_color::{ActiveModel, Column};

	if let Err(e) = UserColor::insert(ActiveModel {
		user_id: Set(user_id.to_string()),
		profile_picture_url: Set(profile_picture_url.to_string()),
		color: Set(color.to_string()),
		images: Set(image.to_string()),
		..Default::default()
	})
	.on_conflict(
		sea_orm::sea_query::OnConflict::column(Column::UserId)
			.update_column(Column::Color)
			.update_column(Column::ProfilePictureUrl)
			.update_column(Column::Images)
			.to_owned(),
	)
	.exec(&**db)
	.await
	{
		error!("Failed to upsert user color for {}: {:?}", user_id, e);
	}
}
