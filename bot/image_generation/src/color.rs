use crate::calculate::cam16ucs_to_lab;
use anyhow::Context;
use image::RgbaImage;
use image::imageops::FilterType;
use palette::cam16::Cam16UcsJab;
use palette::color_difference::ImprovedDeltaE;
use palette::{IntoColor, Lab, Srgb};

#[derive(Clone, Debug)]
pub struct Color {
	pub cielab: Lab,
}

#[derive(Clone, Debug)]
pub struct ColorWithTile {
	pub cielab: Lab,
	pub tile: RgbaImage,
}

pub fn create_color_tile(
	color_string: &str, png_bytes: &[u8], tile_size: u32,
) -> Option<ColorWithTile> {
	let img = image::load_from_memory(png_bytes).ok()?;

	let color = match color_from_string(color_string) {
		Ok(a) => a,
		_ => return None,
	};

	// Triangle (bilinear) filter: fast and sufficient for small tile thumbnails
	let tile = image::imageops::resize(&img, tile_size, tile_size, FilterType::Triangle);

	Some(ColorWithTile {
		cielab: color.cielab,
		tile,
	})
}

// Finds the tile whose average color is perceptually closest to the target
pub fn find_closest_color_index(colors: &[ColorWithTile], target: &Color) -> Option<usize> {
	colors
		.iter()
		.enumerate()
		.min_by(|(_, a), (_, b)| {
			// CIEDE2000 ("improved delta E"): more accurate than Euclidean RGB distance
			let delta_e_a = a.cielab.improved_delta_e(target.cielab);
			let delta_e_b = b.cielab.improved_delta_e(target.cielab);
			// partial_cmp because f32; fallback to Equal handles potential NaN from degenerate colors
			delta_e_a
				.partial_cmp(&delta_e_b)
				.unwrap_or(std::cmp::Ordering::Equal)
		})
		.map(|(i, _)| i)
}

// Stored user colors come in two flavours: the CAM16-UCS triple written by the mean-color pass
// ("cam16;J;a;b") and legacy "#RRGGBB" hex. Both are normalized to CIELAB, the space matching runs in.
pub fn color_from_string(s: &str) -> anyhow::Result<Color> {
	if let Some(rest) = s.strip_prefix("cam16;") {
		let parts: Vec<&str> = rest.splitn(3, ';').collect();
		if parts.len() != 3 {
			anyhow::bail!("invalid cam16 color string: {s}");
		}
		let j: f32 = parts[0].parse().context("invalid J")?;
		let a: f32 = parts[1].parse().context("invalid a")?;
		let b: f32 = parts[2].parse().context("invalid b")?;
		let cam16 = Cam16UcsJab { lightness: j, a, b };
		Ok(Color {
			cielab: cam16ucs_to_lab(cam16),
		})
	} else if let Some(hex) = s.strip_prefix('#') {
		if hex.len() != 6 {
			anyhow::bail!("invalid hex color string: {s}");
		}
		let r = u8::from_str_radix(&hex[0..2], 16).context("invalid R")?;
		let g = u8::from_str_radix(&hex[2..4], 16).context("invalid G")?;
		let b = u8::from_str_radix(&hex[4..6], 16).context("invalid B")?;
		let srgb: Srgb<f32> = Srgb::new(r, g, b).into_format();
		Ok(Color {
			cielab: srgb.into_color(),
		})
	} else {
		anyhow::bail!("unknown color format: {s}")
	}
}
