use std::sync::{Arc, RwLock};
use std::thread;

use anyhow::{Context, Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose;
use image::codecs::png;
use image::codecs::png::{CompressionType, PngEncoder};
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serenity::all::{Context as SerenityContext, GuildId, Member};
use tokio::task;
use tracing::{info, warn};
use uuid::Uuid;

use crate::background_task::server_image::calculate_user_color::{
	change_to_x128_url, get_image_from_url, get_member, return_average_user_color,
};
use crate::background_task::server_image::common::{
	Color, ColorWithUrl, create_color_vector_from_tuple, create_color_vector_from_user_color,
	find_closest_color,
};
use crate::config::ImageConfig;
use crate::constant::THREAD_POOL_SIZE;
use crate::database::prelude::{ServerImage, UserColor};
use crate::database::server_image::{ActiveModel, Column};
use crate::helper::image_saver::general_image_saver::image_saver;

pub async fn generate_local_server_image(
	ctx: &SerenityContext, guild_id: GuildId, image_config: ImageConfig,
	connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let members: Vec<Member> = get_member(ctx.clone(), guild_id).await;

	let average_colors = return_average_user_color(members, connection.clone())
		.await
		.map_err(|e| {
			anyhow!(
				"Failed to return average user color for guild {}. {:?}",
				guild_id,
				e
			)
		})?;

	let color_vec = create_color_vector_from_tuple(average_colors.clone());

	generate_server_image(
		ctx,
		guild_id,
		color_vec,
		String::from("local"),
		image_config,
		connection,
	)
	.await
}

pub async fn generate_global_server_image(
	ctx: &SerenityContext, guild_id: GuildId, image_config: ImageConfig,
	connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let average_colors = UserColor::find().all(&*connection).await?;

	let color_vec = create_color_vector_from_user_color(average_colors.clone());

	generate_server_image(
		ctx,
		guild_id,
		color_vec,
		String::from("global"),
		image_config,
		connection,
	)
	.await
}

pub async fn generate_server_image(
	ctx: &SerenityContext, guild_id: GuildId, average_colors: Vec<ColorWithUrl>,
	image_type: String, image_config: ImageConfig, connection: Arc<DatabaseConnection>,
) -> Result<()> {
	info!("Generating server image for {}.", guild_id);

	let guild = guild_id
		.to_partial_guild(&ctx.http)
		.await
		.context("Failed to get partial guild")?;

	let guild_pfp = change_to_x128_url(
        guild
            .icon_url()
            .unwrap_or(String::from("https://cdn.discordapp.com/icons/1117152661620408531/541e10cc07361e99b7b1012861cd518a.webp?size=128&quality=lossless")),
    );

	let img = get_image_from_url(guild_pfp.clone()).await?;

	let dim = 128 * 128;

	let mut combined_image = DynamicImage::new_rgba8(dim, dim);

	let vec_image_rw: Arc<RwLock<Vec<(u32, u32, DynamicImage)>>> =
		Arc::new(RwLock::new(Vec::new()));

	let mut handles = Vec::new();

	for y in 0..img.height() {
		for x in 0..img.width() {
			let pixel = img.get_pixel(x, y);

			let color_vec_moved = average_colors.clone();

			let vec_image_clone = Arc::clone(&vec_image_rw);

			let handle = thread::spawn(move || {
				let r = pixel[0] as f32 / 255.0;

				let g = pixel[1] as f32 / 255.0;

				let b = pixel[2] as f32 / 255.0;

				let rgb_color = Srgb::new(r, g, b);

				let lab_color: Lab = <palette::rgb::Rgb as IntoColor<Lab>>::into_color(rgb_color);

				let color_target = Color { cielab: lab_color };

				let closest_color = match find_closest_color(&color_vec_moved, &color_target) {
					Some(color) => color,
					None => return,
				};

				let mut guard = match vec_image_clone.write() {
					Ok(guard) => guard,
					Err(_) => return,
				};

				guard.push((x, y, closest_color.image))
			});

			handles.push(handle);

			if handles.len() >= THREAD_POOL_SIZE {
				for handle in handles {
					match handle.join() {
						Ok(_) => {},
						Err(_) => continue,
					}
				}

				handles = Vec::new();
			}
		}
	}

	let vec_image = vec_image_rw
		.read()
		.map_err(|e| {
			anyhow!(
				"Failed to read from RwLock<Vec<(u32, u32, DynamicImage)>>. {:?}",
				e
			)
		})?
		.clone();

	drop(vec_image_rw);

	let internal_vec = vec_image.clone();

	for (x, y, image) in internal_vec {
		match combined_image.copy_from(&image, x * 128, y * 128) {
			Ok(_) => {},
			Err(_) => continue,
		}
	}

	let image = image::imageops::resize(
		&combined_image,
		(4096.0 * 0.6) as u32,
		(4096.0 * 0.6) as u32,
		FilterType::CatmullRom,
	);

	let img = image;

	let mut image_data: Vec<u8> = Vec::new();

	PngEncoder::new_with_quality(
		&mut image_data,
		CompressionType::Best,
		png::FilterType::Adaptive,
	)
	.write_image(
		img.as_raw(),
		img.width(),
		img.height(),
		ExtendedColorType::Rgba8,
	)?;

	let base64_image = general_purpose::STANDARD.encode(image_data.clone());

	let image = format!("data:image/png;base64,{}", base64_image);

	let uuid = Uuid::new_v4();

	// Save the image
	let token = image_config.token.clone().unwrap_or_default();

	let saver = image_config.save_server.clone().unwrap_or_default();

	let save_type = image_config.save_image.clone();

	image_saver(
		guild_id.to_string(),
		format!("{}.png", uuid),
		image_data,
		saver,
		token,
		save_type,
	)
	.await
	.map_err(|e| {
		anyhow!(
			"Failed to save server image for guild {}. {:?}",
			guild_id,
			e
		)
	})?;

	ServerImage::insert(ActiveModel {
		server_id: Set(guild_id.to_string()),
		server_name: Set(guild.name.to_string()),
		image_type: Set(image_type.clone()),
		image: Set(image.clone()),
		image_url: Set(guild_pfp.clone()),
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
	.exec(&*connection)
	.await
	.context("Failed to insert or update server image into database.")?;

	Ok(())
}

pub async fn server_image_management(
	ctx: &SerenityContext, image_config: ImageConfig, connection: Arc<DatabaseConnection>,
) {
	for guild in ctx.cache.guilds() {
		let ctx_clone = ctx.clone();

		let guild_clone = guild;

		let image_config_a = image_config.clone();

		let connection_a = connection.clone();

		task::spawn(async move {
			if let Err(e) =
				generate_local_server_image(&ctx_clone, guild_clone, image_config_a, connection_a)
					.await
			{
				warn!(
					"Failed to generate local server image for guild {}. {:?}",
					guild, e
				);
			} else {
				info!("Generated local server image for guild {}", guild);
			}
		});

		if let Err(e) =
			generate_global_server_image(ctx, guild, image_config.clone(), connection.clone()).await
		{
			warn!(
				"Failed to generate global server image for guild {}. {:?}",
				guild, e
			);
		} else {
			info!("Generated global server image for guild {}", guild);
		}
	}
}
