use anyhow::{Context, Result};
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader};
use std::io::Cursor;
use tracing::debug;

// Discord CDN URLs carry size/quality as query params; strip and replace to control output
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
	// Image decoding is CPU-intensive; run off the async runtime to avoid blocking it
	tokio::task::spawn_blocking(move || {
		let img = ImageReader::new(Cursor::new(resp))
			.with_guessed_format()
			.context(format!(
				"Failed to guess image format from URL: {}",
				url_owned
			))?
			.decode()
			.context(format!("Failed to decode image from URL: {}", url_owned))?;
		Ok(img)
	})
	.await
	.context("spawn_blocking panicked")?
}

fn change_to_full_size_url(url: &str) -> String {
	let base_url = url.split('?').next().unwrap_or(url);
	format!("{}?size=4096&quality=lossless", base_url)
}

pub async fn calculate_user_color_from_url(
	profile_picture_url: &str,
) -> Result<(String, Vec<u8>, Vec<u8>)> {
	let url = change_to_full_size_url(profile_picture_url);
	let img = get_image_from_url(&url).await?;

	// Color averaging and PNG encoding are CPU-bound; keep off the async runtime
	tokio::task::spawn_blocking(move || {
		let img = img.to_rgba8();

		// Sum all pixel channels to compute a simple mean color for the entire image
		let (r_total, g_total, b_total) = img
			.enumerate_pixels()
			.map(|(_, _, pixel)| (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32))
			.fold((0u32, 0u32, 0u32), |(r1, g1, b1), (r2, g2, b2)| {
				(r1 + r2, g1 + g2, b1 + b2)
			});

		// u32 can hold up to ~16M pixels * 255 without overflow (4096x4096 = 16M, fits)
		let num_pixels = img.width() * img.height();
		let r_avg = r_total / num_pixels;
		let g_avg = g_total / num_pixels;
		let b_avg = b_total / num_pixels;
		let average_color = format!("#{:02x}{:02x}{:02x}", r_avg, g_avg, b_avg);

		debug!("Calculated color: {}", average_color);

		let mut full_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut full_png_bytes).write_image(
			img.as_raw(),
			img.width(),
			img.height(),
			ExtendedColorType::Rgba8,
		)?;

		// Lanczos3 for the thumbnail: higher quality downscale than bilinear
		let thumb = image::imageops::resize(&img, 128, 128, image::imageops::FilterType::Lanczos3);
		let mut thumb_png_bytes: Vec<u8> = Vec::new();
		PngEncoder::new(&mut thumb_png_bytes).write_image(
			thumb.as_raw(),
			thumb.width(),
			thumb.height(),
			ExtendedColorType::Rgba8,
		)?;

		// Returns: hex color string, 128x128 thumbnail (for mosaic tiles), full-size PNG (for display)
		Ok((average_color, thumb_png_bytes, full_png_bytes))
	})
	.await
	.context("spawn_blocking panicked")?
}
