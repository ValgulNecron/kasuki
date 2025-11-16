use anyhow::{Context, Result};
use image::codecs::png;
use image::codecs::png::{CompressionType, PngEncoder};
use image::{DynamicImage, Rgba, RgbaImage};
use image::{ExtendedColorType, ImageEncoder};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

pub fn generate_progress_bar_image(percent: u32, user_color: Option<[u8; 3]>) -> Result<()> {
	// Ensure percent is between 0 and 100
	let percent = percent.min(100);

	// Define image dimensions
	let width = 500;
	let height = 50;

	// Create a new image
	let mut img = RgbaImage::new(width, height);

	// Define colors
	let background_color = Rgba([30, 30, 30, 255]); // Flat dark color
	let default_color = Rgba([0, 123, 255, 255]); // Default blue
	let progress_color = match user_color {
		Some([r, g, b]) => Rgba([r, g, b, 255]), // User color with full opacity
		None => default_color,                   // Default color if none provided
	};
	let border_color = Rgba([50, 50, 50, 255]); // Darker border

	// Fill background
	for x in 0..width {
		for y in 0..height {
			img.put_pixel(x, y, background_color);
		}
	}

	// Calculate progress width
	let progress_width = (width as f32 * (percent as f32 / 100.0)) as u32;

	// Fill progress
	for x in 0..progress_width {
		for y in 0..height {
			img.put_pixel(x, y, progress_color);
		}
	}

	// Draw border
	for x in 0..width {
		img.put_pixel(x, 0, border_color);
		img.put_pixel(x, height - 1, border_color);
	}

	for y in 0..height {
		img.put_pixel(0, y, border_color);
		img.put_pixel(width - 1, y, border_color);
	}

	// Convert to DynamicImage
	let dynamic_img = DynamicImage::ImageRgba8(img);

	// Ensure server_image directory exists
	let dir_path = "server_image";
	if !Path::new(dir_path).exists() {
		fs::create_dir_all(dir_path)?;
	}

	// Save the image
	let file_path = format!("{}/progress_{}.png", dir_path, percent);

	// Encode the image as PNG
	let mut image_data: Vec<u8> = Vec::new();
	PngEncoder::new_with_quality(
		&mut image_data,
		CompressionType::Best,
		png::FilterType::Adaptive,
	)
	.write_image(
		dynamic_img.as_bytes(),
		dynamic_img.width(),
		dynamic_img.height(),
		ExtendedColorType::Rgba8,
	)?;

	// Write to file
	fs::write(&file_path, image_data)
		.with_context(|| format!("Failed to write image to {}", file_path))?;

	info!("Generated progress bar image: {}", file_path);

	Ok(())
}

/// Generates all progress bar images needed for the application.
///
/// This function generates progress bar images for 0%, 10%, 20%, ..., 100%.
///
/// # Returns
///
/// A `Result` indicating success or failure.
///

/// Generates a progress bar image in memory with the specified percentage and color.
///
/// # Arguments
///
/// * `percent` - The percentage of the progress bar to fill (0-100)
/// * `user_color` - Optional RGB color to use for the progress bar (as [r, g, b])
///
/// # Returns
///
/// A `Result` containing the image data as a Vec<u8> if successful.
///
pub fn generate_progress_bar_image_in_memory(percent: u32, user_color: [u8; 4]) -> Result<Vec<u8>> {
	// Ensure percent is between 0 and 100
	let percent = percent.min(100);

	// Define image dimensions
	let width = 500;
	let height = 50;

	// Create a new image
	let mut img = RgbaImage::new(width, height);

	// Define colors
	let background_color = Rgba([30, 30, 30, 255]); // Flat dark color
	let progress_color = Rgba::from(user_color);
	let border_color = Rgba([50, 50, 50, 255]); // Darker border

	// Fill background
	for x in 0..width {
		for y in 0..height {
			img.put_pixel(x, y, background_color);
		}
	}

	// Calculate progress width
	let progress_width = (width as f32 * (percent as f32 / 100.0)) as u32;

	// Fill progress
	for x in 0..progress_width {
		for y in 0..height {
			img.put_pixel(x, y, progress_color);
		}
	}

	// Draw border
	for x in 0..width {
		img.put_pixel(x, 0, border_color);
		img.put_pixel(x, height - 1, border_color);
	}

	for y in 0..height {
		img.put_pixel(0, y, border_color);
		img.put_pixel(width - 1, y, border_color);
	}

	// Convert to DynamicImage
	let dynamic_img = DynamicImage::ImageRgba8(img);

	// Encode the image as PNG
	let mut image_data: Vec<u8> = Vec::new();
	PngEncoder::new_with_quality(
		&mut image_data,
		CompressionType::Best,
		png::FilterType::Adaptive,
	)
	.write_image(
		dynamic_img.as_bytes(),
		dynamic_img.width(),
		dynamic_img.height(),
		ExtendedColorType::Rgba8,
	)?;

	Ok(image_data)
}

pub fn generate_all_progress_bar_images() -> Result<()> {
	debug!("Generating all progress bar images");

	// Generate progress bars with default color
	for percent in (0..=100).step_by(10) {
		match generate_progress_bar_image(percent, None) {
			Ok(_) => debug!(
				"Generated progress bar image for {}% with default color",
				percent
			),
			Err(e) => {
				error!(
					"Failed to generate progress bar image for {}%: {}",
					percent, e
				);
				return Err(e).context(format!(
					"Failed to generate progress bar image for {}%",
					percent
				));
			},
		}
	}

	info!("Successfully generated all progress bar images");

	Ok(())
}
