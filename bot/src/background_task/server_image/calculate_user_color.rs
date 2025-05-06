use anyhow::{Context, Result};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use crate::config::DbConfig;
use crate::database::prelude::UserColor;
use crate::database::user_color::{ActiveModel, Column, Model};
use crate::event_handler::{BotData, add_user_data_to_db};
use crate::get_url;
use base64::Engine;
use base64::engine::general_purpose;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::ImageReader;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::{Context as SerenityContext, GuildId, Member, User, UserId};
use serenity::nonmax::NonMaxU16;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, trace};

pub fn change_to_x128_url(url: String) -> String {
	debug!("Changing URL size to 128x128: {}", url);

	let mut url = url
		.replace("?size=4096", "?size=128")
		.replace("?size=2048", "?size=128")
		.replace("?size=1024", "?size=128")
		.replace("?size=512", "?size=128")
		.replace("?size=256", "?size=128")
		.replace("?size=128", "?size=128")
		.replace("?size=64", "?size=128");

	if !url.ends_with("?size=128") {
		url = format!("{}?size=128", url)
	}

	url = format!("{}&quality=lossless", url);

	url
}

pub async fn calculate_users_color(
	members: Vec<Member>, user_blacklist_server_image: Arc<RwLock<Vec<String>>>,
	bot_data: Arc<BotData>,
) -> Result<()> {
	let guard = user_blacklist_server_image.read().await;

	let connection = bot_data.db_connection.clone();

	for member in members {
		trace!("Calculating user color for {}", member.user.id);

		if guard.contains(&member.user.id.to_string()) {
			debug!(
				"Skipping user {} due to USER_BLACKLIST_SERVER_IMAGE",
				member.user.id
			);

			continue;
		}

		let pfp_url = change_to_x128_url(member.user.face());

		let id = member.user.id.to_string();

		let user_color = UserColor::find()
			.filter(Column::UserId.eq(id.clone()))
			.one(&*connection)
			.await?
			.unwrap_or(Model {
				user_id: id.clone(),
				profile_picture_url: String::from(""),
				color: String::from(""),
				images: String::from(""),
				calculated_at: Default::default(),
			});

		let pfp_url_old = user_color.profile_picture_url.clone();

		if pfp_url != pfp_url_old {
			let (average_color, image): (String, String) =
				calculate_user_color(member.user.clone()).await?;

			add_user_data_to_db(member.user.clone(), connection.clone()).await?;

			UserColor::insert(ActiveModel {
				user_id: Set(id.clone()),
				profile_picture_url: Set(pfp_url.clone()),
				color: Set(average_color.clone()),
				images: Set(image.clone()),
				..Default::default()
			})
			.on_conflict(
				OnConflict::column(Column::UserId)
					.update_column(Column::Color)
					.update_column(Column::ProfilePictureUrl)
					.update_column(Column::Images)
					.to_owned(),
			)
			.exec(&*connection)
			.await?;
		}

		trace!("Done calculating user color for {}", member.user.id);

		sleep(Duration::from_nanos(1)).await
	}

	Ok(())
}

pub async fn return_average_user_color(
	members: Vec<Member>, connection: Arc<DatabaseConnection>,
) -> Result<Vec<(String, String, String)>> {
	let mut average_colors = Vec::with_capacity(members.len());

	for member in members {
		let pfp_url = change_to_x128_url(member.user.face());

		let id = member.user.id.to_string();

		let user_color = UserColor::find()
			.filter(Column::UserId.eq(id.clone()))
			.one(&*connection)
			.await?;

		match user_color {
			Some(user_color) => {
				let (color, pfp_url_old, image_old) = (
					user_color.color,
					user_color.profile_picture_url,
					user_color.images,
				);

				if pfp_url != pfp_url_old {
					let (average_color, image) = calculate_user_color(member.user.clone()).await?;

					average_colors.push((average_color.clone(), pfp_url.clone(), image.clone()));

					UserColor::insert(ActiveModel {
						user_id: Set(id.clone()),
						profile_picture_url: Set(pfp_url.clone()),
						color: Set(average_color.clone()),
						images: Set(image.clone()),
						..Default::default()
					})
					.on_conflict(
						OnConflict::column(Column::UserId)
							.update_column(Column::Color)
							.update_column(Column::ProfilePictureUrl)
							.update_column(Column::Images)
							.update_column(Column::CalculatedAt)
							.to_owned(),
					)
					.exec(&*connection)
					.await?;

					continue;
				}

				average_colors.push((color, pfp_url_old, image_old));
			},
			None => {
				let (average_color, image) = calculate_user_color(member.user.clone()).await?;

				average_colors.push((average_color.clone(), pfp_url.clone(), image.clone()));

				UserColor::insert(ActiveModel {
					user_id: Set(id.clone()),
					profile_picture_url: Set(pfp_url.clone()),
					color: Set(average_color.clone()),
					images: Set(image.clone()),
					..Default::default()
				})
				.on_conflict(
					OnConflict::column(Column::UserId)
						.update_column(Column::Color)
						.update_column(Column::ProfilePictureUrl)
						.update_column(Column::Images)
						.update_column(Column::CalculatedAt)
						.to_owned(),
				)
				.exec(&*connection)
				.await?;
			},
		}
	}

	Ok(average_colors)
}

