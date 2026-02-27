use anyhow::{Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader};
use std::io::Cursor;
use tracing::debug;

pub fn change_to_x128_url(url: &str) -> String {
	let base_url = url.split('?').next().unwrap_or(url);
	format!("{}?size=128&quality=lossless", base_url)
}

pub async fn get_image_from_url(url: &str) -> Result<DynamicImage> {
	let resp = reqwest::get(url)
		.await
		.context(format!("Failed to fetch image from URL: {}", url))?
		.bytes()
		.await
		.context(format!("Failed to get image bytes from URL: {}", url))?;

	let url_owned = url.to_string();
	tokio::task::spawn_blocking(move || {
		let img = ImageReader::new(Cursor::new(resp))
			.with_guessed_format()
			.context(format!("Failed to guess image format from URL: {}", url_owned))?
			.decode()
			.context(format!("Failed to decode image from URL: {}", url_owned))?;
		Ok(img)
	})
	.await
	.context("spawn_blocking panicked")?
}

pub async fn calculate_user_color_from_url(profile_picture_url: &str) -> Result<(String, String)> {
	let url = change_to_x128_url(profile_picture_url);
	let img = get_image_from_url(&url).await?;

	tokio::task::spawn_blocking(move || {
		let img = img.to_rgba8();

		let (r_total, g_total, b_total) = img
			.enumerate_pixels()
			.map(|(_, _, pixel)| (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32))
			.fold((0u32, 0u32, 0u32), |(r1, g1, b1), (r2, g2, b2)| {
				(r1 + r2, g1 + g2, b1 + b2)
			});

		let num_pixels = img.width() * img.height();
		let r_avg = r_total / num_pixels;
		let g_avg = g_total / num_pixels;
		let b_avg = b_total / num_pixels;
		let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);

		debug!("Calculated color: {}", average_color);

		let mut image_data: Vec<u8> = Vec::new();
		PngEncoder::new(&mut image_data).write_image(
			img.as_raw(),
			img.width(),
			img.height(),
			ExtendedColorType::Rgba8,
		)?;

		let base64_image = general_purpose::STANDARD.encode(&image_data);
		let image = format!("data:image/png;base64,{}", base64_image);

		Ok((average_color, image))
	})
	.await
	.context("spawn_blocking panicked")?
}
