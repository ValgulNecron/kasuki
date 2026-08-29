use crate::calculate::{COLOR_STRING_PREFIX, make_params, srgb_to_cam16ucs};
use anyhow::Context;
use image::RgbaImage;
use image::imageops::FilterType;
use palette::Srgb;
use palette::cam16::Cam16UcsJab;

#[derive(Clone, Debug)]
pub struct ColorWithTile {
	pub cam16: Cam16UcsJab<f32>,
	pub tile: RgbaImage,
}

// Uses the descriptor precomputed at user-color-calculation time (stored in the DB) instead of
// recomputing it per mosaic build. That stays consistent with the pasted tile because the
// display-referenced mean commutes with gamma-space downscaling: the mean of the stored
// thumbnail equals the mean of any resize of it, including the tile_size x tile_size tile.
pub fn create_color_tile(
	color_string: &str, png_bytes: &[u8], tile_size: u32,
) -> Option<ColorWithTile> {
	let cam16 = color_from_string(color_string).ok()?;
	let img = image::load_from_memory(png_bytes).ok()?;

	// Triangle (bilinear) filter: fast and sufficient for small tile thumbnails
	let tile = image::imageops::resize(&img, tile_size, tile_size, FilterType::Triangle);

	Some(ColorWithTile { cam16, tile })
}

// "cam16v2;J;a;b" is the precomputed display-referenced mean in CAM16-UCS. Legacy formats
// stay parseable so a half-migrated palette still renders: "cam16;J;a;b" (previous algorithm)
// and "#RRGGBB" (original gamma mean, converted on the fly).
pub fn color_from_string(s: &str) -> anyhow::Result<Cam16UcsJab<f32>> {
	if let Some(rest) = s
		.strip_prefix(COLOR_STRING_PREFIX)
		.or_else(|| s.strip_prefix("cam16;"))
	{
		let parts: Vec<&str> = rest.splitn(3, ';').collect();
		if parts.len() != 3 {
			anyhow::bail!("invalid cam16 color string: {s}");
		}
		let j: f32 = parts[0].parse().context("invalid J")?;
		let a: f32 = parts[1].parse().context("invalid a")?;
		let b: f32 = parts[2].parse().context("invalid b")?;
		Ok(Cam16UcsJab { lightness: j, a, b })
	} else if let Some(hex) = s.strip_prefix('#') {
		if hex.len() != 6 {
			anyhow::bail!("invalid hex color string: {s}");
		}
		let r = u8::from_str_radix(&hex[0..2], 16).context("invalid R")?;
		let g = u8::from_str_radix(&hex[2..4], 16).context("invalid G")?;
		let b = u8::from_str_radix(&hex[4..6], 16).context("invalid B")?;
		Ok(srgb_to_cam16ucs(
			Srgb::new(r, g, b).into_format(),
			make_params(),
		))
	} else {
		anyhow::bail!("unknown color format: {s}")
	}
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_current_version_string() {
		let c = color_from_string("cam16v2;61.5;-2.25;4.0").unwrap();
		assert_eq!(c.lightness, 61.5);
		assert_eq!(c.a, -2.25);
		assert_eq!(c.b, 4.0);
	}

	#[test]
	fn parses_legacy_cam16_string() {
		let c = color_from_string("cam16;40;1;-3").unwrap();
		assert_eq!(c.lightness, 40.0);
	}

	#[test]
	fn parses_legacy_hex_string() {
		// #808080 is mid-gray: near-zero chroma, mid lightness
		let c = color_from_string("#808080").unwrap();
		assert!(c.lightness > 30.0 && c.lightness < 80.0, "{c:?}");
		assert!(c.a.abs() < 3.0 && c.b.abs() < 3.0, "{c:?}");
	}

	#[test]
	fn rejects_unknown_format() {
		assert!(color_from_string("rgb(1,2,3)").is_err());
		assert!(color_from_string("cam16v2;1;2").is_err());
	}
}