async fn calculate_user_color(user: User) -> Result<(String, String)> {
	let pfp_url = change_to_x128_url(user.face());

	let img = get_image_from_url(pfp_url).await?;

	// convert to rgba8 so every image use the same color type.
	let img = img.to_rgba8();

	// Fallback to CPU multithreading with rayon
	let (r_total, g_total, b_total) = img
		.enumerate_pixels()
		.par_bridge()
		.map(|(_, _, pixel)| (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32))
		.reduce(
			|| (0, 0, 0),
			|(r1, g1, b1), (r2, g2, b2)| (r1 + r2, g1 + g2, b1 + b2),
		);

	debug!("R: {}, G: {}, B: {}", r_total, g_total, b_total);

	// Calculate the average color by dividing the sum by the total number of pixels
	let num_pixels = img.width() * img.height();

	let r_avg = r_total / num_pixels;

	let g_avg = g_total / num_pixels;

	let b_avg = b_total / num_pixels;

	let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);

	debug!("{}", average_color);

	let mut image_data: Vec<u8> = Vec::new();

	PngEncoder::new(&mut image_data).write_image(
		img.as_raw(),
		img.width(),
		img.height(),
		ExtendedColorType::Rgba8,
	)?;

	let base64_image = general_purpose::STANDARD.encode(image_data.clone());

	let image = format!("data:image/png;base64,{}", base64_image);

	// Return the average color
	Ok((average_color, image))
}

pub async fn get_image_from_url(url: String) -> Result<DynamicImage> {
	// Fetch the image data
	let resp = reqwest::get(&url)
		.await
		.context(format!("Failed to fetch image from URL: {}", url))?
		.bytes()
		.await
		.context(format!("Failed to get image bytes from URL: {}", url))?;

	// Decode the image data
	let img = ImageReader::new(Cursor::new(resp))
		.with_guessed_format()
		.context(format!("Failed to guess image format from URL: {}", url))?
		.decode()
		.context(format!("Failed to decode image from URL: {}", url))?;

	Ok(img)
}

pub async fn color_management(
	guilds: &Vec<GuildId>, ctx_clone: &SerenityContext,
	user_blacklist_server_image: Arc<RwLock<Vec<String>>>, bot_data: Arc<BotData>,
) {
	let mut futures = FuturesUnordered::new();

	for guild in guilds {
		let guild_id = guild.to_string();

		debug!(guild_id);

		let ctx_clone = ctx_clone.clone();

		let guild = *guild;

		let future = get_member(ctx_clone, guild);

		futures.push(future);
	}

	let mut members = Vec::new();

	while let Some(mut result) = futures.next().await {
		let guild_id = match result.first() {
			Some(member) => member.guild_id.to_string(),
			None => String::from(""),
		};

		debug!("{}: {}", guild_id, result.len());

		members.append(&mut result);
	}

	match calculate_users_color(
		members.into_iter().collect(),
		user_blacklist_server_image,
		bot_data,
	)
	.await
	{
		Ok(_) => {},
		Err(e) => error!("{:?}", e),
	};
}

pub async fn get_member(ctx_clone: SerenityContext, guild: GuildId) -> Vec<Member> {
	let mut i = 0;

	let mut members_temp_out: Vec<Member> = Vec::new();

	while members_temp_out.len() == (1000 * i) {
		let mut members_temp_in = if i == 0 {
			match guild
				.members(
					&ctx_clone.http,
					Some(NonMaxU16::new(1000).unwrap_or_default()),
					None,
				)
				.await
			{
				Ok(members) => members,
				Err(e) => {
					error!("{:?}", e);

					break;
				},
			}
		} else {
			let user: UserId = match members_temp_out.last() {
				Some(member) => member.user.id,
				None => break,
			};

			match guild
				.members(
					&ctx_clone.http,
					Some(NonMaxU16::new(1000).unwrap_or_default()),
					Some(user),
				)
				.await
			{
				Ok(members) => members,
				Err(e) => {
					error!("{:?}", e);

					break;
				},
			}
		};

		i += 1;

		members_temp_out.append(&mut members_temp_in);
	}

	members_temp_out
}

pub async fn get_specific_user_color(
	user_blacklist_server_image: Arc<RwLock<Vec<String>>>, user: User, db_config: DbConfig,
) {
	if user_blacklist_server_image
		.read()
		.await
		.contains(&user.id.to_string())
	{
		debug!(
			"Skipping user {} due to USER_BLACKLIST_SERVER_IMAGE",
			user.id
		);

		return;
	}

	let pfp_url = change_to_x128_url(user.face());

	let id = user.id.to_string();

	let connection = sea_orm::Database::connect(get_url(db_config.clone()))
		.await
		.unwrap();

	let user_color = UserColor::find()
		.filter(Column::UserId.eq(id.clone()))
		.one(&connection)
		.await
		.unwrap_or(None)
		.unwrap_or(Model {
			user_id: id.clone(),
			profile_picture_url: String::from(""),
			color: String::from(""),
			images: String::from(""),
			calculated_at: Default::default(),
		});

	let pfp_url_old = user_color.profile_picture_url.clone();

	if pfp_url_old == pfp_url {
		return;
	}

	let (average_color, image): (String, String) =
		calculate_user_color(user.clone()).await.unwrap();

	UserColor::insert(ActiveModel {
		user_id: Set(id.clone()),
		profile_picture_url: Set(pfp_url.clone()),
		color: Set(average_color.clone()),
		images: Set(image.clone()),
		..Default::default()
	})
	.on_conflict(
		OnConflict::column(Column::UserId)
			.update_column(Column::Color)
			.update_column(Column::ProfilePictureUrl)
			.update_column(Column::Images)
			.to_owned(),
	)
	.exec(&connection)
	.await
	.unwrap();
}
