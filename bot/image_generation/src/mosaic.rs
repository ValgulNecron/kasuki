use anyhow::Result;
use image::codecs::png;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage};
use palette::{IntoColor, Lab, Srgb};
use rayon::prelude::*;

use crate::color::{find_closest_color_index, Color, ColorWithTile};

pub fn generate_mosaic(
	guild_icon: &image::DynamicImage, average_colors: &[ColorWithTile],
) -> Result<Vec<u8>> {
	let tile_size: u32 = 32;
	let canvas_dim = 128 * tile_size;

	let mut combined_image = RgbaImage::new(canvas_dim, canvas_dim);

	let indices: Vec<(u32, u32, usize)> = (0..guild_icon.height())
		.flat_map(|y| (0..guild_icon.width()).map(move |x| (x, y)))
		.par_bridge()
		.filter_map(|(x, y)| {
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
		image::imageops::replace(
			&mut combined_image,
			&average_colors[idx].tile,
			(x * tile_size) as i64,
			(y * tile_size) as i64,
		);
	}

	let mut image_data: Vec<u8> = Vec::new();
	PngEncoder::new_with_quality(
		&mut image_data,
		png::CompressionType::Best,
		png::FilterType::Adaptive,
	)
	.write_image(
		combined_image.as_raw(),
		combined_image.width(),
		combined_image.height(),
		ExtendedColorType::Rgba8,
	)?;

	Ok(image_data)
}
