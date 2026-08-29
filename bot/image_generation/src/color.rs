use crate::calculate::{make_params, mean_displayed_srgb, srgb_to_cam16ucs};
use image::RgbaImage;
use image::imageops::FilterType;
use palette::cam16::Cam16UcsJab;

#[derive(Clone, Debug)]
pub struct ColorWithTile {
	pub cam16: Cam16UcsJab<f32>,
	pub tile: RgbaImage,
}

// The descriptor is computed from the exact tile_size x tile_size pixels that get pasted into
// the mosaic, not from the per-user color string stored in the database: the matcher then
// always describes precisely the artifact it places, whatever algorithm produced the DB value.
pub fn create_color_tile(png_bytes: &[u8], tile_size: u32) -> Option<ColorWithTile> {
	let img = image::load_from_memory(png_bytes).ok()?;

	// Triangle (bilinear) filter: fast and sufficient for small tile thumbnails
	let tile = image::imageops::resize(&img, tile_size, tile_size, FilterType::Triangle);

	let mean = mean_displayed_srgb(&tile)?;
	let cam16 = srgb_to_cam16ucs(mean, make_params());

	Some(ColorWithTile { cam16, tile })
}

// Finds the tile whose displayed average color is perceptually closest to the target
pub fn find_closest_color_index(
	colors: &[ColorWithTile], target: &Cam16UcsJab<f32>,
) -> Option<usize> {
	colors
		.iter()
		.enumerate()
		.min_by(|(_, a), (_, b)| {
			let da = cam16_delta_e_weighted(&a.cam16, target);
			let db = cam16_delta_e_weighted(&b.cam16, target);
			// partial_cmp because f32; fallback to Equal handles potential NaN from degenerate colors
			da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
		})
		.map(|(i, _)| i)
}

// Euclidean distance in CAM16-UCS, with lightness weighted above chromaticity: at mosaic scale
// the J channel is what carries the structure of the source icon (edges, facial features), so
// a tile that matches in hue but not in lightness reads as a hole in the image.
pub fn cam16_delta_e_weighted(a: &Cam16UcsJab<f32>, b: &Cam16UcsJab<f32>) -> f32 {
	let dj = a.lightness - b.lightness;
	let da = a.a - b.a;
	let db = a.b - b.b;

	const W_J: f32 = 1.4;
	const W_C: f32 = 1.0;

	((W_J * dj * dj) + (W_C * (da * da + db * db))).sqrt()
}
