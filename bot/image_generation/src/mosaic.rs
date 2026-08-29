use crate::calculate::{make_params, srgb_to_cam16ucs};
use crate::color::{ColorWithTile, find_closest_color_index};
use anyhow::Result;
use image::codecs::png;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, RgbaImage};
use palette::Srgb;
use rayon::prelude::*;

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

			// Composite over white, matching how tile descriptors resolve alpha: the mosaic
			// is an opaque image on a white ground, so transparent icon regions become white
			// targets instead of holes, and semi-transparent edges match their displayed color
			let alpha = pixel[3] as u32;
			let inverse = 255 - alpha;
			let composite = |c: u8| ((c as u32 * alpha + 255 * inverse) / 255) as u8;

			let srgb: Srgb<f32> = Srgb::new(
				composite(pixel[0]),
				composite(pixel[1]),
				composite(pixel[2]),
			)
			.into_format();
			let target = srgb_to_cam16ucs(srgb, params);

			find_closest_color_index(average_colors, &target).map(|idx| (x, y, idx))
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
