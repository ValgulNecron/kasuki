use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::png;
use image::codecs::png::{CompressionType, PngEncoder};
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use rayon::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, EntityTrait};
use serenity::all::{Context as SerenityContext, GuildId, Member};
use tracing::{info, warn};
use uuid::Uuid;

use crate::event_handler::BotData;
use crate::helper::image_saver::general_image_saver::image_saver;
use crate::server_image::calculate_user_color::{
	change_to_x128_url, get_image_from_url, get_member, return_average_user_color,
};
use crate::server_image::common::{
	create_color_vector_from_tuple, create_color_vector_from_user_color, find_closest_color_index,
	Color, ColorWithUrl,
};
use shared::config::ImageConfig;
use shared::database::prelude::{ServerImage, UserColor};
use shared::database::server_image::{ActiveModel, Column};

pub async fn generate_local_server_image(
	ctx: &SerenityContext, guild_id: GuildId, image_config: ImageConfig,
	connection: Arc<DatabaseConnection>,
) -> Result<()> {
	let members: Vec<Member> = get_member(ctx.clone(), guild_id).await;

	let bot_data = ctx.data::<BotData>().clone();
	let user_blacklist = bot_data.user_blacklist.clone();
	let read_guard = user_blacklist.read().await.clone();
	let average_colors = return_average_user_color(members, connection.clone(), read_guard)
		.await
		.map_err(|e| {
			anyhow!(
				"Failed to return average user color for guild {}. {:?}",
				guild_id,
				e
			)
		})?;

	let color_vec =
		tokio::task::spawn_blocking(move || create_color_vector_from_tuple(average_colors))
			.await
			.context("spawn_blocking panicked")?;

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

	let bot_data = ctx.data::<BotData>().clone();
	let user_blacklist = bot_data.user_blacklist.clone();
	let read_guard = user_blacklist.read().await.clone();
	let color_vec = tokio::task::spawn_blocking(move || {
		create_color_vector_from_user_color(average_colors, read_guard)
	})
	.await
	.context("spawn_blocking panicked")?;

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

	// Move all CPU-heavy work (color matching, tile resizing, compositing, PNG encoding)
	// to a blocking thread so we don't starve the tokio runtime and block Discord responses.
	let (image_data, base64_image) = tokio::task::spawn_blocking(move || {
		// Use small tiles (32x32) instead of full 128x128 to reduce memory.
		// The final image is resized to ~2458x2458 anyway, so full-res tiles are wasted.
		let tile_size: u32 = 32;
		let canvas_dim = 128 * tile_size; // 4096 instead of 16384

		let mut combined_image = DynamicImage::new_rgba8(canvas_dim, canvas_dim);

		// Build pixel coordinates list
		let pixels: Vec<(u32, u32)> = (0..img.height())
			.flat_map(|y| (0..img.width()).map(move |x| (x, y)))
			.collect();

		// Use rayon to find the closest color index for each pixel in parallel
		let indices: Vec<(u32, u32, usize)> = pixels
			.par_iter()
			.filter_map(|&(x, y)| {
				let pixel = img.get_pixel(x, y);

				let r = pixel[0] as f32 / 255.0;
				let g = pixel[1] as f32 / 255.0;
				let b = pixel[2] as f32 / 255.0;

				let rgb_color = Srgb::new(r, g, b);
				let lab_color: Lab = <palette::rgb::Rgb as IntoColor<Lab>>::into_color(rgb_color);
				let color_target = Color { cielab: lab_color };

				find_closest_color_index(&average_colors, &color_target).map(|idx| (x, y, idx))
			})
			.collect();

		// Process each pixel sequentially: resize the tile and place it on the canvas
		for (x, y, idx) in indices {
			let tile = image::imageops::resize(
				&average_colors[idx].image,
				tile_size,
				tile_size,
				FilterType::Triangle,
			);
			let tile_img = DynamicImage::ImageRgba8(tile);
			if combined_image
				.copy_from(&tile_img, x * tile_size, y * tile_size)
				.is_err()
			{
				continue;
			}
		}

		// Drop average_colors before the resize to free memory
		drop(average_colors);

		let resized = image::imageops::resize(
			&combined_image,
			(4096.0 * 0.6) as u32,
			(4096.0 * 0.6) as u32,
			FilterType::CatmullRom,
		);

		// Drop the large canvas now that we have the resized version
		drop(combined_image);

		let mut image_data: Vec<u8> = Vec::new();

		PngEncoder::new_with_quality(
			&mut image_data,
			CompressionType::Best,
			png::FilterType::Adaptive,
		)
		.write_image(
			resized.as_raw(),
			resized.width(),
			resized.height(),
			ExtendedColorType::Rgba8,
		)?;

		let base64_image = general_purpose::STANDARD.encode(&image_data);

		Ok::<_, anyhow::Error>((image_data, base64_image))
	})
	.await
	.context("spawn_blocking panicked")??;

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
		if let Err(e) =
			generate_local_server_image(ctx, guild, image_config.clone(), connection.clone()).await
		{
			warn!(
				"Failed to generate local server image for guild {}. {:?}",
				guild, e
			);
		} else {
			info!("Generated local server image for guild {}", guild);
		}

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
