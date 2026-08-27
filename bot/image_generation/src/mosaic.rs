use anyhow::{Context, Result};
use image::codecs::png;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage};
use palette::{FromColor, IntoColor, Lab, Srgb};
use rayon::prelude::*;
use palette::{
	cam16::{Cam16, Cam16UcsJab, Parameters, StaticWp, Surround},
	white_point::D65,
	 LinSrgb,  Xyz,
};
use crate::calculate::make_params;
use crate::color::{Color, ColorWithTile, find_closest_color_index};

pub fn generate_mosaic(
	guild_icon: &image::DynamicImage, average_colors: &[ColorWithTile],
) -> Result<Vec<u8>> {
	let tile_size: u32 = 32;
	// Guild icons are 128x128; each pixel becomes one tile, so canvas = 128 * 32 = 4096px
	let canvas_dim = 128 * tile_size;

	let mut combined_image = RgbaImage::new(canvas_dim, canvas_dim);

	let params = make_params();

	let indices: Vec<(u32, u32, usize)> = (0..guild_icon.height())
		.flat_map(|y| (0..guild_icon.width()).map(move |x| (x, y)))
		// Parallelize the expensive per-pixel color matching across CPU cores
		.par_bridge()
		.filter_map(|(x, y)| {
			let pixel = guild_icon.get_pixel(x, y);
			let alpha = pixel[3] as f32 / 255.0;

			if alpha < 0.1 {
				return None;
			}

			let srgb: Srgb<u8> = Srgb::new(pixel[0], pixel[1], pixel[2]);
			let linear: LinSrgb<f32> = srgb.into_linear();

			let linear = LinSrgb::new(
				linear.red   * alpha,
				linear.green * alpha,
				linear.blue  * alpha,
			);

			let xyz: Xyz<D65, f32> = linear.into_color();
			let cam16 = Cam16::from_xyz(xyz, params);
			let cam16_ucs = Cam16UcsJab::from_color(cam16);
			let color_target = Color { cam16: cam16_ucs };

			find_closest_color_index(average_colors, &color_target).map(|idx| (x, y, idx))
		})
		.collect();

	// Sequential placement: image mutation is not thread-safe, but matching was done in parallel above
	for (x, y, idx) in indices {
		image::imageops::replace(
			&mut combined_image,
			&average_colors[idx].tile,
			(x * tile_size) as i64,
			(y * tile_size) as i64,
		);
	}

	// Best compression + adaptive filter: mosaic PNGs are large (~4096x4096), worth the CPU cost
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

