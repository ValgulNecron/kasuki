use anyhow::Result;
use image::codecs::png;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImage, GenericImageView, ImageEncoder};
use palette::{IntoColor, Lab, Srgb};
use rayon::prelude::*;

use crate::color::{find_closest_color_index, Color, ColorWithUrl};

/// Generate a mosaic image from a guild icon and member color data.
/// Returns raw PNG bytes.
pub fn generate_mosaic(
	guild_icon: &DynamicImage, average_colors: &[ColorWithUrl],
) -> Result<Vec<u8>> {
	let tile_size: u32 = 32;
	let canvas_dim = 128 * tile_size;

	let mut combined_image = DynamicImage::new_rgba8(canvas_dim, canvas_dim);

	let pixels: Vec<(u32, u32)> = (0..guild_icon.height())
		.flat_map(|y| (0..guild_icon.width()).map(move |x| (x, y)))
		.collect();

	let indices: Vec<(u32, u32, usize)> = pixels
		.par_iter()
		.filter_map(|&(x, y)| {
			let pixel = guild_icon.get_pixel(x, y);

			let r = pixel[0] as f32 / 255.0;
			let g = pixel[1] as f32 / 255.0;
			let b = pixel[2] as f32 / 255.0;

			let rgb_color = Srgb::new(r, g, b);
			let lab_color: Lab = <palette::rgb::Rgb as IntoColor<Lab>>::into_color(rgb_color);
			let color_target = Color { cielab: lab_color };

			find_closest_color_index(average_colors, &color_target).map(|idx| (x, y, idx))
		})
		.collect();

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

	let mut image_data: Vec<u8> = Vec::new();
	PngEncoder::new_with_quality(
		&mut image_data,
		png::CompressionType::Best,
		png::FilterType::Adaptive,
	)
	.write_image(
		combined_image.as_bytes(),
		combined_image.width(),
		combined_image.height(),
		ExtendedColorType::Rgba8,
	)?;

	Ok(image_data)
}
